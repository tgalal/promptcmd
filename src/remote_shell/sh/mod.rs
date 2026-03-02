use crate::{config::appconfig::BashMethod, remote_shell::{bashexports::BashExportsRemoteShell,
    bashposix::BashPosixRemoteShell, bashrc::BashRcRemoteShell, fish::FishRemoteShell,
    zsh::ZshRemoteShell, Stage1, Stage2, Stage3, Shell, Channel}};

pub mod stage1;
pub mod stage2;
pub mod stage3;
pub mod motd;

pub struct ShRemoteShell<'a> {
    sh_bin: String,
    workdir: &'a str,
}

pub fn sanitize_posix_function(name: &str) -> String {
    name.replace("-", "_")
        .replace(".", "_")
}

/// Returns `true` if the given name is a valid POSIX-compliant shell function name.
///
/// Per POSIX, a function name must be a valid "Name":
/// - Starts with a letter or underscore
/// - Followed by zero or more letters, digits, or underscores
/// - Must not be a POSIX shell reserved word
pub fn is_posix_function_name(name: &str) -> bool {
    let mut chars = name.chars();

    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }

    !matches!(
        name,
        "if" | "then"
            | "else"
            | "elif"
            | "fi"
            | "do"
            | "done"
            | "case"
            | "esac"
            | "while"
            | "until"
            | "for"
            | "in"
    )
}

impl<'a> ShRemoteShell<'a > {
    pub fn bootstrap(
        sh_bin: &str,
        workdir: &'a str,
        dispatcher_name: &str,
        prompt_names: &[String],
        channel: &Channel,
        shell: &Shell,
        bash_method: &BashMethod,
        remote_cmd: Option<&str>,
        session_pwd: &str,
        motd: bool
    ) -> String {
        let sh = Self {
            sh_bin: sh_bin.to_string(),
            workdir
        };
        let stage2 = sh.stage2(dispatcher_name, prompt_names, channel, shell,
            bash_method, remote_cmd, session_pwd, motd);
        sh.stage1(&stage2)
    }

    fn escape(&self, code: &str) -> String {
        code.replace("$", "\\$")
    }

    pub fn create_autoshell(&self,
        functions_file: &str,
        dispatcher_name: &str,
        prompts_names: &[String],
        bash_method: &BashMethod,
        remote_cmd: Option<&str>
    ) -> String {

        let fish = FishRemoteShell::new(self.workdir);

        let ash_stage3 = ShRemoteShell::new_ash(self.workdir).stage3(functions_file, prompts_names, remote_cmd);
        let zsh= ZshRemoteShell::new(self.workdir);
        let zsh_stage3 = zsh.stage3(functions_file, prompts_names, remote_cmd);
        let dash_stage3 = ShRemoteShell::new_dash(self.workdir).stage3(functions_file, prompts_names, remote_cmd);
        let bash_stage3 = match bash_method {
            BashMethod::Posix => BashPosixRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names, remote_cmd),
            BashMethod::Rc => BashRcRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names, remote_cmd),
            BashMethod::Exports => BashExportsRemoteShell::new("bash", self.workdir).stage3(functions_file, prompts_names, remote_cmd),
        };

        let fish_functions = fish.create_prompt_functions(dispatcher_name, prompts_names);
        let stage3_file = format!("{workdir}/stage3.sh", workdir=self.workdir);
        let posix_functions = self.create_prompt_functions(dispatcher_name, prompts_names);
        let nonposix_promptnames = prompts_names
            .iter()
            .filter(|name| !is_posix_function_name(name))
            .cloned()
            .collect::<Vec<_>>();
        let nonposix_functions = zsh.create_prompt_functions(dispatcher_name, &nonposix_promptnames);

        format!(r#"
SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
  bash)
cat > {stage3_file} << EOF_STAGE2
  {bash_stage3}
EOF_STAGE2
;;
  zsh)
cat > {stage3_file} << EOF_STAGE2
  {zsh_stage3}
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
  {ash_stage3}
EOF_STAGE2
;;
  dash)
cat > {stage3_file} << EOF_STAGE2
  {dash_stage3}
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
    case "$SHELL_NAME" in
        bash | zsh)
        cat >> {functions_file} << EOF_STAGE2
  {nonposix_functions}
EOF_STAGE2
        ;;
        *);;
    esac
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
        fish_stage3=self.escape(&fish.stage3(functions_file, prompts_names, remote_cmd)),
        sh_stage3=self.escape(&self.stage3(functions_file, prompts_names, remote_cmd)),
        fish_functions=self.escape(&fish_functions),
        posix_functions=self.escape(&posix_functions),
        nonposix_functions=self.escape(&nonposix_functions)
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
        let sanitized_function_name = sanitize_posix_function(prompt_name);
        format!(r#"
{sanitized_function_name}() {{
    {dispatcher_name} {prompt_name} "$@"
}}"#,
        )
    }

}
