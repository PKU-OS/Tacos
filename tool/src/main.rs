mod book;
mod build;
mod cli;
mod test;

fn main() -> std::io::Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Build(args) => build::main(args),
        cli::Commands::Test(args) => test::main(args),
        cli::Commands::Book(args) => book::main(args),
    }
}
