#![allow(non_snake_case,unstable)]

#[macro_use]
extern crate lua;
extern crate libc;

use std::io;
use std::io::BufferedReader;
use std::num::Float;

pub fn repl(L: &mut lua::State) {
    let mut stdin = BufferedReader::new(io::stdin());
    let stdout = &mut io::stdout() as &mut io::Writer;
    let stderr = &mut io::stderr() as &mut io::Writer;
    loop {
        L.settop(0); // clear the stack
        let _ = write!(stdout, "> ");
        let _ = stdout.flush();
        let mut line = match stdin.read_line() {
            Ok(line) => line,
            Err(_) => break
        };
        if line.as_slice().starts_with("=") {
            line = format!("return {}", line.as_slice().slice_from(1));
        }
        match L.loadbuffer(line.as_slice(), "=stdin") {
            Ok(_) => (),
            Err(err) => { let _ = writeln!(stderr, "{:?}", err); continue; }
        }
        match L.pcall(0, lua::MULTRET, 0) {
            Ok(_) => (),
            Err(_) => {
                match L.tostring(-1) {
                    Some(msg) => { let _ = writeln!(stderr, "{}", msg); }
                    None => { let _ = writeln!(stderr, "(error object is not a string)"); }
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
                    let _ = writeln!(stderr, "error calling 'print' ({})", L.describe(-1));
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
