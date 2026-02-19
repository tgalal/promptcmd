use crate::remote_shell::{auto::AutoRemoteShell, fish::FishRemoteShell, sh::ShRemoteShell, zsh::ZshRemoteShell, Stage3};

impl<'a> Stage3 for AutoRemoteShell<'a> {
    fn create_prompt_functions(&self, dispatcher_name: &str, prompt_names: &[String]) -> String {
        let sh = ShRemoteShell::new("sh", self.workdir);
        sh.create_prompt_functions(dispatcher_name, prompt_names)
    }

    fn stage3(&self, functions_file: &str) -> String {

        format!(r#"
SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
  zsh)
cat > {workdir}/stage3.sh << EOF_STAGE2
  {zsh_stage3}
EOF_STAGE2 ;;
  fish)
cat > {workdir}/stage3.sh << EOF_STAGE2
  {fish_stage3}
EOF_STAGE2 ;;
  sh)
cat > {workdir}/stage3.sh << EOF_STAGE2
  {sh_stage3}
EOF_STAGE2 ;;
  *) echo "Unsupported shell: $SHELL_NAME"; bash ;;
esac
"#,
        workdir=self.workdir,
        zsh_stage3=ZshRemoteShell::new(self.workdir).stage3(functions_file),
        fish_stage3=FishRemoteShell::new(self.workdir).stage3(functions_file),
        sh_stage3=ShRemoteShell::new("sh", self.workdir).stage3(functions_file)
        )
    }
}
