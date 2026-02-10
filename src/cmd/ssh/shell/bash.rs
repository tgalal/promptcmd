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


pub fn expose(workdir: &str,
    functions: &str,
    commands:&[String],
    dispatcher_name: &str,
    dispatcher_func: &str,
    remote_cmd: Option<&[&str]>) -> String {
    let remote_cmd = if let Some(remote_cmd) = remote_cmd {
        remote_cmd.join(" ")
    } else {
        "exec bash -l".to_string()
    };
    let mut exports = commands.iter()
        .map(|cmd| format!("export -f {cmd} 2>/dev/null || true"))
        .collect::<Vec<_>>()
        .join("\n");

    exports.push('\n');
    exports.push_str(format!("export -f {dispatcher_name} 2>/dev/null || true").as_str());
    exports.push_str(format!("export -f pcmd_exit 2>/dev/null || true").as_str());

    format!(r#"


mkdir -p {workdir}
chmod 700 {workdir}

cat > {workdir}/{functions_file} << "EOF"

{dispatcher_func}
{functions}
{exports}

pcmd_exit() {{
    pcmd_dispatch __exit__
    rm -rf {workdir}
    rm {workdir}.sock 2> /dev/null
}}

if [ ! -f "{workdir}/trap" ]; then
    touch {workdir}/trap
    trap "pcmd_exit" EXIT
    if [ -f ~/.bash_profile ]; then . ~/.bash_profile; fi
else
    if [ -f ~/.bashrc ]; then . ~/.bashrc; fi
fi
{remote_cmd}
EOF

bash {workdir}/{functions_file}
"#, functions_file="func"
    )
}
