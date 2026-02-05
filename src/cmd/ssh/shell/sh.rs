fn create_cmd_function(cmd_name: &str, dispatcher_name: &str) -> String {
    let sanitized_function_name = cmd_name.replace("-", "_");

    format!(
        r#"{sanitized_function_name}() {{
        {dispatcher_name} {cmd_name} "$@"
}}"#,
    )
}

pub fn create_cmd_functions(cmd_names: &[String], dispatcher_name: &str) -> String {
    cmd_names
        .iter()
        .map(|cmd_name| create_cmd_function(cmd_name, dispatcher_name))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn expose(workdir: &str, functions: &str, dispatcher_func: &str, remote_cmd: Option<&[&str]>) -> String {
    let remote_cmd = if let Some(remote_cmd) = remote_cmd {
        format!("sh -c \"{}\"", remote_cmd.join(" "))
    } else {
        "exec sh -i".to_string()
    };
    format!(r#"
mkdir -p {workdir}
cat > {workdir}/{functions_file} << "EOF"
{dispatcher_func}
{functions}
EOF

ENV={workdir}/{functions_file} {remote_cmd}
"#,
    functions_file="funcs",
    )
}
