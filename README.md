# mmemo

Simple CLI memo management tool.

## Usage
```
NAME:
    mmemo - Simple CLI memo management tool

USAGE:
    mmemo <command> [args]

COMMANDS:
    init, i              Initialize configuration and create config.toml
    new, n <title...>    Create a new memo (spaces are allowed)
    list, l              List all memos
    edit, e              Select and edit a memo
    view, v              Select and view a memo
    grep, g <pat...>     Search memos
    delete, d            Select and delete a memo
    config, c            Open config.toml in your editor

GLOBAL OPTIONS:
    -h, --help           Show help
    -v, --version        Show version
```

## Installation

### Nix (flake)

```bash
nix run github:<YOUR_NAME>/<REPO> -- --help
nix run github:<YOUR_NAME>/<REPO> -- list
```

### Cargo (crates.io)
```bash
cargo install mmemo
mmemo --version
```

## Configuration

Run `mmemo init` to create config.toml under your XDG config directory.
- Example: ~/.config/mmemo/config.toml
- `mmemo init` also creates a default template at `~/.config/mmemo/template.txt`.

```toml
# Editor to use for editing memos (optional, default: vim)
editor = "vim"

# Directory to store memos (required)
memo_dir = "~/mmemo"

# Template file for new memos (optional)
# A default template is created by `mmemo init`: ~/.config/mmemo/template.txt
# Supports {{title}} and {{date}} placeholders
memo_template = "~/.config/mmemo/template.txt"

# Selector: builtin or fzf or skim (optional, default: builtin)
selector = "builtin"

# Viewer: builtin or glow (optional, default: builtin)
viewer = "builtin"

# Grep: builtin or ripgrep(rg) (optional, default: builtin)
grep = "builtin"
```

### Notes
- memo_dir / memo_template support ~/ prefix.
- External backends require commands available in PATH.
- If an external command is not found, switch the corresponding setting to "builtin".

## Memo Template
By default, `mmemo init` creates `~/.config/mmemo/template.txt` and config.toml points to it.
Edit that file to customize the template.

If you set `memo_template`, `mmemo new` will start from that template.

Only these placeholders are supported:

- `{{title}}` : memo title
- `{{date}}`  : creation date (`YYYY-MM-DD`)

Other fields such as tags or categories are not supported.

### YAML frontmatter example

```md
---
title: {{title}}
date: {{date}}
---

# {{title}}
```

## Backends
### Grep backend
| Backend | Configuration |  Requirement |
| --------------- | --------------- |  ---------------|
| builtin | grep = "builtin" | none |
| [ripgrep](https://github.com/BurntSushi/ripgrep)  | grep = "ripgrep"  | ripgrep (rg) |

-	builtin: simple AND search (all patterns must appear in the line)
-	ripgrep: passes arguments to rg as-is

Examples (ripgrep backend):
```bash
mmemo grep -n todo
mmemo grep -e "-foo" # pattern starts with '-'
```

### Selector backend (edit/view/delete)
| Backend | Configuration | Requirement |
| --------------- | --------------- | --------------- |
| builtin | selector = "builtin" | none |
| [fzf](https://github.com/junegunn/fzf)  | selector = "fzf" | fzf |
| [skim](https://github.com/skim-rs/skim)  | selector = "skim" | sk (skim) |


### Viewer backend (view)
| Backend | Configuration | Requirement |
| --------------- | --------------- | --------------- |
| builtin | viewer = "builtin" | none |
| [glow](https://github.com/charmbracelet/glow)  | viewer = "glow" | glow |


## License
MIT
