use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select, Sort};
use prettytable::{format, Cell, Row, Table};
use std::env::current_dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

mod input;
mod project;

use project::{project_state_dir, Project, ProjectStorage, ProjectType};

#[derive(Parser, Debug)]
#[command(name = "prj", version, about, long_about = None)]
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
        name: Option<String>,
    },
    /// Remove a project by name or path
    Remove {
        /// Project name to remove
        #[arg(short, long)]
        name: Option<String>,
        /// Path to the project to remove
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// List all projects
    List {
        /// Enable interactive mode
        #[arg(short, long)]
        interactive: bool,
        /// Allow selecting multiple projects
        /// (only applicable in interactive mode)
        /// Keyboard shortcuts:
        /// - `TAB`: Select multiple items
        /// - `CTRL-A`: Select all items
        /// - `CTRL-U`: Clear query
        #[arg(short, long)]
        multi: bool,
    },
    /// Reorder projects
    Reorder,
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
            "Other" => Ok(ProjectType::Other),
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
            // Current directory name as default project name
            let name = name.unwrap_or_else(|| {
                let current_dir_name = current_dir()
                    .expect("Unable to get current directory")
                    .file_name()
                    .expect("Unable to get current directory name")
                    .to_string_lossy()
                    .to_string();

                input::prompt(
                    "Enter project name: ".bold().green().to_string(),
                    Some(current_dir_name),
                )
            });
            let project_type = project_type.unwrap_or_else(|| {
                match input::prompt_enum(
                    "Enter project type (CMake, Cargo, Other): "
                        .bold()
                        .green()
                        .to_string(),
                    &["CMake", "Cargo", "Other"],
                    Some("Other".to_string()),
                ) {
                    Some(project_type) => project_type,
                    None => {
                        println!("{}", "Project creation canceled.".bold().yellow());
                        std::process::exit(0);
                    }
                }
            });
            let path = path.unwrap_or_else(|| {
                let input = input::prompt_empty(
                    "Enter project path (empty for current directory): "
                        .bold()
                        .green()
                        .to_string()
                );

                if input.is_empty() {
                    current_dir().expect("Unable to get current directory")
                } else {
                    match fs::canonicalize(PathBuf::from(input)) {
                        Ok(path) => path,
                        Err(e) => {
                            eprintln!("Failed to resolve path: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            });

            if !path.exists() {
                eprintln!("{}", "Path does not exist.".bold().red());
                return;
            }

            if !path.is_dir() {
                eprintln!("{}", "Path is not a directory.".bold().red());
                return;
            }

            let project = Project {
                name,
                project_type,
                path,
            };
            storage.add_project(project);
            println!("{}", "Project added successfully.".bold().blue());
        }
        Commands::PrintPath { name } => {
            let name = name.unwrap_or_else(|| {
                input::choose_project_name(&storage, false)
                    .map(|names| names.first().unwrap().clone())
                    .unwrap_or_else(|| {
                        input::prompt("Enter project name: ".bold().green().to_string(), None)
                    })
            });

            if let Some(path) = storage.get_project_path(&name) {
                println!("{}", "Project path:".bold().green());
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
        Commands::List { interactive, multi } => {
            if interactive {
                let names = match input::choose_project_name(&storage, multi) {
                    Some(names) => names,
                    None => return,
                };

                let projects = storage
                    .projects
                    .iter()
                    .filter(|project| names.contains(&project.name))
                    .cloned()
                    .collect::<Vec<_>>();

                let actions = vec!["Print Path", "Remove", "Exit"];
                let action_index = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "Select an action for '{}'",
                        if projects.len() == 1 {
                            projects.first().unwrap().name.clone()
                        } else {
                            format!("{} projects", projects.len())
                        }
                    ))
                    .items(&actions)
                    .default(0)
                    .interact_opt()
                    .expect("Failed to open action menu")
                    .unwrap_or(2);

                match action_index {
                    0 => {
                        for project in projects {
                            println!("{}", "Project path:".bold().green());
                            println!("{}", project.path.display());
                        }
                    }
                    1 => {
                        // "Remove"
                        let confirm = Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!(
                                "Are you sure you want to remove '{}'",
                                if projects.len() == 1 {
                                    projects.first().unwrap().name.clone()
                                } else {
                                    format!("{} projects", projects.len())
                                }
                            ))
                            .default(false)
                            .interact_opt()
                            .expect("Failed to capture confirmation")
                            .unwrap_or(false);

                        if confirm {
                            for project in projects {
                                storage.remove_project(Some(project.name.clone()), None);
                                println!("{} '{}'", "Removed project".bold().red(), project.name);
                            }
                        } else {
                            println!("Removal canceled.");
                        }
                    }
                    2 => {}
                    _ => unreachable!(),
                }
            } else {
                println!("{}", "Listing all projects:".bold().blue());
                display_projects(&storage);
            }
        }
        Commands::Reorder => {
            let theme = ColorfulTheme::default();
            let reorder_indices = Sort::with_theme(&theme)
                .with_prompt("Reorder projects (space to select)")
                .items(&storage.projects.iter().map(|p| &p.name).collect::<Vec<_>>())
                .interact_opt()
                .expect("Failed to reorder projects");

            if reorder_indices.is_none() {
                println!("{}", "Reordering canceled.".bold().yellow());
                return;
            }

            let mut reordered_projects = Vec::with_capacity(storage.projects.len());
            for index in reorder_indices.unwrap() {
                reordered_projects.push(storage.projects[index].clone());
            }

            assert_eq!(reordered_projects.len(), storage.projects.len());
            storage.projects = reordered_projects;

            println!("{}", "Projects reordered successfully.".bold().blue());

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
    let mut script_path = project_state_dir();
    script_path.push("setup.sh");

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
    writeln!(file, "\n# Generated by prj\n{}", source_line)
        .expect("Failed to write to shell rc file");

    println!(
        "{} {}",
        "Setup completed.".bold().green(),
        "Please restart your terminal or run 'source ~/{shell_rc}' to activate 'pj' command."
            .italic()
            .yellow()
    );
}

fn display_projects(storage: &ProjectStorage) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    table.set_titles(Row::new(vec![
        Cell::new("Name").style_spec("Fb"),
        Cell::new("Type").style_spec("Fb"),
        Cell::new("Path").style_spec("Fb"),
    ]));

    for project in &storage.projects {
        table.add_row(Row::new(vec![
            Cell::new(&project.name).style_spec("Fb"), // Name in Bold
            Cell::new(&format!("{:?}", project.project_type)).style_spec("Fc"),
            Cell::new(&project.path.display().to_string()).style_spec("id"),
        ]));
    }

    table.printstd();
}
