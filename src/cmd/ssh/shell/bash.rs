fn create_cmd_function(cmd_name: &str, dispatcher_name: &str) -> String {
    format!(
        r#"{cmd_name}() {{
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


pub fn expose(dispatcher_name: &str, commands: &[String]) -> String {
    let mut exports = commands.iter()
        .map(|cmd| format!("export -f {cmd} 2>/dev/null || true"))
        .collect::<Vec<_>>()
        .join("\n");

    exports.push('\n');
    exports.push_str(format!("export -f {dispatcher_name} 2>/dev/null || true").as_str());

    format!(r#"
{exports}
exec bash -l
"#,
    )
}
