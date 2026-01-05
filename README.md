# promptcmd

promptcmd transforms your LLM prompts into runnable programs:

```bash
$ askrust "What's the code for creating a loop again"
$ echo "How is it going?" | translate --to German
```

Or use in VIM without plugins:

```
TODO video showing selection of code and fixing it with a command
```

It doesn't even need to "look like" a command per se:

```bash
$ How can I create a loop in rust
```

## What?

Prompts are described within [Dotprompt]() files.
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

To create a new one:

```bash
$ promptctl create How
```

This bootstraps a configuration for a command called `How` and opens it in your
favorite editor. Lets put in the following:

## Usage

### Read

```bash
promptctl cat translate
```

### Make own Prompts

```bash
promptctl create translate
```

This opens up an editor for the translate prompt, saves config to home
and creates a symlink.

### Override Existing Prompts

```bash
promptctl edit translate
```

### Schema Modifiers

Input schema fields support modifiers:

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

## Configuration

Configuration is stored in TOML format and searched in the following locations:

TODO

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

## Paths

### Config Paths

Linux

```
~/.config/promptcmd/config.toml
/etc/config.toml
~/.promptcmd/config.toml
```

MAC

```
```

### Prompt File Search Paths

TODO:

Linux:

```
~/.local/share/promptcmd/prompts/
```

MAC:

```
```

### Installation Paths

Linux:

```
~/.promptcmd/installers/symlink/
~/.local/bin/
```

MAC:

```
```


## License

See LICENSE file for details.

## Troubleshooting/FAQ

TODO

Other models

Not in path/Notfound

ENV

Defaults/Frontmatter required?

## Advanced Configurations

### Variants

### Load Balancing

TODO

## Examples

Import and use any of the examples under [examples]() using `promptctl import`:

```
promptctl import TODO
```

