use crate::{config::appconfig::BashMethod, remote_shell::{sh::ShRemoteShell, Stage2, Stage3, Shell,
    Channel}};


fn create_dispatcher(channel: &Channel, session_pwd: &str) -> String {
    let dispatcher_body = match channel {
        Channel::Nc(_, remote_port) => {
            format!("prep_args \"$@\" | nc localhost {remote_port}")
        },
        Channel::Socat(_, remote_sockfile) => {
            format!("prep_args \"$@\" | socat -,ignoreeof UNIX-CONNECT:{remote_sockfile}")
        },
        Channel::BashTcp(_, remote_port) => {
            format!("prep_args \"$\"@ | bash -c \"exec 3<>/dev/tcp/localhost/{remote_port}; cat >&3; cat <&3; exec 3>&-\"")
        },
        Channel::FifoSingle(workdir) => {
            format!("prep_args \"$@\" | cat >> {workdir}/send && cat {workdir}/recv")
        },
        Channel::Fifo(workdir) => {
            format!(r#"
identifier=$$
rendezvousfile="{workdir}/rendezvous"
sendfile="{workdir}/${{identifier}}_send"
recvfile="{workdir}/${{identifier}}_recv"

[ -p "$rendezvousfile" ] || mkfifo -m 600 "$rendezvousfile";
mkfifo -m 600 "$sendfile" "$recvfile"
trap "rm -f $sendfile $recvfile" EXIT

if [ "$1" = "__exit__" ]; then
    printf "__exit__\n" >> "$rendezvousfile"
    exit
fi

# Register with server
printf "CONN %s\n" "$identifier" >> "$rendezvousfile"

# Send and wait for response
prep_args \"$@\" | cat >> "$sendfile" && cat "$recvfile"

"#)
        }
    };

    format!(r#"#!/bin/sh

prep_args() {{
    {{
    printf "%s\n" '{session_pwd}'
    printf "%s " "$1"
    shift
    for arg in "$@"; do
        printf "\"%s\" " "$arg"
    done
    printf "\0"
    if [ ! -t 0 ]; then
        cat
    fi
    printf "\0"
    }}
}}

dispatch() {{
    {dispatcher_body}
}}

dispatch "$@"
    "#)
}

impl<'a> Stage2 for ShRemoteShell<'a> {
    fn stage2(&self, dispatcher_name: &str, prompt_names: &[String],
        channel: &Channel, shell: &Shell, bash_method: &BashMethod,
        remote_cmd: Option<&str>,
        session_pwd: &str)-> String {
        let workdir = &self.workdir;
        let dispatcher_path = format!("{workdir}/{dispatcher_name}.sh");
        let functions_file = format!("{workdir}/functions");
        let dispatcher_invocation = format!("sh {dispatcher_path}");

        let dispatcher_init_code = match channel {
            Channel::FifoSingle(workdir) => {
                format!(r#"
[ -p {workdir}/send ] || mkfifo -m 600 {workdir}/send;
[ -p {workdir}/recv ] || mkfifo -m 600 {workdir}/recv;
"#)
            },
            _ => {
                "".to_string()
            }
        };

        let stage3_block = if !matches!(shell, Shell::Auto(_)) {
            let stage3 = shell.stage3(&functions_file, prompt_names, remote_cmd);
            let functions = shell.create_prompt_functions(&dispatcher_invocation, prompt_names);
            format!(r#"
cat > {functions_file} << EOF_STAGE2
{functions}
EOF_STAGE2

cat > {workdir}/stage3.sh << EOF_STAGE2
{stage3}
EOF_STAGE2
            "#, functions=self.escape(&functions), stage3=self.escape(&stage3))
        } else {
            self.create_autoshell(&functions_file, &dispatcher_invocation, prompt_names, bash_method, remote_cmd)
        };

        format!(r#"
OLD_UMASK=$(umask); umask 077
cat > {dispatcher_path} << EOF_STAGE2
{dispatcher_code}
EOF_STAGE2

{stage3_block}

pcmd_exit() {{
    {dispatcher_invocation} __exit__
    rm -rf {workdir}
    rm {workdir}.sock 2> /dev/null
}}

if [ ! -f "{workdir}/trap" ]; then
    touch {workdir}/trap
    trap "pcmd_exit" EXIT
fi

{dispatcher_init_code}

umask $OLD_UMASK

(exec {sh_bin} {workdir}/stage3.sh)
        "#,
            sh_bin=self.sh_bin,
            dispatcher_code=self.escape(&create_dispatcher(channel, session_pwd)))
    }
}

