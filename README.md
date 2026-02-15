# prj

A local project manager for the command line.

`prj` keeps a database of your development projects and lets you fuzzy-search, navigate, tag, inspect, clean, and synchronize them from a single tool.

## Features

- **Fuzzy picker** — interactive TUI to search and jump to any project
- **Project list** — sortable, scrollable TUI table with VCS/build info
- **Auto-detection** — recognizes Git repos and 10 build systems by their marker files
- **Recursive scan** — discover projects under a directory tree in one command
- **Git clone + register** — `prj new --git <url>` clones and adds in one step
- **Stats** — lines of code (via tokei), disk usage, and artifact size per project or across all
- **Git status dashboard** — see branch, dirty state, ahead/behind for every project at once
- **Tags** — organize projects with arbitrary labels, then filter by tag
- **Clean** — delete build artifacts (`target/`, `node_modules/`, etc.) with a dry-run preview
- **Run** — execute a shell command across projects filtered by name, tag, or `--all`
- **Export / Import** — share a project manifest (with git remote URLs) to replicate a workspace
- **Shell integration** — a thin shell function that `cd`s into the selected project
- **GC** — prune projects whose paths no longer exist on disk

## Installation

Build and install from source:

```sh
cargo install --path .
```

### Shell integration

Add the init script to your shell config so that selecting a project changes your working directory:

**Zsh** (`~/.zshrc`):
```sh
eval "$(prj init zsh)"
```

**Bash** (`~/.bashrc`):
```sh
eval "$(prj init bash)"
```

**PowerShell** (`$PROFILE`):
```powershell
Invoke-Expression (prj init powershell)
```

This creates a shell function called `prjp` (customizable with `--cmd`). Use `prjp` instead of `prj` when you want `cd` behavior.

## Quick Start

```sh
# Scan ~/dev for projects
prj scan ~/dev

# Open the fuzzy picker (via the shell wrapper)
prjp

# Tag a project
prj tag my-app work rust

# See stats across all projects
prj stats

# Check git status everywhere
prj status
```

## Commands

### `prj` (no subcommand)

Opens the fuzzy finder picker. Prints the selected project path to stdout (the shell wrapper `cd`s to it).

### `prj add [PATH] [--name NAME]`

Register a project. Defaults to the current directory. Auto-detects VCS, build system, and artifact directories.

### `prj scan <DIR> [--depth N]`

Recursively discover and register projects under `DIR`. Default depth is 3. Skips artifact directories and already-registered projects.

### `prj new --git "<CLONE_ARGS>"`

Run `git clone` with the given arguments, then auto-register the cloned repo.

```sh
prj new --git "https://github.com/user/repo"
prj new --git "git@github.com:user/repo.git my-folder"
```

### `prj remove <PROJECT>`

Unregister a project by name. Does **not** delete any files.

### `prj list [--plain] [--tag TAG]`

Show registered projects. Without `--plain`, opens a TUI table. With `--plain` (or when piped), outputs tab-separated text. Use `--tag` to filter.

### `prj stats [PROJECT] [--json]`

Show statistics for a single project or an overview of all projects. Includes lines of code, disk usage, and artifact sizes. Pass `--json` for machine-readable output.

### `prj status [--json]`

Git status dashboard across all projects. Shows branch, dirty/clean state, changed/staged/untracked counts, and ahead/behind. Pass `--json` for machine-readable output.

### `prj tag <PROJECT> <TAGS...>`

Add one or more tags to a project.

### `prj untag <PROJECT> <TAGS...>`

Remove tags from a project.

### `prj clean [PROJECT] [--all] [--dry-run]`

Delete detected artifact directories. Target a single project by name or use `--all`. Always use `--dry-run` first to preview what would be deleted.

### `prj gc [--dry-run]`

Remove projects whose paths no longer exist on disk. Prompts for confirmation unless `--dry-run` is used.

### `prj run <CMD> [--project NAME] [--tag TAG] [--all]`

Execute a shell command in the directory of matching projects.

```sh
prj run "git pull" --all
prj run "cargo test" --tag rust
prj run "npm install" --project my-app
```

### `prj export [--output FILE] [--base-dir DIR]`

Export all projects to a TOML manifest. Includes git remote URLs and tags. Outputs to stdout unless `--output` is given.

### `prj import <FILE> [--base-dir DIR]`

Import a manifest. Clones missing projects using their remote URLs and registers them with their original tags.

### `prj init <SHELL> [--cmd NAME]`

Print the shell init script. Supported shells: `zsh`, `bash`, `powershell`. The generated function defaults to `prjp` but can be changed with `--cmd`.

## Shell Integration

The `prj` binary writes paths to stdout and UI to stderr. The shell wrapper function captures stdout and `cd`s into it:

```sh
eval "$(prj init zsh)"         # creates `prjp`
eval "$(prj init zsh --cmd p)" # creates `p` instead
```

Supported shells: **zsh**, **bash**, **powershell**.

## Configuration

Config file location: `~/.config/prj/config.toml`

```toml
# Name of the shell function created by `prj init`
shell_cmd = "prjp"

# Maximum directory depth for `prj scan`
scan_depth = 3

# Override the default database location
# database_path = "/path/to/projects.toml"
```

| Option          | Default                            | Description                                  |
|-----------------|------------------------------------|----------------------------------------------|
| `shell_cmd`     | `"prjp"`                           | Shell function name generated by `prj init`  |
| `scan_depth`    | `3`                                | Default max depth for `prj scan`             |
| `database_path` | `~/.local/share/prj/projects.toml` | Path to the project database file            |

## Detected Build Systems

| Build System | Marker File(s)                    | Artifact Directories             |
|--------------|-----------------------------------|----------------------------------|
| Cargo        | `Cargo.toml`                      | `target`                         |
| npm          | `package.json`                    | `node_modules`, `dist`, `build`  |
| CMake        | `CMakeLists.txt`                  | `build`                          |
| Go           | `go.mod`                          | (none)                           |
| Python       | `pyproject.toml`                  | `__pycache__`, `.venv`, `dist`   |
| Zig          | `build.zig`                       | `zig-out`, `zig-cache`           |
| Make         | `Makefile`                        | (none)                           |
| Gradle       | `build.gradle` / `build.gradle.kts` | `build`, `.gradle`            |
| Maven        | `pom.xml`                         | `target`                         |
| Meson        | `meson.build`                     | `builddir`                       |

## License

[MIT](LICENSE)
