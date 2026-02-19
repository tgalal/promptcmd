use crate::remote_shell::{sh::ShRemoteShell, Stage3};

impl<'a> Stage3 for ShRemoteShell<'a> {
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
cat > {workdir}/{sh_env} << "EOF_STAGE3"

. {functions_file}

if [ -f "$HOME/.shrc" ]; then
    . $HOME/.shrc
fi

EOF_STAGE3

{sh_bin} -l -c "ENV={workdir}/{sh_env} exec {sh_bin} -i"
"#,
        sh_bin=self.sh_bin,
        sh_env="sh_env",
        )
    }
}
