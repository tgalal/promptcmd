use crate::remote_shell::{bashrc::BashRcRemoteShell, Stage3};

impl<'a> Stage3 for BashRcRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stage3(&self, functions_file: &str, _prompt_names: &[String]) -> String {

        let workdir = &self.workdir;
        format!(r#"
cat > {workdir}/{rcfile} << "EOF_STAGE3"

source {functions_file}

[[ -e ~/.bashrc ]] && source ~/.bashrc
alias bash="{bin} --rcfile {workdir}/{rcfile}"

EOF_STAGE3

exec {bin} --rcfile {workdir}/{rcfile} -i
"#,
        rcfile="bashrc",
        bin=self.bin
        )
    }
}
