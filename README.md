# mynd

Yet another todo app, because I'm opinionated about the dumbest things.

![image](https://github.com/Gnarus-G/mynd/assets/37311893/1e1567a7-2c06-4371-a66d-c44513c8b0d7)

## Features

- [x] Simple GUI
- [x] CLI for efficiency
- [x] Local Persistence Option
- [x] Soft Delete Done Items
- [ ] Permanent Deletion
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
  done  Mark one or more todo items as done
  ls    List all todos that aren't done
  dump  Dump all todos as json
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [MESSAGE]  What to do

Options:
  -h, --help     Print help
  -V, --version  Print version
```
