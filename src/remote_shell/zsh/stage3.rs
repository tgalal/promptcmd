use crate::remote_shell::{zsh::ZshRemoteShell, Stage3};

impl<'a> Stage3 for ZshRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn stage3(&self, functions_file: &str, _prompt_names: &[String], remote_cmd: Option<&str>) -> String {

        let workdir = &self.workdir;
        let zsh_env = ".zshenv";
        if let Some(remote_cmd) = remote_cmd {
            format!(r#"exec zsh -c ". {functions_file} && {remote_cmd}""#)
        } else {
            format!(r#"
cat > {workdir}/{zsh_env} << "EOF_STAGE3"

source {functions_file}

[[ -e ~/.zshenv ]] &&  source ~/.zshenv

EOF_STAGE3

echo "source ~/.zshrc 2> /dev/null" > {workdir}/.zshrc

ZDOTDIR={workdir} exec zsh -l
"#
            )
        }
    }
}
