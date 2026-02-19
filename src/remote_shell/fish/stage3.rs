use crate::remote_shell::{fish::FishRemoteShell, Stage3};

impl<'a> Stage3 for FishRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn stage3(&self, functions_file: &str, _prompt_names: &[String]) -> String {

        let workdir = &self.workdir;
        // let functions = self.create_prompt_functions(prompts_names, dispatcher_name);

        format!(r#"
cat > {workdir}/{fish_env} << "EOF_STAGE3"

source {functions_file}

function fish
    command fish -C "source {workdir}/{fish_env}"
end

EOF_STAGE3

exec fish -l -C "source {workdir}/{fish_env}"
"#,
        fish_env="fish_env",
        )
    }
}
