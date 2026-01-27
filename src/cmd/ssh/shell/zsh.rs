pub fn setup(workdir: &str, functions: &str) -> String {
    format!(r#"
mkdir -p {workdir}
cat > {workdir}/{functions_file} << "EOF"
{functions}
EOF

echo "source {workdir}/{functions_file}" > {workdir}/.zshenv
echo "[[ -e ~/.zshenv ]] &&  source ~/.zshenv" >> {workdir}/.zshenv
echo "source ~/.zshrc" > {workdir}/.zshrc
zsh -c "ZDOTDIR={workdir} zsh"
"#,
    functions_file="funcs"
    )
}
