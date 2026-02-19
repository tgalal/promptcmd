pub mod stage3;

pub struct BashExportsRemoteShell<'a> {
    bin: String,
    workdir: &'a str,
}

impl<'a> BashExportsRemoteShell<'a> {
    pub fn new(bin: &str, workdir: &'a str) -> Self {
        Self {
            bin: bin.to_string(),
            workdir
        }
    }

    fn sanitize_function_name(&self, fn_name: &str) -> String {
        fn_name.replace(".", "_")
               .replace("-", "_")
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {
        let sanitized_function_name = self.sanitize_function_name(prompt_name);

        format!(r#"
{sanitized_function_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }
}
