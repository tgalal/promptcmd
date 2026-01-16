use clap::{value_parser, Arg, ArgGroup, Command};
use promptcmd::config::resolver::{ResolvedGlobalProperties, ResolvedPropertySource};
use promptcmd::config::{self, appconfig_locator};
use promptcmd::config::appconfig::{AppConfig, GlobalProviderProperties};
use promptcmd::cmd::run;
use promptcmd::dotprompt::renderers::argmatches::DotPromptArgMatches;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::executor::{Executor, PromptInputs};
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::RusqliteStore;
use promptcmd::stats::store::StatsStore;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use promptcmd::storage::PromptFilesStorage;
use std::sync::{Arc, Mutex};
use std::{env};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use log::debug;


fn main() -> Result<()> {
    env_logger::init();

    let prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
    );

    let stats_store = RusqliteStore::new(
        config::base_home_dir()?
    )?;

    let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        AppConfig::try_from(
            fs::read_to_string(&appconfig_path)?.as_str()
        )?
    } else {
        AppConfig::default()
    };

    // Find the executable name directly from args.
    let mut args = env::args();

    let path: PathBuf = args
        .next()
        .context("Could not figure binary name")?
        .into();

    let invoked_binname: String = path
        .file_stem()
        .context("Could not get filename")?
        .to_string_lossy()
        .into();

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
    command = command.arg(Arg::new("dry")
        .long("dry")
        .help("Dry run")
        .action(clap::ArgAction::SetTrue)
        .required(false))
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

    let arc_statsstore: Arc<Mutex<dyn StatsStore + Send>> = Arc::new(Mutex::new(stats_store));
    let lb = WeightedLoadBalancer {
        stats: Arc::clone(&arc_statsstore)
    };
    let arc_prompts_storage = Arc::new(Mutex::new(prompts_storage));
    let arc_appconfig = Arc::new(appconfig);
    let executor = Executor {
        loadbalancer: lb,
        appconfig: arc_appconfig,
        statsstore: arc_statsstore,
        prompts_storage: arc_prompts_storage
    };

    let arc_executor = Arc::new(executor);

    let dry = *matches.get_one::<bool>("dry").unwrap_or(&false);

    let resolved_cmd_properties = ResolvedGlobalProperties {
        source: ResolvedPropertySource::Inputs,
        properties: GlobalProviderProperties {
            temperature: matches.get_one::<f32>("temperature").copied(),
            max_tokens: matches.get_one::<u32>("max_tokens").copied(),
            model: None,
            system: matches.get_one::<String>("system").map(|s| s.to_string()),
            cache_ttl: matches.get_one::<u32>("cache_ttl").copied(),
        }
    };

    let requested_model = matches.get_one::<String>("model").map(|s| s.to_string());

    let argmatches = DotPromptArgMatches {
        matches,
        dotprompt: &dotprompt
    };

    let inputs: PromptInputs = argmatches.try_into()?;

    let result = arc_executor.execute_dotprompt(&dotprompt,
        Some(resolved_cmd_properties), requested_model,
        inputs, dry)?;
    println!("{}", result);

    Ok(())
}
