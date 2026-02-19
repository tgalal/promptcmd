use crate::remote_shell::{bashposix::BashPosixRemoteShell, Stage3};

impl<'a> Stage3 for BashPosixRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stage3(&self, functions_file: &str, _: &[String], remote_cmd: Option<&str>) -> String {

        let workdir = &self.workdir;
        let env = "bashenv";
        let bin = &self.bin;

        if let Some(remote_cmd) = remote_cmd {
            format!(r#"exec {bin} -c ". {functions_file} && {remote_cmd}""#)
        } else {
            format!(r#"
cat > {workdir}/{env} << "EOF_STAGE3"

source {functions_file}

[[ -e ~/.bashrc ]] && source ~/.bashrc
alias bash="{bin} --posix"

EOF_STAGE3

if [ -f ~/.bash_profile ]; then . ~/.bash_profile; fi
ENV={workdir}/{env} exec {bin} --posix -l
    "#,
            )
        }

    }
}
