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
