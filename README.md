# Project

Project is a cross-platform CLI program to execute commands defined in the `Project.yml`. Each command can be Lua script or other program. The main different from [just](https://github.com/casey/just) is Project focus on cross-platform scripting instead of rely on the other tools.

## Key features

- Batteries included.
- Small and lightweight.
- Easy to install on all platforms.
- Single executable with only system dependencies.
- Lua as scripting language.

## Script API

Lua standard libraries are available except `debug` and `package`. The `os` library also has `exit` and `setlocale` removed.

### os.arch()

Returns architecture of the OS as a string. The value will be one of `aarch64` and `x86_64`.

### os.kind()

Returns kind of the OS as a string. The value will be one of `linux`, `macos` and `windows`.

## License

This project is licensed under either of

- Apache License, Version 2.0,
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
