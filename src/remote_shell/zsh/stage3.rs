use crate::remote_shell::{zsh::ZshRemoteShell, Stage3};

impl<'a> Stage3 for ZshRemoteShell<'a> {
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
cat > {workdir}/{zsh_env} << "EOF_STAGE3"

source {functions_file}

[[ -e ~/.zshenv ]] &&  source ~/.zshenv

EOF_STAGE3

echo "source ~/.zshrc 2> /dev/null" > {workdir}/.zshrc

ZDOTDIR={workdir} exec zsh -l
"#,
        zsh_env=".zshenv",
        )
    }
}
