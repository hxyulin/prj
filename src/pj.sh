function pj() {
    project_path=$(prj print-path "$@")

    if [ $? -eq 0 ] ; then
        cd "$project_path" || echo "Error: Failed to change directory to $project_path"
    fi
}
