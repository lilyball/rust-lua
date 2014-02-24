#[feature(phase)];

#[phase(syntax,link)]
extern crate lua;

use std::num;

mod common;

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    L.register("sin", my_sin);
    L.register("cos", my_cos);
    common::repl(&mut L);
}

lua_extern! {
    fn my_sin(L: &mut lua::ExternState) -> i32 {
        unsafe {
            let input = L.checknumber(1);
            let output = num::sin(input);
            L.pushnumber(output);
            1
        }
    }

    fn my_cos(L: &mut lua::ExternState) -> i32 {
        unsafe {
            let input = L.checknumber(1);
            let output = num::cos(input);
            L.pushnumber(output);
            1
        }
    }
}
