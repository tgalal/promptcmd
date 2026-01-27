pub fn setup(workdir: &str, functions: &str) -> String {
    format!(r#"
mkdir -p {workdir}
cat > {workdir}/{functions_file} << "EOF"
{functions}
EOF

ENV={workdir}/{functions_file} exec sh -i
"#,
    functions_file="funcs",
    )
}
