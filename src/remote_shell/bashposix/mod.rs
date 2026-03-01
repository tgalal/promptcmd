use crate::remote_shell::sh;

pub mod stage3;

pub struct BashPosixRemoteShell<'a> {
    bin: String,
    workdir: &'a str,
}

impl<'a> BashPosixRemoteShell<'a> {
    pub fn new(bin: &str, workdir: &'a str) -> Self {
        Self {
            bin: bin.to_string(),
            workdir
        }
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {
        let sanitized_function_name = sh::sanitize_posix_function(prompt_name);
        format!(r#"
{sanitized_function_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }
}
