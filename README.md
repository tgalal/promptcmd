# AI Box

## Usage II

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

