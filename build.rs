#![allow(unstable)]

use std::io::Command;
use std::io::process::{InheritFd, Ignored};

fn main() {
    let mut cmd = Command::new("make");
    cmd.arg("cargo-prep");
    println!("running: {}", cmd);
    assert!(cmd.stdin(Ignored)
               .stdout(InheritFd(1))
               .stderr(InheritFd(2))
               .status()
               .unwrap()
               .success());
}
