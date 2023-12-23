use std::path::Path;
use std::process::Command;

fn main() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Update and init submodule
    if let Err(error) = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .current_dir(src_dir)
        .status()
    {
        eprintln!("{error}");
    }
}
