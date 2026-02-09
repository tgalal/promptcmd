fn create_cmd_function(cmd_name: &str, dispatcher_name: &str) -> String {
    let sanitized_function_name = cmd_name.replace("-", "_")
        .replace(".", "_");

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
        format!("dash -c \"{}\"", remote_cmd.join(" "))
    } else {
        "exec dash -i -l".to_string()
    };
    let dispatcher_func = dispatcher_func.replace("'", "'\\''");
    let functions = functions.replace("'", "'\\''");

    format!(r#"
mkdir -p {workdir}
chmod 700 {workdir}

cat > {workdir}/{functions_file} << "EOF"
{dispatcher_func}
{functions}

pcmd_exit() {{
    rm -rf {workdir}
    rm {workdir}.sock 2> /dev/null
}}

if [ ! -f "{workdir}/trap" ]; then
    touch {workdir}/trap
    trap "pcmd_exit" EXIT
fi
EOF

ENV={workdir}/{functions_file} {remote_cmd}
"#,
    functions_file="funcs",
    )
}
