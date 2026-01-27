pub fn create_function(cmd_name: &str, socket_path: &str) -> String {
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
        }} | socat -,ignoreeof UNIX-CONNECT:{socket_path}
}}"#,
        )
    }

pub fn create_functions(cmd_names: &[String], socket_path: &str) -> String {
    cmd_names
        .iter()
        .map(|cmd_name| create_function(cmd_name, socket_path))
        .collect::<Vec<_>>()
        .join("\n")
}

