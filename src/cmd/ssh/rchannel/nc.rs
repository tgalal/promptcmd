pub fn create_function(cmd_name: &str, port: u32) -> String {
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
    )
}

pub fn create_functions(cmd_names: &[String], port: u32) -> String {
    cmd_names
        .iter()
        .map(|cmd_name| create_function(cmd_name, port))
        .collect::<Vec<_>>()
        .join("\n")
}

