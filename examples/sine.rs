#![allow(non_snake_case)]
#![allow(trivial_numeric_casts)] // FIXME: rust-lang/rfcs#1020

#[macro_use]
extern crate lua;
extern crate libc;

use std::io;
use std::io::prelude::*;

pub fn repl(L: &mut lua::State) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut line = String::new();
    loop {
        L.settop(0); // clear the stack
        let _ = write!(&mut stdout, "> ");
        let _ = stdout.flush();
        line.clear();
        if let Err(_) = stdin.read_line(&mut line) {
            break
        }
        if line.starts_with("=") {
            line = format!("return {}", &line[1..]);
        }
        match L.loadbuffer(&line, "=stdin") {
            Ok(_) => (),
            Err(err) => { let _ = writeln!(&mut stderr, "{:?}", err); continue; }
        }
        match L.pcall(0, lua::MULTRET, 0) {
            Ok(_) => (),
            Err(_) => {
                match L.tostring(-1) {
                    Some(msg) => { let _ = writeln!(&mut stderr, "{}", msg); }
                    None => { let _ = writeln!(&mut stderr, "(error object is not a string)"); }
                }
            }
        }
        if L.gettop() > 0 {
            L.getglobal("print");
            L.insert(1);
            let nargs = L.gettop()-1;
            match L.pcall(nargs, 0, 0) {
                Ok(_) => (),
                Err(_) => {
                    let _ = writeln!(&mut stderr, "error calling 'print' ({})", L.describe(-1));
                    continue;
                }
            }
        }
    }
}

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    L.register("sin", my_sin);
    L.register("cos", my_cos);
    L.register("tan", my_tan);
    repl(&mut L);
}

lua_extern! {
    unsafe fn my_sin(L: &mut lua::ExternState) -> i32 {
        let input = L.checknumber(1);
        let output = input.sin();
        L.pushnumber(output);
        1
    }

    unsafe fn my_cos(L: &mut lua::ExternState) -> i32 {
        let input = L.checknumber(1);
        let output = input.cos();
        L.pushnumber(output);
        1
    }
}

lua_extern_pub! {
    // this function is marked public
    unsafe fn my_tan(L: &mut lua::ExternState) -> i32 {
        let input = L.checknumber(1);
        let output = input.tan();
        L.pushnumber(output);
        1
    }
}
