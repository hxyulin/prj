function pj() {
    local project_name="$1"
    if [ -z "$project_name" ]; then
        echo "Please provide a project name."
        return 1
    fi
    project_path=$(prj print-path "$project_name")
    if [ $? -eq 0 ]; then
        cd "$project_path" || echo "Failed to navigate to project path."
    else
        echo "Project '$project_name' not found."
    fi
}
