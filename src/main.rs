use clap::{Parser, Subcommand};
use colored::*;
use prettytable::{format, Cell, Row, Table};
use std::env::current_dir;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

mod project;
use project::{project_state_dir, Project, ProjectStorage, ProjectType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new project
    Add {
        /// Project name (optional; will prompt if missing)
        name: Option<String>,
        /// Project type (optional; will prompt if missing)
        #[arg(value_enum)]
        project_type: Option<ProjectType>,
        /// Path to the project (optional; defaults to current directory if not provided)
        path: Option<PathBuf>,
    },
    /// Print project path
    PrintPath {
        /// Project name to get path
        name: String,
    },
    /// Remove a project by name or path
    Remove {
        /// Project name to remove
        #[arg(long)]
        name: Option<String>,
        /// Path to the project to remove
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// List all projects
    List,
    /// Setup the project manager
    Setup,
    /// Clone a new GitHub repository and add as a project
    Clone {
        /// GitHub repository URL to clone
        repo_url: String,
        /// Optional project type (e.g., Cargo, CMake)
        #[arg(value_enum, long = "type")]
        project_type: Option<ProjectType>,
        #[arg(long = "name")]
        /// Optional project name (defaults to repository name)
        project_name: Option<String>,
        #[arg(long = "path")]
        project_path: Option<PathBuf>,
        /// Additional Git options (like branch)
        #[arg(last = true)]
        git_options: Vec<String>,
    },
}

impl FromStr for ProjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CMake" => Ok(ProjectType::CMake),
            "Cargo" => Ok(ProjectType::Cargo),
            _ => Err(format!("Invalid project type: {}", s)),
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut storage = ProjectStorage::load_or_initialize();

    match args.command {
        Commands::Add {
            name,
            project_type,
            path,
        } => {
            let name =
                name.unwrap_or_else(|| prompt("Enter project name: ".bold().green().to_string()));
            let project_type = project_type.unwrap_or_else(|| {
                prompt_enum(
                    "Enter project type (CMake, Cargo): "
                        .bold()
                        .green()
                        .to_string(),
                    &["CMake", "Cargo"],
                )
            });
            let path = path.unwrap_or_else(|| {
                std::env::current_dir().expect("Unable to get current directory")
            });

            let project = Project {
                name,
                project_type,
                path,
            };
            storage.add_project(project);
            println!("{}", "Project added successfully.".bold().blue());
        }
        Commands::PrintPath { name } => {
            if let Some(path) = storage.get_project_path(&name) {
                println!("{}", path.display());
            } else {
                println!("{}", "Project not found.".bold().red());
            }
        }
        Commands::Remove { name, path } => {
            if name.is_some() || path.is_some() {
                storage.remove_project(name, path);
                println!("{}", "Project removed successfully.".bold().blue());
            } else {
                println!("{}", "Please specify a project name or path.".bold().red());
            }
        }
        Commands::List => {
            println!("{}", "Listing all projects:".bold().blue());
            display_projects(&storage);
        }
        Commands::Setup => {
            setup_script();
        }
        Commands::Clone {
            repo_url,
            project_type,
            project_name,
            project_path,
            git_options,
        } => {
            clone_and_add_project(
                &repo_url,
                project_type,
                project_name,
                project_path,
                git_options,
                &mut storage,
            );
        }
    }
}

fn clone_and_add_project(
    repo_url: &str,
    project_type: Option<ProjectType>,
    project_name: Option<String>,
    project_path: Option<PathBuf>,
    git_options: Vec<String>,
    storage: &mut ProjectStorage,
) {
    // Parse the repository name from the URL
    let git_name = repo_url
        .split('/')
        .last()
        .expect("Invalid repo URL")
        .replace(".git", "");
    let repo_name = project_name.unwrap_or(git_name.clone());

    // Define the clone path
    let clone_path = current_dir()
        .expect("Unable to get current directory")
        .join(project_path.unwrap_or_else(|| PathBuf::from(&git_name)));

    println!(
        "{} {}",
        "Cloning repository to:".bold().green(),
        clone_path.display()
    );

    // Prepare the `git clone` command with additional options
    let mut command = Command::new("git");
    command.arg("clone").arg(repo_url).arg(&clone_path);

    // Add additional git options
    for arg in git_options {
        command.arg(arg);
    }

    // Execute the `git clone` command
    let status = command.status();

    match status {
        Ok(status) if status.success() => {
            let project = Project {
                name: repo_name.clone(),
                project_type: project_type.unwrap_or(ProjectType::Other),
                path: clone_path,
            };
            storage.add_project(project);
            println!("{}", "Project cloned and added successfully.".bold().blue());
        }
        Ok(_) => println!(
            "{} Git command exited with an error.",
            "Failed to clone repository:".bold().red()
        ),
        Err(e) => println!("{} {}", "Failed to clone repository:".bold().red(), e),
    }
}

fn setup_script() {
    let script_content = include_str!("pj.sh");
    let script_path = project_setup_script_path();

    fs::write(&script_path, script_content).expect("Failed to write setup script");

    let shell_rc = if cfg!(target_os = "macos") {
        ".zshrc"
    } else {
        ".bashrc"
    };
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    let rc_file = home_dir.join(shell_rc);

    let source_line = format!("source {}", script_path.display());
    if let Ok(contents) = fs::read_to_string(&rc_file) {
        if contents.contains(&source_line) {
            println!(
                "{}",
                "Setup already completed. No changes made.".bold().yellow()
            );
            return;
        }
    }

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&rc_file)
        .expect("Failed to open shell rc file");
    writeln!(
        file,
        "\n# Source prj project manager setup script\n{}",
        source_line
    )
    .expect("Failed to write to shell rc file");

    println!(
        "{} {}",
        "Setup completed.".bold().green(),
        format!(
            "Please restart your terminal or run 'source ~/{shell_rc}' to activate 'pj' command."
        )
        .italic()
        .yellow()
    );
}

fn project_setup_script_path() -> PathBuf {
    let mut path = project_state_dir();
    path.push("setup.sh");
    path
}

fn display_projects(storage: &ProjectStorage) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.set_titles(Row::new(vec![
        Cell::new(&"Name".bold().green()),
        Cell::new(&"Type".bold().green()),
        Cell::new(&"Path".bold().green()),
    ]));

    for project in &storage.projects {
        table.add_row(Row::new(vec![
            Cell::new(&project.name),
            Cell::new(&format!("{:?}", project.project_type).cyan()),
            Cell::new(&project.path.display().to_string().italic()),
        ]));
    }

    table.printstd();
}

/// Prompt the user for a string input
fn prompt(message: String) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

/// Prompt the user for an enum input
fn prompt_enum<T: std::str::FromStr>(message: String, options: &[&str]) -> T {
    loop {
        let input = prompt(message.clone());
        if let Ok(value) = input.parse::<T>() {
            return value;
        } else {
            println!(
                "{}",
                format!(
                    "Invalid option. Available options are: {}",
                    options.join(", ")
                )
                .bold()
                .red()
            );
        }
    }
}
