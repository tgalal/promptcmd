pub fn expose(workdir: &str, functions: &str, dispatcher_func: &str) -> String {
    format!(r#"
mkdir -p {workdir}
cat > {workdir}/{functions_file} << "EOF"
{dispatcher_func}
{functions}
EOF

ENV={workdir}/{functions_file} exec sh -i
"#,
    functions_file="funcs",
    )
}
