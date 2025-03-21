# Project

Project is a cross-platform CLI program to execute commands defined in the `Project.yml`. Each command is a Lua script. The main different from [just](https://github.com/casey/just) is Project focus on cross-platform scripting instead of rely on the other tools.

## Key features

- Batteries included.
- Small and lightweight.
- Easy to install on all platforms.
- Single executable with only system dependencies.
- Lua 5.4 **without** compatibility with the previous version as scripting language.

## Script API

Lua standard libraries are available except `debug` and `package`. The `os` library also has `exit` and `setlocale` removed.

### os.arch()

Returns architecture of the OS. The value will be one of `aarch64` and `x86_64`.

### os.kind()

Returns kind of the OS. The value will be one of `linux`, `macos` and `windows`.

### os.run(prog [, ...])

Run `prog` with the remaining arguments as its arguments. Unlike `os.execute`, this does not use OS shell to run `prog`. This function will raise an error by default if `prog` exit with non-zero code. By default, all standard streams will be inherits from `project` process and working directory will be the directory that contains `Project.yml`.

If `prog` is not an absolute path, the `PATH` will be searched in an OS-defined way.

All `nil` in the arguments will be removed (e.g. `os.run('echo', 'abc', nil, 'def')` will invoke `echo` with only 2 arguments).

## License

This project is licensed under either of

- Apache License, Version 2.0,
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
