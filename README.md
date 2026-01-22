# promptcmd

[Quick Start](#quick-start) | [Documentation](https://docs.promptcmd.sh) | [Examples](#examples)

**promptcmd** is a manager and executor for programmable prompts. Define a
prompt template once, then execute it like any other terminal command, complete
with argument parsing, `--help` text, and stdin/stdout integration.

![promptcmd demo](./docs/img/create.gif)

Unlike tools that require you to manually manage prompt files or rely on
implicit tool-calling, promptcmd gives you explicit control over what data your
models access. Build compositional workflows by nesting prompts, executing
shell commands within templates, and piping data through multi-step AI
pipelines.

## Key Features

- **Prompts as CLI Commands**: Create a `.prompt` file, enable it with
`promptctl`, and execute it like any native tool.

- **Multi-Provider Support**: Supports Anthropic (Claude),
OpenAI (GPT), Google (Gemini), OpenRouter, and Ollama.

- **Caching and Load Balancing**:
Configure load balancing groups to distribute requests across providers with
equal or weighted distribution for cost optimization. Configure caching in
order to not waste tokens on identical prompts.

## Quick Start

### Install

**Linux/macOS:**
```bash
curl -LsSf https://installer.promptcmd.sh | sh
```

**macOS (Homebrew):**
```bash
brew install tgalal/tap/promptcmd
```

**Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://installer-ps.promptcmd.sh | iex"
```

### Configure API Keys

Configure your API keys by editing `config.toml`:

```bash
promptctl config edit
```
Find your provider's name, e.g., for anthropic:

```toml
[providers.anthropic]
api_key = "sk-ant-api03-..."
```

Alternatively, you can set the keys via Environment Variables:

```
PROMPTCMD_ANTHROPIC_API_KEY="your_api_key"
PROMPTCMD_OPENAI_API_KEY="your_api_key"
PROMPTCMD_ANTHROPIC_API_KEY="your_api_key"
PROMPTCMD_OPENROUTER_API_KEY="your_api_key"
```

### Create Your First Prompt

Create a `summarize.prompt` file:

```bash
promptctl create summarize
```

Insert the following:

```yaml
---
model: anthropic/claude-sonnet-4-5
input:
  schema:
    words?: integer, Summary length in words
---
Summarize the following text{{#if words}} in {{words}} words{{/if}}:

{{STDIN}}
```

Enable and use it:

```bash
# Enable as a command
promptctl enable summarize

# Use it
cat article.txt | summarize
echo "Long text here..." | summarize --words 10

# Auto-generated help
summarize --help
```

That's it. Your prompt is now a native command.

## Documentation

**Full documentation available at: [docs.promptcmd.sh](http://docs.promptcmd.sh)**

## Examples

Browse the [Examples](https://github.com/tgalal/promptcmd/tree/main/examples)
directory or visit [https//promptcmd.sh/lib](https://promptcmd.sh/lib) for
interactive viewing.

## License

GPLv3 License - see LICENSE file for details
