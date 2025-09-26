
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("rusty-audio");

    let profile = env::var("PROFILE").unwrap();
    let target_dir = if profile == "release" {
        "target/release"
    } else {
        "target/debug"
    };

    let target_path = PathBuf::from(target_dir);
    if !target_path.exists() {
        fs::create_dir_all(&target_path).unwrap();
    }

    let executable_path = env::current_exe().unwrap();
    let dest_file = target_path.join(executable_path.file_name().unwrap());

    if dest_path.exists() {
        fs::copy(&dest_path, &dest_file).unwrap();
    }
}
