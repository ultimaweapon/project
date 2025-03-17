use std::path::MAIN_SEPARATOR_STR;

fn main() {
    // Build C++ sources.
    let mut cc = cc::Build::new();
    let sources = [["src", "script", "engine.cpp"].as_slice()];

    cc.cpp(true)
        .std("c++17")
        .include(std::env::var_os("DEP_LUA_INCLUDE_PATH").unwrap());

    for src in sources {
        let path = src.join(MAIN_SEPARATOR_STR);

        println!("cargo::rerun-if-changed={path}");

        cc.file(path);
    }

    cc.compile("project");
}
