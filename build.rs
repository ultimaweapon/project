use std::path::{MAIN_SEPARATOR_STR, PathBuf};

use flate2::read::MultiGzDecoder;

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Download and extract C++ dependencies.
    ensure_dep("lua-5.4.7", "https://www.lua.org/ftp/lua-5.4.7.tar.gz");

    // Build C++ sources.
    let lua = PathBuf::from_iter(["deps", "lua-5.4.7", "src"]);
    let mut cc = cc::Build::new();
    let sources = [["src", "script", "engine.cpp"].as_slice()];

    cc.cpp(true).include(&lua);

    for src in sources {
        let path = src.join(MAIN_SEPARATOR_STR);

        println!("cargo::rerun-if-changed={path}");

        cc.file(path);
    }

    cc.compile("project");

    drop(cc);

    // Build Lua.
    let mut cc = cc::Build::new();
    let sources = [
        "lapi.c",
        "lcode.c",
        "lctype.c",
        "ldebug.c",
        "ldo.c",
        "ldump.c",
        "lfunc.c",
        "lgc.c",
        "llex.c",
        "lmem.c",
        "lobject.c",
        "lopcodes.c",
        "lparser.c",
        "lstate.c",
        "lstring.c",
        "ltable.c",
        "ltm.c",
        "lundump.c",
        "lvm.c",
        "lzio.c",
        "lauxlib.c",
        "lbaselib.c",
        "lcorolib.c",
        "ldblib.c",
        "liolib.c",
        "lmathlib.c",
        "loadlib.c",
        "loslib.c",
        "lstrlib.c",
        "ltablib.c",
        "lutf8lib.c",
        "linit.c",
    ];

    // Use C++ exception instead of setjmp/longjmp for error/yield.
    cc.cpp(true);

    if cc.get_compiler().is_like_msvc() {
        cc.flag("/TP");
    } else {
        cc.flag("-xc++");
    }

    match os.as_str() {
        "linux" => cc.define("LUA_USE_LINUX", None),
        "macos" => cc.define("LUA_USE_MACOSX", None),
        "windows" => &mut cc,
        _ => panic!("target OS is not supported"),
    };

    for src in sources {
        cc.file(lua.join(src));
    }

    cc.compile("lua-vendored"); // Avoid attempt to link with system Lua.
}

fn ensure_dep(dir: &str, tar: &str) {
    // Do nothing if directory for the dependency already exists.
    let path = PathBuf::from_iter(["deps", dir]);

    if path.exists() {
        return;
    }

    // Download source.
    let tar = ureq::get(tar).call().unwrap().into_body().into_reader();
    let tar = MultiGzDecoder::new(tar);
    let mut tar = tar::Archive::new(tar);

    tar.unpack("deps").unwrap();
}
