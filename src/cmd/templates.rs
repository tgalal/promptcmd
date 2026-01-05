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

You can create/edit your configuration file at:

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

You can create/edit your configuration file at:

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

You can create/edit your configuration file at:

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

You can create/edit your configuration file at:

"#;

pub const ONBOARDING_OLLAMA: &str = r#"
You have successfully created a prompt file using Ollama as the LLM provider.
An endpoint and model are required by not yet configured. Update your
configuration file:

[providers.ollama]
endpoint="http://ollama_address:11434"

You can create/edit your configuration file at:

"#;
