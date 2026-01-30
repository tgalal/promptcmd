use clap::{value_parser, Arg, ArgGroup, Command};
use promptcmd::config::resolver::{ResolvedGlobalProperties, ResolvedPropertySource};
use promptcmd::config::{self, appconfig_locator};
use promptcmd::config::appconfig::{AppConfig, GlobalProviderProperties};
use promptcmd::cmd::run;
use promptcmd::dotprompt::renderers::argmatches::DotPromptArgMatches;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::executor::{ExecutionOutput, Executor, PromptInputs};
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::{RusqliteStore};
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use promptcmd::storage::PromptFilesStorage;
use promptcmd::ENV_CONFIG;
use std::sync::{Arc};
use std::{env};
use anyhow::{Context, Result};
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

    let appconfig = if let Some(appconfig_path) = appconfig_path && appconfig_path.exists() {
        let appconfig_data = fs::read_to_string(&appconfig_path)?;
        let appconfig = match AppConfig::try_from(appconfig_data.as_str()) {
            Ok(appconfig) => appconfig,
            Err(err) => panic!("Failed to parse configuration at {}: {}", appconfig_path.to_string_lossy(), err)
        };
        APP_CONFIG.get_or_init(|| appconfig)
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
    command = command.next_help_heading("General Options");
    command = command.arg(
        Arg::new("dry")
            .long("dry")
            .help("Dry run")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(Arg::new("render")
            .long("render")
            .short('r')
            .help("Render only mode")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(
            Arg::new("help")
            .long("help")
            .short('h')
            .action(clap::ArgAction::Help)
            .help("Print help")
        );
    command = command.next_help_heading("Optional Configuration Overrides")
        .arg(Arg::new("model")
            .long("config-model")
            .short('m')
        )
        .arg(Arg::new("stream")
            .long("config-stream")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(Arg::new("nostream")
            .long("config-no-stream")
            .action(clap::ArgAction::SetTrue)
        )
        .group(ArgGroup::new("streamgroup").args(["stream", "nostream"]))
        .arg(Arg::new("cache_ttl")
            .long("config-cache-ttl")
            .value_parser(value_parser!(u32))
        )
        .arg(Arg::new("temperature")
            .long("config-temperature")
            .alias("config-temp")
            .value_parser(value_parser!(f32))
        )
        .arg(Arg::new("max_tokens")
            .long("config-max-tokens")
            .value_parser(value_parser!(u32))
        )
        .arg(Arg::new("system")
            .long("config-system")
        )
        ;

    let matches = command.get_matches();

    let lb = WeightedLoadBalancer {
        stats: statsstore
    };
    let executor = Executor {
        loadbalancer: lb,
        appconfig,
        statsstore,
        prompts_storage
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
        inputs, dry, render).await?;

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
        ExecutionOutput::DryRun => {
            println!("[dry run, no llm response]");
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

    Ok(())
}
