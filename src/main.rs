use clap::Parser;
use prj::cli::args::Cli;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    prj::cli::run(cli)
}
