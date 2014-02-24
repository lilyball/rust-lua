extern crate lua;

use std::{libc, num};

mod common;

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    L.register("sin", my_sin);
    common::repl(&mut L);
}

extern "C" fn my_sin(L: *mut lua::raw::lua_State) -> libc::c_int {
    unsafe {
        let mut L = lua::ExternState::from_lua_State(L);
        let input = L.checknumber(1);
        let output = num::sin(input);
        L.pushnumber(output);
        1
    }
}
