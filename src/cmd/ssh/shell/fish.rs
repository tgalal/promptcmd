fn create_cmd_function(cmd_name: &str, dispatcher_name: &str) -> String {
    format!(
        r#"function {cmd_name}
        {dispatcher_name} {cmd_name} $argv
        end
        "#,
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
        remote_cmd.join(" ")
    } else {
        "".to_string()
    };
    let dispatcher_func = dispatcher_func.replace("'", "'\\''");
    let functions = functions.replace("'", "'\\''");
    format!(r#"
mkdir -p {workdir}
chmod 700 {workdir}

cat > {workdir}/{functions_file} << "EOF"
{dispatcher_func}
{functions}
function fish
    command fish -C "source {workdir}/{functions_file}"
end

function pcmd_exit
    rm -rf {workdir}
    rm {workdir}.sock 2> /dev/null
end

if not test -f "{workdir}/trap"
    touch {workdir}/trap
    trap "rm -rf {workdir}; rm -rf {workdir}.sock" EXIT
end
EOF

exec fish -l -C "source {workdir}/{functions_file};{remote_cmd}"
"#,
    functions_file="funcs",
    )
}
