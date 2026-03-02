pub mod stage3;

pub struct BashExportsRemoteShell<'a> {
    bin: String,
    workdir: &'a str,
}

impl<'a> BashExportsRemoteShell<'a> {
    pub fn new(bin: &str, workdir: &'a str) -> Self {
        Self {
            bin: bin.to_string(),
            workdir,
        }
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {
        format!(r#"
{prompt_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }
}
