# Project

Project is a cross-platform CLI program to execute commands defined in the `Project.yml`. Each command is a Lua script. The main different from [just](https://github.com/casey/just) is Project focus on cross-platform scripting instead of rely on the other tools.

> [!WARNING]
> There is a plan to upgrade to Lua 5.5 so don't write your script in such a way that it is not [compatible](https://www.lua.org/manual/5.5/manual.html#8.1) with it.

## Key features

- Batteries included.
- Small and lightweight.
- Easy to install on Linux, macOS and Windows.
- Single executable with only system dependencies.
- Lua 5.4 as scripting language.
- Non-blocking concurrent execution with Lua thread.
- Simple APIs designed for project automation.

## Why use this instead of Python?

- Project are easy lightweight to install (especially if you already have Rust or on Windows).
- No additional steps to install external dependencies required by your scripts.
- APIs designed for project automation.
- Declarative command.

## Installation

There are 3 ways to install Project:

1. Via cargo (recommended if you have Rust).
2. Via automated script (recommended if you don't have Rust).
3. Manual download.

### Cargo

If you have Rust installed you can use [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) to install Project:

```sh
cargo install --git https://github.com/ultimaweapon/project.git
```

Try running Project to see if you have Cargo installation directory in the `PATH`:

```sh
project
```

It should output something like:

```
Failed to open Project.yml: No such file or directory (os error 2).
```

If it error with command not found you need to add Cargo installation directory to `PATH` manually. You can find this directory in the outputs of `cargo install` on the above.

## Quick start

Create `Project.yml` in the root of your repository with the following content:

```yaml
commands:
  build:
    description: Build the project
    args:
      release:
        description: Enable optimization
        long: release
        short: r
        type: bool
    script: scripts/build.lua
```

Then create `scripts/build.lua` with the following content:

```lua
if args['release'] then
  print('Start building release build!')
else
  print('Start building debug build!')
end
```

Then run:

```sh
project --help
```

It will outputs something like:

```
Run a command defined in Project.yml

Usage: project <COMMAND>

Commands:
  build  Build the project
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

Notice the `build` command that loaded from your `Project.yml`. Run the following command to see how to use your `build` command:

```sh
project build --help
```

It will outputs something like:

```
Build the project

Usage: project build [OPTIONS]

Options:
  -r, --release  Enable optimization
  -h, --help     Print help
```

If you run `project build` it will run `scripts/build.lua`, which output the following text to the console:

```
Start building debug build!
```

## Script API

Lua implementation used here is [Tsuki](https://github.com/ultimaweapon/tsuki). There are some differences with vanilla Lua, which you can see in Tsuki's README. The following is a list of additional changes from Project:

- `pcall` has been removed.
- `os` library has `exit` removed.

### args[name]

A global variable contains all command arguments. If argument `name` does not present it will return `false` for `bool` argument or `nil` for the other type.

### exit(code)

Cause Project process to exit immediately. Unlike `os.exit`, this function always close all to-be-closed variables.

### json.parse(json)

Parse a JSON string and return a corresponding value (e.g. the result will be a table for JSON object).

### os.arch

Architecture of the OS. The value will be one of `aarch64` and `x86_64`.

### os.createdir(path [, ...])

Recursively create a directory and all of its parent components if they are missing. Path will be **joined** together with native path separator to form a path to directory so:

```lua
local r = os.createdir('abc', 'def')

if t[1] then
  -- 'abc' does not exists before and was created by the call
end
```

Will result in `abc/def` on *nix and `abc\def` on Windows as a path to create. This use [PathBuf::push](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push) to create the path so if any arguments is an absolute path it will **discard** the path that was created by previous arguments.

Returns a table consist of argument number as a key and `boolean` as a value indicated if the component was created by the call (that is, not exists before the call).

### os.kind

Kind of the OS. The value will be one of `linux`, `macos` and `windows`.

### os.removedir(path [, ...])

Remove a directory and its content, which mean it will **always** remove the directory even if the directory is not empty. Path will be **joined** together with native path separator to form a path to directory so:

```lua
os.removedir('abc', 'def')
```

Will result in `abc/def` on *nix and `abc\def` on Windows as a path to remove. This use [PathBuf::push](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push) to create the path so if any arguments is an absolute path it will **discard** the path that was created by previous arguments.

### os.run(prog [, ...])

Run `prog` with the remaining arguments as its arguments. Unlike `os.execute`, this does not use OS shell to run `prog`. This function will raise an error by default if `prog` exit with non-zero code. By default, stdin will be a null stream and stdout/stderr will be inherits from Project process and working directory will be the directory that contains `Project.yml`.

All `nil` in the arguments will be removed (e.g. `os.run('echo', 'abc', nil, 'def')` will invoke `echo` with only 2 arguments).

### os.capture(prog [, ...])

Run `prog` with the remaining arguments as its arguments and return its outputs, which is stdout by default. This does not use OS shell to run `prog`. The returned string will have LF and/or CR at the end removed by default. This function will raise an error by default if `prog` exit with non-zero code. By default, stdin will be a null stream and a non-captured stream will be inherits from Project process. Working directory will be the directory that contains `Project.yml` by default.

All `nil` in the arguments will be removed (e.g. `os.capture('echo', 'abc', nil, 'def')` will invoke `echo` with `abc` and `def` as arguments).

If `prog` is a table the item at index #1 must be the name of program to run and it can contains the following additional items:

#### from

Can be either `stdout`, `stderr` or `both`. If this key does not present it will default to `stdout`. With `both` this function will return a table contains `stdout` and `stderr` fields.

### os.spawn(prog [, ...])

Run `prog` with the remaining arguments as its arguments and return a process object to manipulate it. This does not use OS shell to run `prog`. By default, stdin will be a null stream and a non-captured stream will be inherits from Project process. Working directory will be the directory that contains `Project.yml` by default.

All `nil` in the arguments will be removed (e.g. `os.spawn('echo', 'abc', nil, 'def')` will spawn `echo` with only 2 arguments).

The process object can be a [to-be-closed](https://www.lua.org/manual/5.4/manual.html#3.3.8) variable, which will kill the process when the object goes out of scope. If the variable does not have `close` attribute the process will get killed when the object is freed by Lua GC.

If `prog` is a table the item at index #1 must be the name of program to run and it can contains the following additional items:

#### cwd

Working directory for the process. If this key does not present it will default to the directory that contains `Project.yml`.

#### stdout

Can be either `null`, `inherit` or `pipe`. If this key does not present it will default to `inherit`. For `pipe` the process object will have `stdout` property, which have [read](https://www.lua.org/manual/5.4/manual.html#pdf-file:read) method.

### path.basename(path)

Returns the final component of the path, if there is one. This use [Path::file_name](https://doc.rust-lang.org/std/path/struct.Path.html#method.file_name) to extract the name.

### path.join(component [, ...])

Returns a joined path components so:

```lua
path.join('abc', 'def')
```

Will result in `abc/def` on *nix and `abc\def` on Windows. This use [PathBuf::push](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push) to create the path so if any arguments is an absolute path it will **discard** the path that was created by previous arguments.

### Url:new(url)

Create an instance of `Url` class from `url`. This class has the following properties and methods:

#### path

Returns the path for the URL, as a percent-encoded ASCII string.

## Exit code

Project will exit with exit code 0 when all operations completed successfully. The script can use `exit` to exit with a custom exit code. The code 100 and above are reserved for Project use and have the following meaning:

### 100

The script exit with an error. This is runtime error, not compile time.

### 101

Project process was panic. This indicate an underlying bug on Project itself to please report this!

### 102

Project unable to open `Project.yml`.

### 103

Project unable to parse `Project.yml`.

### 104

No action is defined for some commands.

### 105

Project unable to read Lua script for the command.

### 106

Project unable to load Lua script for the command.

### 109

Project unable to setup Tokio.

## Project.yml

### commands

List of commands.

### commands.<command_id>

A command definition.

### commands.<command_id>.description

Description of the command.

### commands.<command_id>.script

Path to Lua script to execute when this command is invoked. Path separator always is `/` even on Windows and Project will convert to native path.

## License

This project is licensed under either of

- Apache License, Version 2.0,
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
