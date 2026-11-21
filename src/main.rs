mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    let mut args = Cli::parse();
    args.resolve_flags();

    println!("Parsed CLI Arguments: {:#?}", args);
}
