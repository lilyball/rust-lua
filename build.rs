use std::io::Command;

fn main() {
    match Command::new("make").arg("cargo-prep").spawn() {
        Ok(p) => p,
        Err(e) => panic!("failed to build config.rs: {}", e),
    };
}
