use crate::{config::appconfig::BashMethod, remote_shell::{bashexports::BashExportsRemoteShell,
    bashposix::BashPosixRemoteShell, bashrc::BashRcRemoteShell, fish::FishRemoteShell,
    sh::ShRemoteShell, zsh::ZshRemoteShell, Stage3, Shell}};

impl Stage3 for Shell {
    fn stage3(&self, functions_file: &str, prompts_names: &[String], remote_cmd: Option<&str>) -> String {
        match self {
            Shell::Sh(workdir) => {
                ShRemoteShell::new("sh", workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Ash(workdir) => {
                ShRemoteShell::new("ash", workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Dash(workdir) => {
                ShRemoteShell::new("dash", workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Fish(workdir) => {
                FishRemoteShell::new(workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Zsh(workdir) => {
                ZshRemoteShell::new(workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Bash(BashMethod::Rc, workdir) => {
                BashRcRemoteShell::new("bash", workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Bash(BashMethod::Posix, workdir) => {
                BashPosixRemoteShell::new("bash", workdir)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            Shell::Bash(BashMethod::Exports, workdir) => {
                BashExportsRemoteShell::new("bash", workdir, false)
                    .stage3(functions_file, prompts_names, remote_cmd)
            }
            _ => {panic!("Not implemented")}
        }
    }

    fn create_prompt_functions(&self, dispatcher_name: &str, prompts_names: &[String]) -> String {
        match self {
            Shell::Sh(workdir) => {
                ShRemoteShell::new("sh", workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Ash(workdir) => {
                ShRemoteShell::new("ash", workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Dash(workdir) => {
                ShRemoteShell::new("dash", workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Fish(workdir) => {
                FishRemoteShell::new(workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Zsh(workdir) => {
                ZshRemoteShell::new(workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Bash(BashMethod::Rc, workdir) => {
                BashRcRemoteShell::new("bash", workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Bash(BashMethod::Posix, workdir) => {
                BashPosixRemoteShell::new("bash", workdir)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            Shell::Bash(BashMethod::Exports, workdir) => {
                BashExportsRemoteShell::new("bash", workdir, false)
                    .create_prompt_functions(dispatcher_name, prompts_names)
            }
            _ => {panic!("Not implemented")}
        }

    }
}
