pub mod stage3;

pub struct ZshRemoteShell<'a> {
    workdir: &'a str
}

impl<'a> ZshRemoteShell<'a> {
    pub fn new(workdir: &'a str) -> Self {
        Self {
            workdir
        }
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {
        let sanitized_function_name = prompt_name.replace("-", "_")
            .replace(".", "_");
        format!(r#"
{sanitized_function_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }
}
