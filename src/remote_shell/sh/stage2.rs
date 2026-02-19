use crate::{config::appconfig::BashMethod, remote_shell::{sh::ShRemoteShell, Stage2, Stage3, Shell,
    Channel}};


fn create_dispatcher(channel: &Channel) -> String {
    let dispatcher_body = match channel {
        Channel::Nc(_, remote_port) => {
            format!("prep_args $@ | nc localhost {remote_port}")
        },
        Channel::Socat(_, remote_sockfile) => {
            format!("prep_args $@ | socat -,ignoreeof UNIX-CONNECT:{remote_sockfile}")
        },
        Channel::BashTcp(_, remote_port) => {
            format!("prep_args $@ | bash -c \"exec 3<>/dev/tcp/localhost/{remote_port}; cat >&3; cat <&3; exec 3>&-\"")
        },
        Channel::Fifo(workdir) => {
            format!("prep_args $@ | cat >> {workdir}/send && cat {workdir}/recv")
        }
    };

    format!(r#"#!/bin/sh

prep_args() {{
    {{
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

dispatch $@
    "#)
}

impl<'a> Stage2 for ShRemoteShell<'a> {
    fn stage2(&self, dispatcher_name: &str, prompt_names: &[String],
        channel: &Channel, shell: &Shell, bash_method: &BashMethod)-> String {
        let workdir = &self.workdir;
        let dispatcher_path = format!("{workdir}/{dispatcher_name}.sh");
        let functions_file = format!("{workdir}/functions");
        let dispatcher_invocation = format!("sh {dispatcher_path}");

        let dispatcher_init_code = match channel {
            Channel::Fifo(workdir) => {
                format!("mkfifo {workdir}/send; mkfifo {workdir}/recv")
            },
            _ => {
                "".to_string()
            }
        };

        let stage3_block = if !matches!(shell, Shell::Auto(_)) {
            let stage3 = shell.stage3(&functions_file, prompt_names);
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
            self.create_autoshell(&functions_file, &dispatcher_invocation, prompt_names, bash_method)
        };

        format!(r#"
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
(exec {sh_bin} {workdir}/stage3.sh)
        "#,
            sh_bin=self.sh_bin,
            dispatcher_code=self.escape(&create_dispatcher(channel)))
    }
}

