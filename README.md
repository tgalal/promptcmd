# Command? Prompt!

Command? Prompt! integrates AI into your terminal. Why leave your terminal when
you can come up with any command to run a prompt, like:

```
$ askrust "What's the code for creating a loop again"
$ echo "How is it going?" | translate --to German
```

Or use in VIM w/o plugins:

```
video showing selection of code and fixing it with a command 
```

It doesn't even need to "look like" a command per se:

```
$ How can I create a loop in rust
```

## What?

Lets take a look at `translate` from above, using the `promptctl` tool:

```
$ promptctl edit translate
```

This is a [dotprompt]() file. The schema of `input` is used to generate on the fly a command
with the same name (by default), accepting the arguments described above:

```
$ translate --help
```

To uninstall the program (remove from path):

```
$ promptctl disable translate
```

To enable again:

```
$ promptctl enable translate
```

To create a new one:

```
$ promptctl create How
```

This bootstraps a configuration for a command called `How` and opens it in your
favorite editor. Lets put the following:

```
---
model: ollama/gpt-oss:20b
input:
  schema:
    question!: string, A question to ask AI
output:
  format: text
---
Following is a question from the user. Be brief, and straight to the point:

How {{question}}?
```

Save, quit. Now you can run:

```
$ How can I stop relying on AI
```

## Names

- PromptCtl
- Prompt Commander
- Command Prompt
- Command & Prompt
- CmdPrompt
- cmdprmpt
- cmdpmpt
- Command: Prompted
- BYOCommand
- BYOPrompt

## Usage

### Run

```
promptctl run [--dry] translate -- --from en --to german
promptbox promptname [promptboxargs] [promptargs]
translate [promptboxargs][promptargs]
```

### Read

```
promptctl read translate
```

### Make own Prompts

```
promptctl new translate
```

This opens up an editor for the translate prompt, saves config to home
and creates a symlink

### Override Existing Prompts

```
promptctl edit translate
```

Like systemd, creates translate.override

### List

```
# Option 1
promptctl ls --enabled --disabled --long --prompts | --commands

/home/tarek/asdsad/translate.prompt -> /home/tarek/.bin/translate
/home/tarek/asdsad/askrust.prompt [disabled]

# Option 2

translate -> /home/tarek/asdas/translate.prompt
N/A -> /home/tarek/adsasd/ask.prompt


# --commands
translate ask-rust

# --commands -l
translate -> /home/tarke/asdasd/translate.prompt


# --commands --paths
/home/tarek/bin/translate 
/home/tarek/bin/askrust 

# --commands -l  --paths

/usr/local/bin/translate -> /home/tarke/asdasd/translate.prompt

```


## Paths

Dynamic Symlinks Paths (must be writable)

```
CWD
/home/USER/.aibox/bin/
/home/USER/.local/bin/
/usr/local/bin/
```

Binary Paths (symlinked)

```
CURR BIN DIR/runner (maybe only that?)
/home/USER/.aibox/bin/
/home/USER/.local/bin/
/usr/local/bin/
```

