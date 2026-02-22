use clap::{Arg, Command};
use promptcmd::cmd::ssh::utils::WaitForMasterResult;
use promptcmd::cmd::ssh::{controlpath, utils};
use promptcmd::config::resolver::{ResolvedGlobalProperties, ResolvedPropertySource};
use promptcmd::config::{self, appconfig_locator};
use promptcmd::config::appconfig::{AppConfig, GlobalProviderProperties};
use promptcmd::cmd::{self, command_add_configuration_options, command_add_general_options, command_add_remote_options, run};
use promptcmd::dotprompt::renderers::argmatches::DotPromptArgMatches;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::executor::{ExecContext, ExecutionOutput, Executor, MultiplexedSession, PromptInputs, RemoteExecContext};
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::{RusqliteStore};
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use promptcmd::storage::PromptFilesStorage;
use promptcmd::ENV_CONFIG;
use tokio::process::Child;
use std::sync::{Arc};
use anyhow::{Context, Result, anyhow, bail};
use std::{env, process};
use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use log::debug;
use std::io::{self, Write};
use std::sync::OnceLock;

static PROMPTS_STORAGE: OnceLock<FileSystemPromptFilesStorage> = OnceLock::new();
static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();
static STATS_STORE: OnceLock<RusqliteStore> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let prompt_storage_path = config::prompt_storage_dir()?;
    let base_home_dir = config::base_home_dir()?;
    let prompts_storage = PROMPTS_STORAGE.get_or_init(||
        FileSystemPromptFilesStorage::new(prompt_storage_path)
    );
    let statsstore = STATS_STORE.get_or_init(||
        match RusqliteStore::new(base_home_dir) {
            Ok(store) => store,
            Err(err) => panic!("{}", err)
        }
    );

    let appconfig_path =
        env::var(ENV_CONFIG).ok().map(PathBuf::from)
        .or_else(appconfig_locator::path);

    let appconfig = if let Some(appconfig_path) = appconfig_path.as_ref() {
        if !appconfig_path.exists() {
            bail!("Could not find a config file at {} ", appconfig_path.to_string_lossy());
        }
        let appconfig_data = fs::read_to_string(appconfig_path)
        .map_err(|e| anyhow!("Error reading config at {}: {e}", appconfig_path.to_string_lossy()))?;

        APP_CONFIG.get_or_init(||
            match AppConfig::try_from(appconfig_data.as_str()) {
                Ok(appconfig) => appconfig,
                Err(err) => panic!("Failed to initialize: {}", err)
            }
        )
    } else {
        APP_CONFIG.get_or_init(AppConfig::default)
    };

    // Find the executable name directly from args.
    let mut args = env::args();

    let path: PathBuf = args
        .next()
        .context("Could not figure binary name")?
        .into();

    let invoked_binname: String = path
        .file_name()
        .context("Could not get filename")?
        .to_string_lossy()
        .into();

    #[cfg(target_os="windows")]
    let invoked_binname: String = if let Some(exe_stripped) =
    invoked_binname.strip_suffix(".exe") {
        exe_stripped.to_string()
    } else {
        invoked_binname
    };

    debug!("Executable name: {invoked_binname}");

    let mut command: Command = Command::new(&invoked_binname);
    let promptname = if invoked_binname == config::RUNNER_BIN_NAME {
        // Not running: via symlink, first positional argument is the prompt name or path
        command = command.arg(Arg::new("promptname"));
        args
            .next()
            .context("Could not determine prompt name")?

    } else {
        // if the executable name differs from BIN_NAME, then this might be symlink
        // TODO: check!
        invoked_binname
    };
    debug!("Prompt name: {promptname}");
    // Check if loading by path (this handles also shebangs)
    let path = PathBuf::from(&promptname);
    let promptdata = if path.exists() {
        debug!("Reading prompt from file: {}", promptname);
        fs::read_to_string(path)?
    } else {
        debug!("Reading prompt from storage");
        prompts_storage.load(&promptname)?.1
    };

    let dotprompt: DotPrompt = DotPrompt::try_from((promptname.as_str(), promptdata.as_str()))?;

    command = command.disable_help_flag(true);
    command = command.next_help_heading("Prompt inputs");
    command = run::generate_arguments_from_dotprompt(command, &dotprompt)?;
    command = command_add_general_options(command);
    command = command_add_remote_options(command);
    command = command_add_configuration_options(command);

    let matches = command.get_matches();

    let lb = WeightedLoadBalancer {
        stats: statsstore
    };

    let remote_dest = matches.get_one::<String>("remote_dest");
    let remote_port = *matches.get_one::<u32>("remote_port").unwrap_or(&22);

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let (conmon_tx, conmon_rx) = tokio::sync::oneshot::channel::<()>();
    let (ssh_handle, executor) = if let Some(remote_dest) = remote_dest {
        let controlpath = controlpath::control_path(process::id()).context("Could not determine control path")?;
        let destination = utils::get_destination(remote_dest);
        debug!("Destination: {:#?}", destination);

        let session_info = MultiplexedSession {
            controlpath: controlpath.clone(),
            destination,
            port: remote_port
        };

        let connection_sharing_args = vec![
            "-o".to_string(),
            "ControlMaster=yes".to_string(),
            "-o".to_string(),
            format!("ControlPath={}", &controlpath),
            "-o".to_string(),
            "ControlPersist=no".to_string(),
        ];

        debug!("Setting up master connection to {}", &remote_dest);
        let remote_dest_copy = remote_dest.clone();
        let ssh_cmd_handle = tokio::spawn(async move {
            let mut child = tokio::process::Command::new("ssh")
                .arg("-t")
                .arg("-N")
                .arg("-p")
                .arg(remote_port.to_string())
                .args(connection_sharing_args)
                .arg(remote_dest_copy)
                .spawn().context("Error in ssh proc")?;

            tokio::select! {
                _ = child.wait() => {
                    debug!("SSH session terminated normally");
                    let _ = conmon_tx.send(());
                }
                 _ = rx => {
                    debug!("Terminating master SSH session due to shutdown signal");
                    let _ = child.kill().await;
                }
            }
            Ok::<Child, anyhow::Error>(child)
        });

        debug!("Waiting 30 seconds for master connection to succeed");
        let wait_conn_result = cmd::ssh::utils::async_wait_for_master_ready(
            &controlpath,
            remote_dest,
            remote_port,
            Duration::from_secs(120),
            Some(conmon_rx)
        ).await.map_err(|err|
            anyhow!(err)
        )?;

        if let WaitForMasterResult::Timeout = wait_conn_result {
            bail!("Timeout waiting for control master");
        } else if let WaitForMasterResult::Aborted = wait_conn_result {
            debug!("Connection was aborted");
            return Ok(())
        }

        debug!("Master connection succeeded, proceeding.");

        (Some(ssh_cmd_handle),
        Executor {
            loadbalancer: lb,
            appconfig,
            statsstore,
            prompts_storage,
            exec_context: ExecContext::Remote(RemoteExecContext::MultiplexedSession(session_info.clone()))
        })
    } else {
        (None, Executor {
            loadbalancer: lb,
            appconfig,
            statsstore,
            prompts_storage,
            exec_context: ExecContext::Local
        })
    };


    let arc_executor = Arc::new(executor);

    let dry = *matches.get_one::<bool>("dry").unwrap_or(&false);
    let render = *matches.get_one::<bool>("render").unwrap_or(&false);

    let stream = if let Some(true) = matches.get_one::<bool>("stream") {
        Some(true)
    } else if let Some(true) = matches.get_one::<bool>("nostream") {
        Some(false)
    } else {
        None
    };

    let resolved_cmd_properties = ResolvedGlobalProperties::from((
        &GlobalProviderProperties {
            temperature: matches.get_one::<f32>("temperature").copied(),
            max_tokens: matches.get_one::<u32>("max_tokens").copied(),
            model: None,
            system: matches.get_one::<String>("system").map(|s| s.to_string()),
            cache_ttl: matches.get_one::<u32>("cache_ttl").copied(),
            stream
        },
        ResolvedPropertySource::Inputs
    ));

    let requested_model = matches.get_one::<String>("model").map(|s| s.to_string());

    let argmatches = DotPromptArgMatches {
        matches,
        dotprompt: &dotprompt
    };

    let inputs: PromptInputs = argmatches.try_into()?;

    let result = arc_executor.execute_dotprompt(&dotprompt,
        Some(resolved_cmd_properties), requested_model,
        inputs, None, dry, render).await?;

    match result {
        ExecutionOutput::StreamingOutput(mut stream) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            let mut ends_with_newline = false;

            while let Some(res) = stream.sync_next().await {
                let data_str = res?;
                let data = data_str.as_bytes();

                ends_with_newline = data_str.ends_with("\n");
                handle.write_all(data)?;
                handle.flush()?;
            }
            if !ends_with_newline {
                handle.write_all("\n".as_bytes())?;
            }
        }
        ExecutionOutput::StructuredStreamingOutput(mut stream) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let mut ends_with_newline = false;

            while let Some(res) = stream.sync_next().await {
                let data_str = res?;
                let data = data_str.as_bytes();

                ends_with_newline = data_str.ends_with("\n");
                handle.write_all(data)?;
                handle.flush()?;
            }
            if !ends_with_newline {
                handle.write_all("\n".as_bytes())?;
            }
        }
        ExecutionOutput::ImmediateOutput(output) => {
            print!("{}", &output);
            if !output.ends_with("\n") {
                println!();
            }
        }
        ExecutionOutput::DryRun(output) => {
            println!("{output}");
        }
        ExecutionOutput::Cached(output) => {
            print!("{}", &output);
            if !output.ends_with("\n") {
                println!();
            }
        }
        ExecutionOutput::RenderOnly(output) => {
            println!("{}", &output);
        }
    };

    if let Some(handle) = ssh_handle {
        let _ = tx.send(());
        let _ = handle.await;
    }

    Ok(())
}
