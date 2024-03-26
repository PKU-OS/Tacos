use std::process;

const MAKE_DIR: &str = "..";

pub fn main(args: crate::cli::BuildArgs) -> std::io::Result<()> {
    if args.rebuild {
        make(args.verbose, &vec!["clean-tacos"])?;
        make(args.verbose, &vec![])?;
    } else if args.clean {
        make(args.verbose, &vec!["clean-tacos"])?;
    } else {
        make(args.verbose, &vec![])?;
    }
    Ok(())
}

fn make(verbose: bool, args: &Vec<&str>) -> std::io::Result<()> {
    let mut child = process::Command::new("make");
    child.current_dir(MAKE_DIR).stdin(process::Stdio::piped());
    if !verbose {
        child
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped());
    }
    let mut child = child.args(args).spawn()?;
    child.wait()?;
    Ok(())
}
