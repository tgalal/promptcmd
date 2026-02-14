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
        format!("sh -c \"{}\"", remote_cmd.join(" "))
    } else {
        "exec sh -l".to_string()
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
    pcmd_dispatch __exit__
    rm -rf {workdir}
    rm {workdir}.sock 2> /dev/null
}}

if [ ! -f "{workdir}/trap" ]; then
    touch {workdir}/trap
    trap "pcmd_exit" EXIT
fi

# cat /etc/motd 2>/dev/null

if [ -f "$HOME/.shrc" ]; then
    . $HOME/.shrc
fi

EOF

{remote_cmd} -c "ENV={workdir}/{functions_file} exec sh -i"
"#,
    functions_file="funcs",
    )
}
