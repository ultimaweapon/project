use std::path::PathBuf;

use flate2::read::MultiGzDecoder;

fn main() {
    // Download and extract C++ dependencies.
    ensure_dep("lua-5.4.7", "https://www.lua.org/ftp/lua-5.4.7.tar.gz");

    // Build C++ sources.
    cc::Build::new()
        .cpp(true)
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
