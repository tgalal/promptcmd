pub mod stage3;

pub struct BashRcRemoteShell<'a> {
    bin: String,
    workdir: &'a str,
}

impl<'a> BashRcRemoteShell<'a> {
    pub fn new(bin: &str, workdir: &'a str) -> Self {
        Self {
            bin: bin.to_string(),
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
