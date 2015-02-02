#![feature(io)]

use std::old_io::Command;
use std::old_io::process::{InheritFd, Ignored};

fn main() {
    let mut cmd = Command::new("make");
    cmd.arg("cargo-prep");
    println!("running: {:?}", cmd);
    assert!(cmd.stdin(Ignored)
               .stdout(InheritFd(1))
               .stderr(InheritFd(2))
               .status()
               .unwrap()
               .success());
}
