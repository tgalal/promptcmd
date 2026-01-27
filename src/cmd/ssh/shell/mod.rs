pub mod zsh;
pub mod bash;
pub mod sh;

pub enum Shell {
    Bash,
    // workdir
    Zsh(&'static str),
    // workdir
    Sh(&'static str),
    Auto(&'static str)
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
    fn build(&self, shell: &Shell) -> ChannelCode {
        match self {
            Channel::Nc(_, remote_port) => self.build_nc(*remote_port, shell),
            Channel::Socat(_, remote_socket) => self.build_socat(remote_socket, shell),
            Channel::BashTcp(_, remote_port) => self.build_bashtcp(*remote_port, shell),
        }
    } 

    fn build_nc(&self, port: u32, shell: &Shell) -> ChannelCode {
        match shell {
            Shell::Auto(_) | Shell::Bash | Shell::Zsh(_) | Shell::Sh(_) => ChannelCode::from_post(format!("| nc localhost {port}")),
        }
    }

    fn build_bashtcp(&self, port: u32, shell: &Shell) -> ChannelCode {
        match shell {
            Shell::Auto(_) | Shell::Bash | Shell::Zsh(_) | Shell::Sh(_) => { 
                ChannelCode {
                    pre: Some(format!("exec 3<>/dev/tcp/localhost/{port}")),
                    post: r#">&3
cat <&3
exec 3<&-
exec 3>&-"#.to_string()
                }
                 
            }
        }
    }

    fn build_socat(&self, socket_path: &str, shell: &Shell) -> ChannelCode {
        match shell {
            Shell::Auto(_) | Shell::Bash | Shell::Zsh(_) | Shell::Sh(_) => ChannelCode::from_post(format!("| socat -,ignoreeof UNIX-CONNECT:{socket_path} ")),
        }
    }
}

impl Shell {

    pub fn build(&self, channel: &Channel, prompts: &[String]) -> String {
        match self  {
            Shell::Bash => self.build_bash(channel, prompts),
            Shell::Zsh(workdir) => self.build_zsh(workdir, channel, prompts),
            Shell::Sh(workdir) => self.build_sh(workdir, channel, prompts),
            Shell::Auto(workdir) => self.build_auto(workdir, channel, prompts)
    }
    }

    fn build_dispatcher_func(&self, funcname: &str,  channel: &Channel) -> String {
        let channel_code = channel.build(self);

        match self {
            Shell::Auto(_) | Shell::Bash | Shell::Zsh(_) | Shell::Sh(_) => {
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
                    # Only read stdin if it's not a terminal (i.e., piped data)
                    if [ ! -t 0 ]; then
                        cat
                    fi
                    printf "\0"
                    }}{post}
            }}"#, pre=channel_code.pre.unwrap_or("".to_string()), post=&channel_code.post )
            }
        }
    }
    fn build_auto(&self, workdir: &str, channel: &Channel, prompts: &[String]) -> String {

        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel);

        let bash_expose = bash::expose("pcmd_dispatch", prompts);
        let zsh_expose = zsh::expose(workdir, &prompt_functions, &dispatcher_func);
        let sh_expose  = sh::expose(workdir, &prompt_functions, &dispatcher_func);

        format!(r#"
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
        )

    }

    fn build_bash(&self, channel: &Channel, prompts: &[String]) -> String {
        let mut result = self.build_dispatcher_func("pcmd_dispatch", channel);
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);
        let expose_string = bash::expose("pcmd_dispatch", prompts);

        result.push('\n');
        result.push_str(&prompt_functions);
        result.push('\n');
        result.push_str(&expose_string);

        result
    }

    fn build_zsh(&self, workdir: &str, channel: &Channel, prompts: &[String]) -> String {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel);
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);

        zsh::expose(workdir, &prompt_functions, &dispatcher_func)
    }

    fn build_sh(&self, workdir: &str, channel: &Channel, prompts: &[String]) -> String {
        let dispatcher_func = self.build_dispatcher_func("pcmd_dispatch", channel);
        let prompt_functions = self.build_bashlike_functions("pcmd_dispatch", prompts);
 
        sh::expose(workdir, &prompt_functions, &dispatcher_func)
    }

    fn build_bashlike_functions(&self, dispatcher_name: &str, prompts: &[String]) -> String {
        bash::create_cmd_functions(prompts, dispatcher_name)
    }
}

