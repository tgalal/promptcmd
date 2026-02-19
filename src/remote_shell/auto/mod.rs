pub mod stage3;

struct AutoRemoteShell<'a> {
    workdir: &'a str
}

impl<'a> AutoRemoteShell<'a> {
    pub fn new(workdir: &'a str) -> Self {
        Self {
            workdir
        }
    }
}
