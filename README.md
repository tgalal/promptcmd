# promptcmd

promptcmd turns your GenAI prompts into runnable programs:

```bash
$ askrust "What's the code for creating a loop again"
$ echo "How is it going?" | translate --to German
```

Or use within an editor without plugins:

![askrust_vim_demo](./docs/img/askrust.gif)

It doesn't even need to "look like" a command per se:

```bash
$ How can I create a loop in rust
```

## Table of Contents

- [What](#what)
- [Usage](#usage)
- [Monitoring Usage](#monitoring-usage)
- [Configuration](#configuration)
- [Advanced](#advanced-configuration)
  - [Variants](#variants)
  - [Load Balancing](#load-balancing)
- [Roadmap](#roadmap)
- [Examples](#examples)
- [License](#license)

## What?

Prompts are described within [Dotprompt](https://google.github.io/dotprompt/) files.
Use `promptctl` to create a `translate` prompt file:

```bash
$ promptctl create translate
```

Insert the following:

```
---
model: anthropic/claude-sonnet-4 #(or another model of your choice)
input:
  schema:
    to: string, Target language
---
Translate the following to {{to}}:
{{STDIN}}
```

Save, and close. Follow the printed instructions for configuring your model,
then you can run:

```bash
$ translate --help

Usage: translate --to <to>

Options:
      --to <to>   Target language
  -h, --help     Print help
```

And execute:

```
$ echo " Hello world!" | translate --to German

Hallo Welt
```

To uninstall the program (remove from path):

```bash
$ promptctl disable translate
```

To enable again:

```bash
$ promptctl enable translate
```

## Usage

```bash
$ promptctl --help

Usage: promptctl [OPTIONS] <COMMAND>

Commands:
  edit           Edit an existing prompt file
  enable         Enable a prompt
  disable        Disable a prompt
  create         Create a new prompt file [aliases: new]
  list           List commands and prompts [aliases: ls]
  cat            Print promptfile contents
  run            Run promptfile
  import         Import promptfile
  stats          Print statistics
  resolve        Resolve model name
  help           Print this message or the help of the given subcommand(s)
```

### dotprompt Files

These are files based on [dotprompt](https://google.github.io/dotprompt/) where
prompts are described in the following format:


```
---
model: model_name
input:
  schema:
    input1: string, This is a required string input
    input2: string, This is an optional string input
    input3: boolean, This is true/false input
    input4: integer, This is an integer input
    input5: number, This is an integer or a float input
output:
  format: text # can also be json
---
This is the template section. You can inject any of the declared inputs
like {{input1}} or {{input4}}.

You can generally make use of handlebar syntax like conditional checks:

{{#if (eq input3 "true")}}
input3 is set
{{/if}}

{{#if (gt input4 5)}}
input4 is greater than 5
{{else}}
input4 is less than 5
{{/if}}

Or render stdin:

{{STDIN}}
```

Input schema fields modifiers:

- `field`: Required, named argument
- `field?`: Optional, named argument
- `field!`: Required, positional argument
- `field?!` or `field!?`: Optional, positional argument

### Supported Data Types

- `string`: Text input
- `boolean`: Flag/switch argument
- `integer`: Integer
- `number`: Integer or float

## Monitoring Usage

Monitor all use of provider + model:

```
$ promptctl stats

provider      model                     runs     prompt tokens     completion tokens     avg tps
anthropic     claude-opus-4-5           15       1988              1562                  31
openai        gpt-5-mini-2025-08-07     2        88                380                   42
```

Get information of the last execution:

```
$ promptctl stats --last

provider      model               prompt tokens     completion tokens     time
anthropic     claude-opus-4-5     206               168                   5
```

Perform dry runs (simulate runs without querying the model provider):

```
$ promptctl run --dry PROMPTNAME -- [PROMPT ARGS]
```

## Configuration

Configuration is stored in TOML format and is looked up at:

Linux:

```
~/.config/promptcmd/config.toml
```

### Example Configuration

```toml
[providers]
temperature = 0.7
max_tokens = 1000

[providers.anthropic]
api_key = "sk-ant-..."

[providers.openai]
endpoint = "https://api.openai.com/v1"

[providers.ollama]
endpoint = "http://localhost:11434"

```

### Paths

### Config Paths

Linux

```
~/.config/promptcmd/config.toml
```

MAC

```
~/Library/Application Support/promptcmd/config.toml
```

### Prompt File Search Paths

Linux:

```
~/.local/share/promptcmd/prompts/
```

MAC

```
~/Library/Application Support/promptscmd/prompts/
```

### Installation Paths

```
~/.local/share/promptcmd/installed/symlink/
```

MAC

```
~/Library/Application Support/promptcmd/installed/symlink/
```

## Advanced Configurations

### Variants

You can define custom "instances" of an already configured model, and refer to
it by name:

```toml
[providers.anthropic.rust-coder]
system = """You are a rust coding assistant helping me with rust questions.
Be brief, do not use markdown in your answers. Prefer to answer with pure code
(no before and after explanation unless very appropriate)."""
```

`rust-coder` is referred to as Variant of the Base Anthrophic. A Variant
inherits all properties of its Base, and optionally overrides any of them.
It can be referred to in dotprompt files by name:

```
---
model: rust-coder
---

Template here
```

### Load Balancing

You can group several bases or variants together into a group, load balancing
executions across them:

```
[groups.group1]
providers = [
  "anthropic", "openai"
]

# or vary the ratio of execution in terms of total number of tokens:

[groups.group2]
providers = [
  { "name": "openai", "weight": 1 },
  { "name": "anthropic", "weight": 2 },
]
```

## Roadmap

- [x] Google, Anthropic, OpenRouter, Ollama, OpenAI
- [x] Groups and Load balancing
- [x] Symlink Installer
- [x] Shebang Support
- [x] MAC suppport
- [ ] Windows suppport
- [ ] Support tools
- [ ] Better statistics
- [ ] Advanced Load balancing
- [ ] Interactive Input Program Installer
- [ ] File Inputs
- [ ] Web UI Installer


## Examples

Import and use any of the examples under [examples]() using `promptctl import`:

```
curl https://raw.githubusercontent.com/tgalal/promptcmd/refs/heads/main/examples/commitmsg.toml | \
promptctl import -pe commitmsg -
```

## License

See LICENSE file for details.

