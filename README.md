![promptcmd](./docs/img/banner.png)

```bash
$ askrust "What's the code for creating a loop again"
$ echo "How is it going?" | translate --to German
```

Use prompts directly from your editor without plugins:

![askrust_vim_demo](./docs/img/askrust.gif)

Natural language commands work too:

```bash
$ How can I create a loop in rust
```

---

## Table of Contents

- [Getting Started](#getting-started)
- [Command Reference](#command-reference)
- [Dotprompt Files](#dotprompt-files)
  - [Schema Syntax](#schema-syntax)
  - [Supported Data Types](#supported-data-types)
  - [Examples](#examples)
- [Configuration](#configuration)
  - [Example Configuration](#example-configuration)
  - [File Locations](#file-locations)
- [Advanced Configuration](#advanced-configuration)
  - [Variants](#variants)
  - [Load Balancing](#load-balancing)
  - [Caching](#caching)
- [Monitoring Usage](#monitoring-usage)
- [Related Projects](#related-projects)
- [License](#license)

---

## Getting Started

Prompts are defined using [Dotprompt](https://google.github.io/dotprompt/) files. Create a new prompt with `promptctl`:

```bash
$ promptctl create translate
```

Define your prompt template:

```yaml
---
model: anthropic/claude-sonnet-4
input:
  schema:
    to: string, Target language
---
Translate the following to {{to}}:
{{STDIN}}
```

After saving, follow the configuration instructions, then use your new command:

```bash
$ translate --help

Usage: translate --to <to>

Options:
      --to <to>   Target language
  -h, --help     Print help
```

Execute the command:

```bash
$ echo "Hello world!" | translate --to German
Hallo Welt
```

Manage your commands:

```bash
# Disable a command
$ promptctl disable translate

# Re-enable a command
$ promptctl enable translate
```

---

## Command Reference

Available commands:

| Command | Description |
|---------|-------------|
| `create` | Create a new prompt file |
| `edit` | Edit an existing prompt file |
| `enable` | Enable a prompt command |
| `disable` | Disable a prompt command |
| `list` | List all commands and prompts |
| `cat` | Display prompt file contents |
| `run` | Execute a prompt file |
| `import` | Import an existing prompt file |
| `stats` | View usage statistics |
| `resolve` | Resolve model name to provider |

---

## Dotprompt Files

Dotprompt files define your prompt templates using a YAML frontmatter format:

```yaml
---
model: model_name
input:
  schema:
    input1: string, This is a required string input
    input2?: string, This is an optional string input
    input3: boolean, This is true/false input
    input4: integer, This is an integer input
    input5: number, This is an integer or a float input
    input5(enum, This is an choice input): [option1, option2, option3]
output:
  format: text  # Can also be 'json'
---
This is the template section. You can inject declared inputs using
handlebars syntax: {{input1}} or {{input4}}.

Conditional logic is supported:

{{#if input3 )}}
input3 is set
{{/if}}

{{#if (gt input4 5)}}
input4 is greater than 5
{{else}}
input4 is less than or equal to 5
{{/if}}

You can also render stdin:

{{STDIN}}
```

### Schema Syntax

Field modifiers control how arguments are parsed:

| Syntax | Description |
|--------|-------------|
| `field` | Required, named argument (`--field value`) |
| `field?` | Optional, named argument (`--field value`) |
| `field!` | Required, positional argument |
| `field?!` or `field!?` | Optional, positional argument |

### Supported Data Types

| Type | Description |
|------|-------------|
| `string` | Text input |
| `boolean` | Flag or switch argument |
| `integer` | Whole numbers |
| `number` | Integers or floating-point numbers |
| `enum` | Choose one of the options|


### Examples

Browse the [examples](examples) directory or check out the gallery at
[promptcmd/lib](https://promptcmd.sh/lib/).

---

## Configuration

A single TOML configuration file `config.tml`.

### Example Configuration

```toml
[providers]
temperature = 0.7
max_tokens = 1000
# For 20 seconds identical request respond from cache
cache_ttl = 20

[providers.anthropic]
api_key = "sk-ant-..."

[providers.openai]
endpoint = "https://api.openai.com/v1"

[providers.ollama]
endpoint = "http://localhost:11434"
```

For full reference see [config.example.toml](./config.example.toml).

### File Locations

**Configuration File**

| Platform | Path |
|----------|------|
| Linux | `~/.config/promptcmd/config.toml` |
| macOS | `~/Library/Application Support/promptcmd/config.toml` |

**Prompt Files**

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/promptcmd/prompts/` |
| macOS | `~/Library/Application Support/promptcmd/prompts/` |

**Installed Commands**

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/promptcmd/installed/symlink/` |
| macOS | `~/Library/Application Support/promptcmd/installed/symlink/` |

---

## Advanced Configuration

### Variants

Create specialized model configurations that inherit from base providers:

```toml
[providers.anthropic.rust-coder]
system = """You are a Rust coding assistant.
Be brief, avoid markdown formatting.
Provide code-first answers without unnecessary explanations."""
```

Reference variants by name in your prompt files:

```yaml
---
model: rust-coder
---
{{STDIN}}
```

Variants inherit all properties from their base provider and can override any settings.

### Load Balancing

Distribute requests across multiple providers using groups:

```toml
# Equal distribution
[groups.balanced]
providers = ["anthropic", "openai"]

# Weighted distribution (by token count)
[groups.weighted]
providers = [
  { name = "openai", weight = 1 },
  { name = "anthropic", weight = 2 }
]
```

### Caching

You can specify an amount of time where identical requests result in cached
responses, i.e., without hitting your provider's API.

This can be done across all providers or specific to particular ones:

```toml
[providers]
cache_ttl = 2 # cache for number of seconds

[providers.anthropic]
cache_ttl = 4

[providers.anthropic.myvariant]
cache_ttl = 8
```
---

## Monitoring Usage

Track usage across all providers and models:

```bash
$ promptctl stats

provider      model                     runs     prompt tokens     completion tokens     avg tps
anthropic     claude-opus-4-5           15       1988              1562                  31
openai        gpt-5-mini-2025-08-07     2        88                380                   42
```

View statistics for the most recent execution:

```bash
$ promptctl stats --last

provider      model               prompt tokens     completion tokens     time
anthropic     claude-opus-4-5     206               168                   5
```

Test prompts without calling the API:

```bash
$ promptctl run --dry PROMPTNAME -- [PROMPT ARGS]
```

## Related Projects

promptcmd is inspired by existing tools like
[llm](https://github.com/simonw/llm),
[runprompt](https://github.com/chr15m/runprompt),
[claude-switcher](https://github.com/andisearch/claude-switcher), among others.
What distinguishes promptcmd is that it treats prompts as first-class CLI
programs. Each prompt becomes a dedicated command with typed parameters, help
text, and proper argument parsing—reducing repeated boilerplate.

Further distinguishing aspects:

- Prompts can invoke other prompts for compositional workflows where each prompt
does one thing and does it (hopefully) right.
- Load balancing and auto selecting next model based on token consumption
- Caching responses for a defined number of seconds

Check out this [comparison table](https://promptcmd.sh/#comparison) for more details.

---

## License

See LICENSE file for details.

