use crate::remote_shell::{bashexports::BashExportsRemoteShell, Stage3};

impl<'a> Stage3 for BashExportsRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stage3(&self, functions_file: &str, prompt_names: &[String], remote_cmd: Option<&str>) -> String {
        let bin = &self.bin;
        let env = "bashenv";

        if let Some(remote_cmd) = remote_cmd {
            format!(r#"exec {bin} -c ". {functions_file} && {remote_cmd}""#)
        } else {
            let exports = prompt_names.iter()
                .map(|p| format!("export -f {fn_name} 2>/dev/null || true", fn_name=self.sanitize_function_name(p)))
                .collect::<Vec<_>>()
                .join("\n");

            let workdir = &self.workdir;
            format!(r#"
OLD_UMASK=$(umask); umask 077
cat > {workdir}/{env} << "EOF_STAGE3"
source {functions_file}

{exports}

[[ -e $HOME/.bashrc ]] && source $HOME/.bashrc

EOF_STAGE3

umask $OLD_UMASK
exec {bin} -c "source {workdir}/{env}; exec {bin} -l"
    "#,
            )
        }
    }
}
