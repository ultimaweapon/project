use std::path::PathBuf;

use flate2::read::MultiGzDecoder;

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Download and extract C++ dependencies.
    ensure_dep("lua-5.4.7", "https://www.lua.org/ftp/lua-5.4.7.tar.gz");

    // Build Lua.
    let lua = PathBuf::from_iter(["deps", "lua-5.4.7", "src"]);
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

    cc.cpp(true); // Use C++ exception instead of setjmp/longjmp for error/yield.

    if cc.get_compiler().is_like_msvc() {
        cc.flag("/TP"); // cc does not do this for us
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

    cc.compile("lua");

    // Build C++ sources.
    cc::Build::new()
        .cpp(true)
        .include(lua)
        .file(PathBuf::from_iter(["src", "script", "engine.cpp"]))
        .compile("project");
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
