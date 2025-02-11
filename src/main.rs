use commands::ProjectCommand;
use input::Fail;

mod commands;
mod config;
mod data_store;
mod input;

#[derive(clap::Parser)]
pub struct Arguments {
    #[clap(subcommand)]
    project: ProjectCommand,
}

fn main() {
    let data = data_store::DataStore::load().unwrap_or_fail();
    println!("{:?}", data);
}
