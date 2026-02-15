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
    local result
    result="$(\command prj "$@" 2>/dev/tty)"
    if [[ -n "$result" ]]; then
        \builtin cd -- "$result"
    fi
}}
"#
    )
}

fn generate_bash(cmd: &str) -> String {
    format!(
        r#"function {cmd}() {{
    local result
    result="$(\command prj "$@" 2>/dev/tty)"
    if [[ -n "$result" ]]; then
        \builtin cd -- "$result"
    fi
}}
"#
    )
}

fn generate_powershell(cmd: &str) -> String {
    format!(
        r#"function {cmd} {{
    $result = & prj @args 2>$null
    if ($result) {{
        Set-Location -Path $result
    }}
}}
"#
    )
}
