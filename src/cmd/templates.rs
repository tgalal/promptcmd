pub const PROMPTFILE: &str = r#"---
# model: model
# input:
#   schema:
# output:
#   format: text
---
You are a useful assistant

{{STDIN}}
"#;

pub const ONBOARDING_ANTHROPIC: &str = r#"
You have successfully created a prompt file using Anthropic as the LLM provider.
An API key and an Anthropic model are required by not yet configured. To configure an API key:

1. Go to https://console.anthropic.com/settings/keys
2. Sign up or log in to your account
3. Navigate to API Keys and create a new key
4. Copy the key and add it to your configuration file:

[providers.anthropic]
api_key="sk-ant-xxxxx..."

Edit your configuration file by running:

promptctl config --edit

Or manually create/edit the file at:

"#;

pub const ONBOARDING_OPENAI: &str = r#"
You have successfully created a prompt file using OpenAI as the LLM provider.
An API key and an OpenAI model are required by not yet configured. To configure an API key:

1. Go to https://platform.openai.com/settings/organization/api-keys
2. Sign up or log in to your account
3. Create a new key
4. Copy the key and add it to your configuration file:

[providers.openai]
api_key="sk-xxxxx..."

Edit your configuration file by running:

promptctl config --edit

Or manually create/edit the file at:

"#;

pub const ONBOARDING_GOOGLE: &str = r#"
You have successfully created a prompt file using Google as the LLM provider.
An API key and a Google model is required by not yet configured. To configure an API key:

1. Go to https://aistudio.google.com/api-keys
2. Sign up or log in to your account
3. Create a new key
4. Copy the key and add it to your configuration file:

[providers.google]
api_key="sk-xxxxx..."

Edit your configuration file by running:

promptctl config --edit

Or manually create/edit the file at:

"#;

pub const ONBOARDING_OPENROUTER: &str = r#"
You have successfully created a prompt file using OpenRouter as the LLM provider.
An API key and a supported model is required by not yet configured. To configure an API key:

1. Go to https://openrouter.ai/settings/keys
2. Sign up or log in to your account
3. Create a new key
4. Copy the key and add it to your configuration file:

[providers.openrouter]
api_key="sk-xxxxx..."

Edit your configuration file by running:

promptctl config --edit

Or manually create/edit the file at:

"#;

pub const ONBOARDING_OLLAMA: &str = r#"
You have successfully created a prompt file using Ollama as the LLM provider.
An endpoint and model are required by not yet configured. Update your
configuration file:

[providers.ollama]
endpoint="http://ollama_address:11434"

Edit your configuration file by running:

promptctl config --edit

Or manually create/edit the file at:

"#;

pub const CONFIG_TEMPLATE: &str =
r#"############### promptcmd ################
#
# To modify, uncomment any section to edit
# it configuration.
#
##########################################
### Configuration for the "create" command
##########################################
# [create]
# no_enable = false # Auto enable prompt once created
# force = false  # Force save prompt files disregarding validation result

##########################################
### Configuration for the "import" command
##########################################
# [import]
# enable = false # Auto enable prompt once imported
# force = false  # Force import disregarding validation result

###########################################
### Default Configuration for all providers
###########################################
# [providers]
### Default model to use if not specified in prompt files.
### Can be in provider/model format, just the provider name,
### a variant, or a group name.
# default = "ollama/gpt-oss:20b"
# temperature = 1.0
# system = "You are a useful assistant"
# max_tokens = 1000
# cache_ttl = 0 # Number of seconds to cache responses

#################################
### GenAI Providers Configuration
#################################
# [providers.openai]
# model = "gpt-5-mini-2025-08-07"
# api_key = "sk-proj-xxxx"
# temperature = 1.0
# max_tokens = 1.0
# system = "You are a useful assistant"
# cache_ttl = 0

# [providers.openrouter]
# model = "anthropic/claude-sonnet-4"
# api_key = "sk-or-xxxx"
# temperature = 1.0
# max_tokens = 1.0
# system = "You are a useful assistant"
# cache_ttl = 0

# [providers.google]
# model = "gemini-2.5-flash"
# api_key = "aaaaaa..."
# temperature = 1.0
# max_tokens = 1.0
# system = "You are a useful assistant"
# cache_ttl = 0

# [providers.anthropic]
# api_key = "sk-ant-xxxx"
# model = "claude-opus-4-5"
# temperature = 1.0
# max_tokens = 1.0
# system = "You are a useful assistant"
# cache_ttl = 0

# [providers.ollama]
# endpoint = "http://127.0.0.1:11434"
# model = "gpt-oss:20b"
# cache_ttl = 0

#########################################################################
### Configurations for Variants.
### These inherit their provider's configuration, overriding any property
### as needed
#########################################################################
# [providers.anthropic.rust-coder]
# system = """You are a rust coding assistant helping me with rust questions.
# Be brief, do not use markdown in your answers. Prefer to answer with pure code
# (no before and after explanation unless very appropriate)"""

##########################################################################
### Configurations for Groups.
### Executions are load balanced across members of a group based on
### token consumption. Load is split proportionate to any indicated weight
###########################################################################
# [groups.balanced]
# providers = [
#   "anthropic", "google", "openrouter"
# ]

# [groups.unbalanced]
# providers = [
#   { name = "google",    weight = 5 },
#   { name = "anthropic", weight = 1 },
# ]
"#;
