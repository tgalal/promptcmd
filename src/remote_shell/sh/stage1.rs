use crate::remote_shell::{sh::ShRemoteShell, Stage1};

// writes and executes stage2
impl<'a> Stage1 for ShRemoteShell<'a> {
    fn stage1(&self, stage2: &str) -> String {
        format!(r#"
mkdir -m 700 -p {workdir}
touch {workdir}/stage2.sh
chmod 600 {workdir}/stage2.sh

printf '%s' '
{stage2}
' >> {workdir}/stage2.sh

exec {sh_bin} {workdir}/stage2.sh

"#,
            sh_bin=self.sh_bin,
            workdir=&self.workdir)
    }
}
