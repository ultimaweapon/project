# Project

Project is a cross-platform CLI program to execute commands defined in the `Project.yml`. Each command is a Lua script. The main different from [just](https://github.com/casey/just) is Project focus on cross-platform scripting instead of rely on the other tools.

## Key features

- Batteries included.
- Small and lightweight.
- Easy to install on all platforms.
- Single executable with only system dependencies.
- Lua 5.4 as scripting language.

## Installation

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
print('Hello, world!')
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
Hello, world!
```

## Script API

Lua [standard libraries](https://www.lua.org/manual/5.4/manual.html#6) are available except `debug` and `package`. The `os` library also has `exit` and `setlocale` removed. Note that Lua version is 5.4 **without** compatibility with the previous version. You can see the list of the differences [here](https://www.lua.org/manual/5.4/manual.html#8).

### os.arch()

Returns architecture of the OS. The value will be one of `aarch64` and `x86_64`.

### os.kind()

Returns kind of the OS. The value will be one of `linux`, `macos` and `windows`.

### os.run(prog [, ...])

Run `prog` with the remaining arguments as its arguments. Unlike `os.execute`, this does not use OS shell to run `prog`. This function will raise an error by default if `prog` exit with non-zero code. By default, all standard streams will be inherits from Project process and working directory will be the directory that contains `Project.yml`.

If `prog` is not an absolute path, the `PATH` will be searched in an OS-defined way.

All `nil` in the arguments will be removed (e.g. `os.run('echo', 'abc', nil, 'def')` will invoke `echo` with only 2 arguments).

## Exit code

Project will exit with exit code 0 when all operations completed successfully. The script can also return a custom exit code:

```lua
return 5
```

Will cause Project to exit with exit code 5. The code 100 and above are reserved for Project use and have the following meaning:

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

### 107

Return value from Lua script is not either nil or integer.

### 108

Return value from Lua script is integer outside 0 - 99.

## License

This project is licensed under either of

- Apache License, Version 2.0,
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
