pub fn setup(commands: &[String], functions: &str) -> String {
    let exports = commands.iter()
        .map(|cmd| format!("export -f {cmd} 2>/dev/null || true"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"
{functions}
{exports}
exec bash -l
"#,
    )
}
