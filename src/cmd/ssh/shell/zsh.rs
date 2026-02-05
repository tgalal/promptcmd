
pub fn expose(workdir: &str, functions: &str, dispatcher_func: &str, remote_cmd: Option<&[&str]>) -> String {
    let remote_cmd = if let Some(remote_cmd) = remote_cmd {
        let remote_cmd_joined = remote_cmd.join(" ");
        format!("zsh -c \"ZDOTDIR={workdir} zsh -c '{remote_cmd_joined}; exit'\"")
    } else {
        format!("zsh -c 'ZDOTDIR={workdir} zsh'")
    };
    format!(r#"
mkdir -p {workdir}
chmod 700 {workdir}
cat > {workdir}/{functions_file} << "EOF"
{dispatcher_func}
{functions}
if [ ! -f "{workdir}/trap" ]; then
    touch {workdir}/trap
    echo "setting trap"
    trap 'rm -rf {workdir}' EXIT
fi
EOF

echo "source {workdir}/{functions_file}" > {workdir}/.zshenv
echo "[[ -e ~/.zshenv ]] &&  source ~/.zshenv" >> {workdir}/.zshenv
echo "source ~/.zshrc" > {workdir}/.zshrc
{remote_cmd}
"#,
    functions_file="funcs"
    )
}

