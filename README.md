# mynd

_Yet another todo app_.

A [very] simple todo list management cli tool for developers, with an optional gui. The fastest way I've found to go from needing to write something quickly (i.e during a meeting)
to having it written down.

![image](https://github.com/Gnarus-G/mynd/assets/37311893/69358ce2-5711-4f5b-a8be-cb989ec0c112)

## Features

- [x] Simple GUI
- [x] CLI for efficiency
- [x] Local Persistence Option
- [x] Soft Delete Done Items
- [x] Permanent Deletion (Drap-n-drop to trash bin)
- [ ] Todo Language & LSP
- [ ] Remind Command: /r (Desktop Notifications)
- [ ] Remote Persistence Option

## Install (Linux only)

### Recommended Option

```sh
curl -fsSL https://raw.githubusercontent.com/Gnarus-G/mynd/main/install.sh | sh
```

This depends on you having installed [bun](https://bun.sh/) and [rust](https://doc.rust-lang.org/cargo/getting-started/installation.html); `git` as well, but you
probably already have that.

### Other Options

Find the executables in the [releases](https://github.com/Gnarus-G/mynd/releases).

## Usage

### CLI

At any point you can pull up your terminal and add a todo item like so.

```sh
todo "todo message"
```

### GUI

Start up the GUI.

```sh
todo gui
```

Or just call `mynd` directly, which is what `todo gui` does.

Very convenient when your manager is rapping requirements at you during a meeting.

```
Usage: todo [MESSAGE] [COMMAND]

Commands:
  done    Mark one or more todo items as done
  rm      Delete a todo item, regardless of if it's done or not
  ls      List all todos that aren't done
  gui     Launch the GUI (mynd). Assuming it's in the path
  import  Read and save todos from a given file
  dump    Dump all todos as json
  config  Manage global configuration values
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [MESSAGE]  What to do

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Syntax Highlighting (Neovim)

There is a treesitter grammar for the `todo` syntax.
To setup syntax Highlighting in Neovim, see [tree-sitter-todolang](https://github.com/Gnarus-G/tree-sitter-todolang)

## Lsp setup

I doubt this language server will ever land into `neovim/nvim-lspconfig`, so here's an example
of my lsp config setup.

```lua
local nvim_lsp = require("lspconfig");

local configs = require 'lspconfig.configs'

if not configs.todols then
  configs.todols = {
    default_config = {
      cmd = { "todo", "lsp" },
      filetypes = { "td", "todo" },
    },
  }
end

nvim_lsp.todols.setup({
  on_attach = on_attach, --[[your on_attach function goes here]]
  single_file_support = true,
  capabilities = require('cmp_nvim_lsp')
      .default_capabilities(vim.lsp.protocol.make_client_capabilities())
})
```

## Dev References

https://github.com/tauri-apps/tauri-docs/blob/8cdc0505ffb9e78be768a0216bd91980306206a5/docs/guides/distribution/sign-android.md
