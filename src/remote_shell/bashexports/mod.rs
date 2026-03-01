use crate::remote_shell::sh;

pub mod stage3;

pub struct BashExportsRemoteShell<'a> {
    bin: String,
    workdir: &'a str,
    sanitize: bool
}

impl<'a> BashExportsRemoteShell<'a> {
    pub fn new(bin: &str, workdir: &'a str, sanitize: bool) -> Self {
        Self {
            bin: bin.to_string(),
            workdir,
            sanitize
        }
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {

        let fn_name = if self.sanitize {
            &sh::sanitize_posix_function(prompt_name)
        } else {
            prompt_name
        };

        format!(r#"
{fn_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }
}
