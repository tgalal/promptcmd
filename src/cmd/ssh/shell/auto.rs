use crate::cmd::ssh::shell::{bash, sh, zsh};

pub fn setup(workdir: &str, commands: &[String], functions: &str) -> String {

    let bash_setup = bash::setup(commands, functions);
    let zsh_setup = zsh::setup(workdir, functions);
    let sh_setup = sh::setup(workdir, functions);

    format!(r#"
SHELL_NAME=$(basename "$SHELL")
case "$SHELL_NAME" in
  bash) {bash_setup};;
  zsh) {zsh_setup} ;;
  sh|dash|ash) {sh_setup} ;;
  *) echo "Unsupported shell: $SHELL_NAME"; bash ;;
esac
"#,
    )
}
