pub fn generate_ssh_bootstrap_command(registry: &Vec<String>, port: u32) -> String {
    let script = generate_bootstrap_script(registry, port);
    // debug!("bootstrap script: {script}");

    format!(
        "{}\nexec bash -l",
        script
    )
}
/// Generate a POSIX-compliant bootstrap script that deploys shell functions
pub fn generate_bootstrap_script(registry: &Vec<String>, port: u32) -> String {
    let mut script = String::new();

    // Generate a function for each registered command
    for cmd in registry {
        let function = generate_shell_function(cmd, port);
        script.push_str(&function);
        script.push_str("\n\n");
    }

    // Export all functions
    for cmd in registry {
        script.push_str(&format!("export -f {} 2>/dev/null || true\n", cmd));
    }

    script
}

fn generate_shell_function(cmd_name: &str, port: u32) -> String {
    format!(
        r#"{cmd_name}() {{
        {{
        printf '%s ' "{cmd_name}"
        for arg in "$@"; do
            printf '"%s" ' "$arg"
        done
        printf "\0"
        # Only read stdin if it's not a terminal (i.e., piped data)
        if [ ! -t 0 ]; then
            cat
        fi
        printf "\0"
        }} | nc localhost {port}
}}"#,
        cmd_name = cmd_name,
        port = port
    )
}
