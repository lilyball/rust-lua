#![feature(process)]

use std::process::{Command,Stdio};

fn main() {
    let mut cmd = Command::new("make");
    cmd.arg("cargo-prep");
    println!("running: {:?}", cmd);
    assert!(cmd.stdin(Stdio::null())
               .status()
               .unwrap()
               .success());
}
