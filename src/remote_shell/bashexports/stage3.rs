use crate::remote_shell::{bashexports::BashExportsRemoteShell, Stage3};

impl<'a> Stage3 for BashExportsRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        prompt_names
            .iter()
            .map(|prompt_name| self.create_prompt_function(prompt_name, dispatcher_name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stage3(&self, functions_file: &str, prompt_names: &[String]) -> String {

    let exports = prompt_names.iter()
        .map(|p| format!("export -f {fn_name} 2>/dev/null || true", fn_name=self.sanitize_function_name(p)))
        .collect::<Vec<_>>()
        .join("\n");

        let workdir = &self.workdir;
        format!(r#"
cat > {workdir}/{env} << "EOF_STAGE3"
source {functions_file}

{exports}

[[ -e ~/.bashrc ]] && source ~/.bashrc

EOF_STAGE3

exec {bin} -c "source {workdir}/{env}; exec {bin} -l"
"#,
        env="bashenv",
        bin=self.bin
        )
    }
}
