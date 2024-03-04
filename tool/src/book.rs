extern crate once_cell;
extern crate serde;
extern crate toml;

use crate::cli::BookArgs;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cases(pub HashMap<String, Case>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Case(
    /// Args.
    pub String,
    /// Grade. None = DEFAULT_GRADE.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Option<usize>,
    /// Timeout. None = DEFAULT_Timeout.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Option<u64>,
);

const PREVIOUS_FAILED_BOOK_NAME: &str = "previous-failed";
const DEFAULT_GRADE: usize = 1;
const DEFAULT_TIMEOUT: u64 = 60;

const BUILTIN_NAMES: [&str; 5] = ["unit", "lab1", "lab2", "lab3", PREVIOUS_FAILED_BOOK_NAME];

static BUILTINS: Lazy<[HashMap<String, Case>; 5]> = Lazy::new(|| {
    let mut b = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];
    for (i, builtin) in BUILTIN_NAMES.iter().enumerate() {
        if let Ok(cases) = read_book(builtin) {
            b[i].extend(cases.0);
        }
    }
    b
});

static ALL_BUILTIN: Lazy<HashMap<String, Case>> = Lazy::new(|| {
    let mut all = HashMap::new();
    for builtin in BUILTINS.iter() {
        for (k, v) in builtin.iter() {
            all.insert(k.clone(), v.clone());
        }
    }
    all
});

/* -------------------------------------------------------------------------- */
/*                                PUBLIC TOOLS                                */
/* -------------------------------------------------------------------------- */

pub fn main(args: BookArgs) -> Result<()> {
    for b in BUILTIN_NAMES {
        if args.name == b {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot modify built-in bookmark: {}", b),
            ));
        }
    }
    let mut original = Cases(HashMap::new());
    if let Ok(cases) = read_book(&args.name) {
        original.0.extend(cases.0);
    }
    let cases = if args.del {
        action(&args, original, |map, k, _v| {
            map.remove(k);
        })
        // del_mode(&args, original)
    } else {
        action(&args, original, |map, k, v| {
            map.insert(k.clone(), v.clone());
        })
        // add_mode(&args, original)
    };
    if args.del && args.cases.is_empty() {
        std::fs::remove_file(book_path(&args.name))?;
    } else {
        write_book(&args.name, &cases)?;
    }
    Ok(())
}

/// Return test cases from test arguments. The cases are separated into 'unit', 'lab1', 'lab2', 'lab3'.
pub fn test_cases(args: &crate::cli::TestArgs) -> Result<(Cases, Cases, Cases, Cases)> {
    let mut selected = HashSet::<String>::new();
    for case in &args.cases {
        selected.insert(case.clone());
    }
    for book in &args.books {
        if let Ok(cases) = read_book(book) {
            for (k, _) in cases.0 {
                selected.insert(k);
            }
        }
    }
    if args.previous_failed {
        for (k, _) in BUILTINS[4].iter() {
            selected.insert(k.clone());
        }
    }

    let unit = from_builtin(0, &selected);
    let lab1 = from_builtin(1, &selected);
    let lab2 = from_builtin(2, &selected);
    let lab3 = from_builtin(3, &selected);
    Ok((unit, lab1, lab2, lab3))
}

pub fn grade(case: &String) -> Option<usize> {
    ALL_BUILTIN.get(case)?.1.or(Some(DEFAULT_GRADE))
}

pub fn timeout(case: &String) -> Option<u64> {
    ALL_BUILTIN.get(case)?.2.or(Some(DEFAULT_TIMEOUT))
}

pub fn write_previous_failed(cases: &Vec<String>) -> Result<()> {
    let mut previous_failed = Cases(HashMap::new());
    for case in cases {
        if ALL_BUILTIN.contains_key(case) {
            let v = ALL_BUILTIN.get(case).unwrap().clone();
            previous_failed.0.insert(case.clone(), v);
        }
    }
    write_book(PREVIOUS_FAILED_BOOK_NAME, &previous_failed)?;
    Ok(())
}

/* -------------------------------------------------------------------------- */
/*                                   HLEPERS                                  */
/* -------------------------------------------------------------------------- */

fn from_builtin(builtin: usize, selected: &HashSet<String>) -> Cases {
    let mut cases = Cases(HashMap::new());
    for (k, v) in BUILTINS[builtin].iter() {
        if selected.contains(k) {
            cases.0.insert(k.clone(), v.clone());
        }
    }
    cases
}

fn action(
    args: &BookArgs,
    mut cases: Cases,
    action: impl Fn(&mut HashMap<String, Case>, &String, &Case),
) -> Cases {
    for case in &args.cases {
        if ALL_BUILTIN.contains_key(case) {
            let k = case;
            let v = ALL_BUILTIN.get(case).unwrap();
            action(&mut cases.0, k, v);
        }
    }
    for books in &args.books {
        if let Ok(c) = read_book(books) {
            for (k, v) in &c.0 {
                if ALL_BUILTIN.contains_key(k) {
                    action(&mut cases.0, k, v);
                }
            }
        }
    }
    if args.previous_failed {
        for (k, v) in BUILTINS[4].iter() {
            if ALL_BUILTIN.contains_key(k) {
                action(&mut cases.0, k, v);
            }
        }
    }
    cases
}

fn read_book(name: &str) -> Result<Cases> {
    let file = File::open(book_path(name))?;
    let content = std::io::read_to_string(file)?;
    let cases = toml::from_str(&content).expect("failed to read toml");
    Ok(cases)
}

fn write_book(name: &str, cases: &Cases) -> Result<()> {
    let mut file = File::create(book_path(name))?;
    let content = toml::to_string(cases).expect("failed to serialize toml");
    std::io::Write::write_all(&mut file, content.as_bytes())?;
    Ok(())
}

fn book_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from("./bookmarks");
    path.push(name);
    path.set_extension("toml");
    path
}
