mod cli;
mod tui;
mod shell;

use clap::Parser;
use cli::args::Cli;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    cli::run(cli)
}
