extern crate clap;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "tool",
    about = "A tool for building, testing and debugging Tacos."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build the project.
    Build(BuildArgs),
    /// Specify and run test cases.
    Test(TestArgs),
    /// Remember specific test cases.
    Book(BookArgs),
}

/* ---------------------------------- BUILD --------------------------------- */

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Clean.
    #[arg(short, long)]
    pub clean: bool,

    /// Clean and rebuild.
    /// `--clean` will be ignored.
    #[arg(short, long)]
    pub rebuild: bool,

    /// Show the build process.
    #[arg(short, long)]
    pub verbose: bool,
}

/* ---------------------------------- TEST ---------------------------------- */

#[derive(Args, Debug)]
pub struct TestArgs {
    /// The test cases to run. Only the name of test case is required.
    ///
    /// Example:
    /// `tool test -c args-none`
    /// , `tool test -c args-none,args-many`
    #[arg(short, long, value_delimiter = ',')]
    pub cases: Vec<String>,

    /// Load specified bookmarks and add to the test suite.
    #[arg(short, long, value_delimiter = ',')]
    pub books: Vec<String>,

    /// Add test cases that failed in previous run to the test suite.
    #[arg(short, long)]
    pub previous_failed: bool,

    /// Only show the command line to run, without starting it.
    #[arg(long)]
    pub dry: bool,

    /// Run in gdb mode. Only receive single test case.
    #[arg(long)]
    pub gdb: bool,

    /// Grading after test.
    #[arg(short, long)]
    pub grade: bool,

    /// Verbose mode without testing. Suppress `--gdb` and `--grade`.
    #[arg(short, long)]
    pub verbose: bool,
}

/* -------------------------------- BOOKMARK -------------------------------- */

#[derive(Args, Debug)]
pub struct BookArgs {
    /// The name of the bookmark.
    ///
    /// Bookmark will be saved in `/bookmarks/<name>.json`
    #[arg(short, long)]
    pub name: String,

    /// Delete mode. Turn 'add' into 'delete' in other options.
    ///
    /// If no test cases are specified, delete the whole bookmark (with its file!).
    #[arg(short, long)]
    pub del: bool,

    /// Test cases to add in this bookmark.
    #[arg(short, long, value_delimiter = ',')]
    pub cases: Vec<String>,

    /// Bookmarks to load and add to this bookmark.
    #[arg(short, long, value_delimiter = ',')]
    pub books: Vec<String>,

    /// Add test cases that failed in previous run to the bookmark.
    #[arg(short, long)]
    pub previous_failed: bool,
}
