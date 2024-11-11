# `prj` - Project Manager CLI

`prj` is a simple CLI tool for managing projects on your local machine. It allows you to add, list, remove, navigate to, and clone projects directly from GitHub. With built-in support for various project types (like Cargo and CMake), `prj` is designed to streamline your workflow.

## Features

- **Add Projects**: Easily add projects with a name, type, and path.
- **List Projects**: View all added projects in a formatted, colored table.
- **Navigate to Projects**: Quickly retrieve project paths for easy navigation.
- **Remove Projects**: Remove projects by name or path.
- **Clone Repositories**: Clone GitHub repositories and automatically add them as projects.
- **Setup Shortcut**: Setup a `pj` shortcut for quick project navigation.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/prj
   cd prj
   ```

2. Build and install:
   ```bash
   cargo install --path .
   ```

3. Run the setup to configure the `pj` shortcut:
   ```bash
   prj setup
   ```

   This will add a `pj` function to your shell (like `.bashrc` or `.zshrc`) for easier navigation. To activate it immediately, source your shell configuration file:

   ```bash
   source ~/.bashrc   # or source ~/.zshrc
   ```

## Usage

```bash
prj <COMMAND> [OPTIONS]
```

### Commands

#### `add`
Manually add a project with a specified name, type, and path.
```bash
prj add --name myproject --type Cargo --path /path/to/project
```
- **--name**: Project name (optional, will prompt if missing)
- **--type**: Project type (`Cargo`, `CMake`, etc.)
- **--path**: Path to the project (defaults to current directory)

#### `list`
List all added projects in a formatted table.
```bash
prj list
```

#### `print-path`
Print the path of a specified project.
```bash
prj print-path --name myproject
```
- **--name**: Name of the project

#### `remove`
Remove a project by its name or path.
```bash
prj remove --name myproject
```
- **--name**: Name of the project (optional)
- **--path**: Path of the project (optional)

#### `clone`
Clone a GitHub repository and automatically add it as a project.
```bash
prj clone https://github.com/author/repo --type Cargo --name customname --path /custom/path -- --branch main --depth 1
```
- **repo_url**: URL of the GitHub repository to clone
- **--type**: Project type (optional, defaults to `Other`)
- **--name**: Custom name for the project (defaults to repository name)
- **--path**: Custom path for the project (defaults to current directory)
- **git_options**: Additional options for the `git clone` command, such as `--branch` and `--depth`.

#### `setup`
Sets up the project manager and adds a `pj` shell function for quick project navigation. Use this once after installation.

### Example Workflow

1. **Add a Project**:
   ```bash
   prj add --name myproject --type Cargo --path /home/user/myproject
   ```

2. **List Projects**:
   ```bash
   prj list
   ```

3. **Clone a Repository**:
   ```bash
   prj clone https://github.com/author/repo -- --branch main --depth 1
   ```

4. **Navigate to a Project with `pj`**:
   ```bash
   pj myproject
   ```

   This will `cd` into the project directory if you sourced your shell configuration after running `prj setup`.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
