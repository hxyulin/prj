function pj() {
    project_path=$(prj print-path "$@" | tail -n 1)

    if [ $? -eq 0 ] && [ -n "$project_path" ]; then
        cd "$project_path" || echo "Error: Failed to change directory to $project_path"
    else
        echo "Error: $project_path"
    fi
}
