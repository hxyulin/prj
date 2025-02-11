#[derive(clap::Subcommand)]
pub enum ProjectCommand {
    New(NewCommand),
    Info(InfoCommand),
}

#[derive(clap::Args)]
pub struct NewCommand {
    pub name: String,
}

#[derive(clap::Args)]
pub struct InfoCommand {
    pub name: String,
}
