pub mod zsh;
pub mod bash;
pub mod sh;
pub mod ash;
pub mod dash;
pub mod fish;
pub mod bashrc;
pub mod bashposix;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("{0}")]
    UnsupportedMode(String)
}

pub enum Shell {
    Bash(String),
    Zsh(String),
    Sh(String),
    Ash(String),
    Dash(String),
    Fish(String),
    Auto(String)
}

pub enum Channel {
    // local_port, remote_port
    Nc(u32, u32),
    // local_socket, remtoe_socket
    Socat(String, String),
    // local_port, remote_port
    BashTcp(u32, u32),

}

struct ChannelCode {
    pre: Option<String>,
    post: String
}

impl ChannelCode {
    pub fn from_post(post: String) -> Self {
        Self {
            pre: None,
            post
        }
    }
}

impl Channel {
    fn build(&self, shell: &Shell) -> Result<ChannelCode, ShellError> {
        match self {
            Channel::Nc(_, remote_port) => Ok(self.build_nc(*remote_port, shell)),
            Channel::Socat(_, remote_socket) => Ok(self.build_socat(remote_socket, shell)),
            Channel::BashTcp(_, remote_port) => self.build_bashtcp(*remote_port, shell),
        }
    }

    fn build_nc(&self, port: u32, shell: &Shell) -> ChannelCode {
        match shell {
            Shell::Auto(_) |
            Shell::Bash(_) |
            Shell::Zsh(_) |
            Shell::Sh(_) |
            Shell::Ash(_) |
            Shell::Dash(_) |
            Shell::Fish(_) => ChannelCode::from_post(format!("| nc localhost {port}")),
        }
    }

    fn build_bashtcp(&self, port: u32, shell: &Shell) -> Result<ChannelCode, ShellError> {
        match shell {
            Shell::Bash(_) => {
                Ok(ChannelCode {
                    pre: Some(format!("exec 3<>/dev/tcp/localhost/{port}")),
                    post: r#">&3
cat <&3
exec 3<&-
exec 3>&-"#.to_string()
                })

            },
            Shell::Zsh(_) => {
                Err(ShellError::UnsupportedMode( "Cannot use bashtcp mode with zsh".to_string()))
            },
            Shell::Sh(_) | Shell::Ash(_) | Shell::Dash(_) => {
                Err(ShellError::UnsupportedMode( "Cannot use bashtcp mode with sh".to_string()))
            },
            Shell::Fish(_) => {
                Err(ShellError::UnsupportedMode( "Cannot use bashtcp mode with fish shell".to_string()))
            },
            Shell::Auto(_) => {
                Err(ShellError::UnsupportedMode( "Cannot use bashtcp mode in auto mode".to_string()))
            }
        }
    }

    fn build_socat(&self, socket_path: &str, shell: &Shell) -> ChannelCode {
        match shell {
            Shell::Auto(_) |
            Shell::Bash(_) |
            Shell::Zsh(_) |
            Shell::Sh(_) |
            Shell::Ash(_) |
            Shell::Dash(_) |
            Shell::Fish(_) => ChannelCode::from_post(format!("| socat -,ignoreeof UNIX-CONNECT:{socket_path} ")),
        }
    }
}

impl Shell {

    pub fn build(&self, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        match self  {
            // Shell::Bash => self.build_bash(channel, prompts, remote_cmd),
            Shell::Bash(workdir) => self.build_bashposix(workdir, channel, prompts, remote_cmd),
            Shell::Zsh(workdir) => self.build_zsh(workdir, channel, prompts, remote_cmd),
            Shell::Sh(workdir) => self.build_sh(workdir, channel, prompts, remote_cmd),
            Shell::Ash(workdir) => self.build_ash(workdir, channel, prompts, remote_cmd),
            Shell::Dash(workdir) => self.build_dash(workdir, channel, prompts, remote_cmd),
            Shell::Auto(workdir) => self.build_auto(workdir, channel, prompts, remote_cmd),
            Shell::Fish(workdir) => self.build_fish(workdir, channel, prompts, remote_cmd)
    }
    }

    fn build_dispatcher_func(&self, funcname: &str,  channel: &Channel) -> Result<String, ShellError> {
        let channel_code = channel.build(self)?;

        let res = match self {
            Shell::Auto(_) | Shell::Bash(_) | Shell::Zsh(_) | Shell::Sh(_) | Shell::Ash(_) | Shell::Dash(_) => {
                format!(
                    r#"{funcname}() {{
                    {pre}
                    {{
                    printf '%s ' "$1"
                    shift
                    for arg in "$@"; do
                        printf '"%s" ' "$arg"
                    done
                    printf "\0"
                    if [ ! -t 0 ]; then
                        cat
                    fi
                    printf "\0"
                    }}{post}
            }}"#, pre=channel_code.pre.unwrap_or("".to_string()), post=&channel_code.post )
            },
            Shell::Fish(_) => {
                format!(
                r#"function {funcname}
                    begin
                        printf '%s ' $argv[1]
                        set -e argv[1]
                        for arg in $argv
                            printf '"%s" ' $arg
                        end
                        printf "\0"
                        if not isatty stdin
                            cat
                        end
                        printf "\0"
                    end {post}
                end
                "#, post=&channel_code.post)
            }
        };
        Ok(res)
    }

    fn build_auto(&self, workdir: &str, channel: &Channel, prompts: &[String], remte_cmd: Option<&[&str]>) -> Result<String, ShellError> {

        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;

        let bash_expose = bashposix::expose(workdir, &prompt_functions, &dispatcher_func, remte_cmd);
        let zsh_expose = zsh::expose(workdir, &prompt_functions, &dispatcher_func, remte_cmd);
        let sh_expose  = sh::expose(workdir, &prompt_functions, &dispatcher_func, remte_cmd);

        Ok(format!(r#"
{prompt_functions}
{dispatcher_func}
SHELL_NAME=$(basename "$SHELL")
case "$SHELL_NAME" in
  bash) {bash_expose};;
  zsh) {zsh_expose} ;;
  sh|dash|ash) {sh_expose} ;;
  *) echo "Unsupported shell: $SHELL_NAME"; bash ;;
esac
"#,
        ))
    }

    fn build_bash(&self, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let mut result = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);
        let expose_string = bash::expose("pcmd_dispatch", prompts, remote_cmd);

        result.push('\n');
        result.push_str(&prompt_functions);
        result.push('\n');
        result.push_str(&expose_string);

        Ok(result)
    }

    fn build_bashrc(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);

        Ok(bashrc::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_bashposix(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);

        Ok(bashposix::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_zsh(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);

        Ok(zsh::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_sh(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = sh::create_cmd_functions(prompts, "pcmd_dispatch");

        Ok(sh::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_ash(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = ash::create_cmd_functions(prompts, "pcmd_dispatch");

        Ok(ash::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_dash(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = dash::create_cmd_functions(prompts, "pcmd_dispatch");

        Ok(dash::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }

    fn build_bashlike_functions(&self, dispatcher_name: &str, prompts: &[String]) -> String {
        bashrc::create_cmd_functions(prompts, dispatcher_name)
    }

    fn build_fish(&self, workdir: &str, channel: &Channel, prompts: &[String], remote_cmd: Option<&[&str]>) -> Result<String, ShellError> {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel)?;
        let prompt_functions = fish::create_cmd_functions(prompts, "pcmd_dispatch");

        Ok(fish::expose(workdir, &prompt_functions, &dispatcher_func, remote_cmd))
    }
}

