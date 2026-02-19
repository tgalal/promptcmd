use crate::remote_shell::{fish::FishRemoteShell, Stage3};

impl<'a> Stage3 for FishRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn stage3(&self, functions_file: &str, _prompt_names: &[String], remote_cmd: Option<&str>) -> String {

        let workdir = &self.workdir;
        let fish_env = "fish_env";
        if let Some(remote_cmd) = remote_cmd {
            format!(r#"exec fish -C "source {functions_file};{remote_cmd}""#)
        } else {
            format!(r#"
cat > {workdir}/{fish_env} << "EOF_STAGE3"

source {functions_file}

function fish
    command fish -C "source {workdir}/{fish_env}"
end

EOF_STAGE3

exec fish -l -C "source {workdir}/{fish_env}"
"#,
            )
        }

    }
}
