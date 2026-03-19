use color_eyre::eyre::bail;

pub fn generate_init(shell: &str, cmd: &str) -> color_eyre::Result<String> {
    match shell {
        "zsh" => Ok(generate_zsh(cmd)),
        "bash" => Ok(generate_bash(cmd)),
        "powershell" | "pwsh" => Ok(generate_powershell(cmd)),
        _ => bail!("unsupported shell: {shell} (supported: zsh, bash, powershell)"),
    }
}

fn generate_zsh(cmd: &str) -> String {
    format!(
        r#"function {cmd}() {{
    if [[ $# -eq 0 ]] || [[ "$1" == "list" ]]; then
        local result
        result="$(\command prj "$@" 2>/dev/tty)"
        if [[ -n "$result" ]]; then
            \builtin cd -- "$result"
        fi
    else
        \command prj "$@"
    fi
}}
"#
    )
}

fn generate_bash(cmd: &str) -> String {
    format!(
        r#"function {cmd}() {{
    if [[ $# -eq 0 ]] || [[ "$1" == "list" ]]; then
        local result
        result="$(\command prj "$@" 2>/dev/tty)"
        if [[ -n "$result" ]]; then
            \builtin cd -- "$result"
        fi
    else
        \command prj "$@"
    fi
}}
"#
    )
}

fn generate_powershell(cmd: &str) -> String {
    format!(
        r#"function {cmd} {{
    if ($args.Count -eq 0 -or $args[0] -eq 'list') {{
        $result = & prj @args 2>$null
        if ($result) {{
            Set-Location -Path $result
        }}
    }} else {{
        & prj @args
    }}
}}
"#
    )
}
