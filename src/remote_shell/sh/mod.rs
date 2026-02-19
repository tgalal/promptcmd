use crate::{config::appconfig::BashMethod, remote_shell::{bashexports::BashExportsRemoteShell,
    bashposix::BashPosixRemoteShell, bashrc::BashRcRemoteShell, fish::FishRemoteShell,
    zsh::ZshRemoteShell, Stage1, Stage2, Stage3, Shell, Channel}};

pub mod stage1;
pub mod stage2;
pub mod stage3;

pub struct ShRemoteShell<'a> {
    sh_bin: String,
    workdir: &'a str,
}

impl<'a> ShRemoteShell<'a > {
    pub fn bootstrap(
        sh_bin: &str,
        workdir: &'a str,
        dispatcher_name: &str,
        prompt_names: &[String],
        channel: &Channel,
        shell: &Shell,
        bash_method: &BashMethod
    ) -> String {
        let sh = Self {
            sh_bin: sh_bin.to_string(),
            workdir
        };
        let stage2 = sh.stage2(dispatcher_name, prompt_names, channel, shell, bash_method);
        sh.stage1(&stage2)
    }

    fn escape(&self, code: &str) -> String {
        code.replace("$", "\\$")
    }

    pub fn create_autoshell(&self,
        functions_file: &str,
        dispatcher_name: &str,
        prompts_names: &[String],
        bash_method: &BashMethod
    ) -> String {

        let fish = FishRemoteShell::new(self.workdir);

        let ash = ShRemoteShell::new_ash(self.workdir).stage3(functions_file, prompts_names);
        let zsh= ZshRemoteShell::new(self.workdir).stage3(functions_file, prompts_names);
        let dash = ShRemoteShell::new_dash(self.workdir).stage3(functions_file, prompts_names);
        let bash = match bash_method {
            BashMethod::Posix => BashPosixRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names),
            BashMethod::Rc => BashRcRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names),
            BashMethod::Exports => BashExportsRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names),
        };

        let fish_functions = fish.create_prompt_functions(dispatcher_name, prompts_names);
        let stage3_file = format!("{workdir}/stage3.sh", workdir=self.workdir);
        let posix_functions = self.create_prompt_functions(dispatcher_name, prompts_names);

        format!(r#"
SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
  bash)
cat > {stage3_file} << EOF_STAGE2
  {bash}
EOF_STAGE2
;;
  zsh)
cat > {stage3_file} << EOF_STAGE2
  {zsh}
EOF_STAGE2
;;
  fish)
cat > {stage3_file} << EOF_STAGE2
  {fish_stage3}
EOF_STAGE2
;;
  sh)
cat > {stage3_file} << EOF_STAGE2
  {sh_stage3}
EOF_STAGE2
;;
  ash)
cat > {stage3_file} << EOF_STAGE2
  {ash}
EOF_STAGE2
;;
  dash)
cat > {stage3_file} << EOF_STAGE2
  {dash}
EOF_STAGE2
;;
  *) echo "Unsupported shell: $SHELL_NAME";
;;
esac

case "$SHELL_NAME" in
  zsh | sh | bash | ash | dash)
cat > {functions_file} << EOF_STAGE2
  {posix_functions}
EOF_STAGE2
;;
  fish)
cat > {functions_file} << EOF_STAGE2
  {fish_functions}
EOF_STAGE2
;;
  *) echo "Unsupported shell: $SHELL_NAME"
;;
esac
"#,
        fish_stage3=self.escape(&fish.stage3(functions_file, prompts_names)),
        sh_stage3=self.escape(&self.stage3(functions_file, prompts_names)),
        fish_functions=self.escape(&fish_functions),
        posix_functions=self.escape(&posix_functions)
        )

    }

    pub fn new(sh_bin: &str, workdir: &'a str) -> Self {
        Self {
            sh_bin: sh_bin.to_string(),
            workdir
        }
    }

    pub fn new_sh(workdir: &'a str) -> Self {
        Self::new("sh", workdir)
    }

    pub fn new_ash(workdir: &'a str) -> Self {
        Self::new("ash", workdir)
    }

    pub fn new_dash(workdir: &'a str) -> Self {
        Self::new("dash", workdir)
    }

    fn create_prompt_function(&self, prompt_name: &str, dispatcher_name: &str) -> String {
        let sanitized_function_name = prompt_name.replace("-", "_")
            .replace(".", "_");
        format!(r#"
{sanitized_function_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }

}
