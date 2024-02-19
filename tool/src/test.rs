extern crate colored;
extern crate ctrlc;
extern crate once_cell;

use colored::*;
use once_cell::sync::{Lazy, OnceCell};
use std::io::Result;
use std::process::Output;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::book::Cases;

const OS_DIR: &str = "..";

type Runner = fn(&String, Vec<&str>, &mut Record) -> Result<()>;
static RUNNER: OnceCell<Runner> = OnceCell::new();
static CTRLC: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static GDB: OnceCell<bool> = OnceCell::new();

struct Record(Vec<String>, Vec<String>);

pub fn main(mut args: crate::cli::TestArgs) -> Result<()> {
    // Rebuild the os.
    if !args.dry {
        build()?;
    }
    // Avoid CTRL-C on QEMU break the file system.
    set_ctrlc_handler();
    // Create a record.
    let record = &mut Record(Vec::new(), Vec::new());
    // Get the cases and its belonging lab.
    let (unit, lab1, lab2, lab3) = crate::book::test_cases(&args)?;
    // Suppress gdb and grading when running verbose mode.
    if args.verbose {
        args.gdb = false;
        args.previous_failed = false;
        args.grade = false;
    }
    // Check and set for gdb mode.
    if args.gdb {
        let _chk = ((unit.0.len() + lab1.0.len() + lab2.0.len() + lab3.0.len()) <= 1)
            .then_some(())
            .expect(&format!(
                "{}",
                "More than 1 cases in GDB mode is forbidden!".bold().red()
            ));
        GDB.get_or_init(|| true);
    } else {
        GDB.get_or_init(|| false);
    }
    // Set runner.
    if args.dry {
        RUNNER.get_or_init(|| dry_run);
    } else if args.verbose {
        RUNNER.get_or_init(|| verbose_run);
    } else {
        RUNNER.get_or_init(|| run);
    }
    // Test.
    let _ = test_krnl(unit, record);
    let _ = test_schedule(lab1, record);
    let _ = test_user(lab2, record);
    let _ = test_user(lab3, record);
    // Grading. (don't when CTRL-C)
    if !args.dry && !args.gdb && !CTRLC.load(std::sync::atomic::Ordering::SeqCst) {
        if args.grade {
            let (passed, total) = grading(record);
            println!("{}: {}/{}", "GRADE".bold().magenta(), passed, total);
        }
        crate::book::write_previous_failed(&record.1)?;
    }
    Ok(())
}

fn test_user(cases: Cases, record: &mut Record) -> Result<()> {
    for (k, v) in cases.0 {
        let args = format!("{}{}{}", &k, if !v.0.is_empty() { " " } else { "" }, &v.0);
        let mut cargo = vec!["run", "-r", "-q", "-F", "test-user", "--", "-append", &args];
        if *GDB.get().unwrap() {
            // to debug mode
            cargo.remove(1);
            cargo.extend(["-s", "-S"].iter());
        }
        let runner = RUNNER.get().unwrap();
        let _ = runner(&k, cargo, record);
        if CTRLC.load(std::sync::atomic::Ordering::SeqCst) {
            println!("{}", "Early stopping...".bold());
            break;
        }
    }
    Ok(())
}

fn test_krnl(cases: Cases, record: &mut Record) -> Result<()> {
    for (k, _v) in cases.0 {
        let feature = format!("test-{}", &k);
        let mut cargo = vec!["run", "-r", "-q", "-F", &feature];
        if *GDB.get().unwrap() {
            // to debug mode
            cargo.remove(1);
            cargo.extend(["--", "-s", "-S"].iter());
        }
        let runner = RUNNER.get().unwrap();
        let _ = runner(&k, cargo, record);
        if CTRLC.load(std::sync::atomic::Ordering::SeqCst) {
            println!("{}", "Early stopping...".bold());
            break;
        }
    }
    Ok(())
}

fn test_schedule(cases: Cases, record: &mut Record) -> Result<()> {
    for (k, v) in cases.0 {
        let args = format!("{}{}{}", &k, if !v.0.is_empty() { " " } else { "" }, &v.0);
        let mut cargo = vec![
            "run",
            "-r",
            "-q",
            "-F",
            "test-schedule",
            "--",
            "-append",
            &args,
        ];
        if *GDB.get().unwrap() {
            // to debug mode
            cargo.remove(1);
            cargo.extend(["-s", "-S"].iter());
        }
        let runner = RUNNER.get().unwrap();
        let _ = runner(&k, cargo, record);
        if CTRLC.load(std::sync::atomic::Ordering::SeqCst) {
            println!("{}", "Early stopping...".bold());
            break;
        }
    }
    Ok(())
}

fn run(case: &String, args: Vec<&str>, record: &mut Record) -> Result<()> {
    use std::io::Write;
    let child = std::process::Command::new("cargo")
        .current_dir(OS_DIR)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .args(args)
        .spawn()?;
    if *GDB.get().unwrap() {
        // TODO: functionality not tested.
        println!("{}", "Use \'gdb-multiarch\' to debug.".bold().italic());
    } else {
        let timeout = crate::book::timeout(case).unwrap();
        print!("Running {} ... ", case.bold());
        let _ = std::io::stdout().flush();
        let output = wait_timeout(child, timeout)?;
        check(case, output, record);
    }
    Ok(())
}

enum Wait {
    Timeout,
    Output(Output),
}

fn wait_timeout(mut child: std::process::Child, timeout: u64) -> Result<Wait> {
    let start = Instant::now();
    // first sec busy wait
    while child.try_wait()?.is_none() {
        if start.elapsed().as_secs() >= 1 {
            break;
        }
    }
    if start.elapsed().as_secs() >= timeout {
        child.kill()?;
        return Ok(Wait::Timeout);
    }
    while child.try_wait()?.is_none() {
        std::thread::sleep(Duration::from_secs(1));
        if start.elapsed().as_secs() >= timeout {
            child.kill()?;
            return Ok(Wait::Timeout);
        }
    }
    child.wait_with_output().map(Wait::Output)
}

fn check(case: &str, output: Wait, record: &mut Record) {
    let output = match output {
        Wait::Timeout => {
            print!("(TIMEOUT) ");
            fail(case, record);
            return;
        }
        Wait::Output(output) => output,
    };
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(()) = stdout
            .lines()
            .last()
            .and_then(|s| s.contains("Goodbye, World!").then_some(()))
        {
            pass(case, record);
        } else {
            fail(case, record);
            {
                println!("{}", "STDOUT:".bold().underline().italic().cyan());
                let lines: Vec<&str> = stdout.lines().collect();
                let len = lines.len();
                for i in 0.max(len - 10)..len {
                    println!("{}", lines[i]);
                }
                // Actually nonthing will be in STDERR.
                /*
                println!("{}", "STDERR:".bold().underline().italic().bright_blue());
                let lines: Vec<&str> = stderr.lines().collect();
                let len = lines.len();
                for i in 0.max(len - 10)..len {
                    println!("{}", lines[i]);
                }
                */
            }
        }
    } else {
        println!(
            "{} @ {:?}",
            "FAILED TO RUN QEMU".bright_yellow(),
            &output.status
        );
    }
}

fn pass(case: &str, record: &mut Record) {
    record.0.push(case.to_owned());
    println!("{}", "PASSED".bold().green());
}

fn fail(case: &str, record: &mut Record) {
    record.1.push(case.to_owned());
    println!("{}", "FAILED".bold().red());
}

fn grading(record: &mut Record) -> (usize, usize) {
    let mut total = 0usize;
    let mut pass = 0usize;
    for case in &record.0 {
        let g = crate::book::grade(case).unwrap();
        total += g;
        pass += g;
    }
    for case in &record.1 {
        let g = crate::book::grade(case).unwrap();
        total += g;
    }
    (pass, total)
}

fn verbose_run(case: &String, args: Vec<&str>, _record: &mut Record) -> Result<()> {
    println!("==================== {} ====================", case.bold());
    let _child = std::process::Command::new("cargo")
        .current_dir(OS_DIR)
        .stdin(std::process::Stdio::piped())
        .args(args)
        .spawn()?
        .wait()?;
    Ok(())
}

fn dry_run(case: &String, args: Vec<&str>, _record: &mut Record) -> Result<()> {
    println!("{}", &format!("Command for {}:", case.bold()).dimmed());
    println!(
        "cargo {}",
        args.iter()
            // manually add \"\" to long args.
            // need not to do this in `std::process`.
            .map(|st| if st.contains(' ') {
                format!("\"{}\"", st)
            } else {
                st.to_string()
            })
            .collect::<Vec<String>>()
            .join(" ")
    );
    Ok(())
}

fn build() -> Result<()> {
    let args = crate::cli::BuildArgs {
        clean: false,
        rebuild: true,
        verbose: false,
    };
    crate::build::main(args)
}

fn set_ctrlc_handler() {
    let ctrlc = CTRLC.clone();
    ctrlc::set_handler(move || {
        ctrlc.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .unwrap();
}
