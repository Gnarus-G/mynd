# mynd

Yet another todo app, because I'm opinionated about the dumbest things.

![image](https://github.com/Gnarus-G/mynd/assets/37311893/1e1567a7-2c06-4371-a66d-c44513c8b0d7)

## Features

- [x] Simple GUI
- [x] CLI for efficiency
- [x] Local Persistence Option
- [x] Soft Delete Done Items
- [x] Permanent Deletion
- [ ] Remind Command: /r (Desktop Notifications)
- [ ] Remote Persistence Option

## Install

```sh
git clone https://github.com/Gnarus-G/mynd
cd mynd
sh install.sh
```

This depends on you having installed [bun](https://bun.sh/) and [rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Usage

Start up the GUI.

```sh
mynd
```

At any point you can pull up your terminal and add a todo item like so.

```sh
todo "todo message"
```

Very convenient when your manager is rapping requirements at you during a meeting.

```
Usage: todo [MESSAGE] [COMMAND]

Commands:
  done    Mark one or more todo items as done
  rm      Delete a todo item, regardless of if it's done or not
  ls      List all todos that aren't done
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

## Dev References

https://github.com/tauri-apps/tauri-docs/blob/8cdc0505ffb9e78be768a0216bd91980306206a5/docs/guides/distribution/sign-android.md
