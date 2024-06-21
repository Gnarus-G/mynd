# mynd

_Yet another todo app_.

A [very] simple todo list management cli tool for developers, with an optional gui. The fastest way I've found to go from needing to write something quickly (i.e during a meeting)
to having it written down.

## Edit in the terminal

![image](https://github.com/Gnarus-G/mynd/assets/37311893/17729eb9-ab8b-42f4-aaf2-8d2014356f89)

## Edit in an editor (e.g Neovim with language server)

![image](https://github.com/Gnarus-G/mynd/assets/37311893/7a79b1fa-704d-481a-bac4-2b1e067ef9c4)

## Edit in the GUI

![image](https://github.com/Gnarus-G/mynd/assets/37311893/69358ce2-5711-4f5b-a8be-cb989ec0c112)

## Features

- [x] Simple GUI
- [x] CLI for efficiency
- [x] Local Persistence Option
- [x] Soft Delete Done Items
- [x] Permanent Deletion (Drap-n-drop to trash bin)
- [x] Todo Language & LSP
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

or, to lauch your default editor [$EDITOR].

```sh
todo
```

Note: This option [editor] is only viable with the lsp intergration.

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
  edit    Edit the tood list in your default editor ($EDITOR) [default]
  dump    Dump all todos as json
  config  Manage global configuration values
  lsp     Start the language server
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
      filetypes = { "todolang" },
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

For codelens support in Neovim; Set it up so the codelenses refresh often, and have a convenient
shortcut for running codelenses.

```lua
local function on_attach(_, bufnr)
  -- ...

  -- codeLens
  -- auto refresh code lens
  local codelenses_augroup = vim.api.nvim_create_augroup("codeLenses", { clear = true })

  local function supports_code_lenses(buf)
    local clients = vim.lsp.get_clients({ buffer = buf })
    for _, client in pairs(clients) do
      if client.server_capabilities.codeLensProvider then
        return true
      end
    end
    return false
  end

  if supports_code_lenses(bufnr) then
    vim.api.nvim_create_autocmd({ 'CursorHold', 'BufEnter', 'InsertLeave' }, {
      group = codelenses_augroup,
      buffer = bufnr,
      callback = function(event)
        vim.lsp.codelens.refresh({ bufnr = event.buf })
      end
    })

    vim.api.nvim_create_autocmd('LspDetach', {
      group = codelenses_augroup,
      buffer = bufnr,
      callback = function(event)
        vim.lsp.codelens.clear(event.data.client_id, event.buf)
      end
    })
  end

  vim.keymap.set('n', '<leader>lr', vim.lsp.codelens.run, opts)
  -- ...
end
```

## Dev References

https://github.com/tauri-apps/tauri-docs/blob/8cdc0505ffb9e78be768a0216bd91980306206a5/docs/guides/distribution/sign-android.md
https://github.com/neovim/neovim/pull/13165
https://github.com/ray-x/navigator.lua/blob/master/lua/navigator/lspwrapper.lua#L122
