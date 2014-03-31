// Simple API example, ported from http://lua-users.org/wiki/SimpleLuaApiExample

// This is a simple introductory example of how to interface to Lua from Rust.
// The Rust program loads a Lua script file, sets some Lua variables, runs the
// Lua script, and reads back the return value.

#![allow(uppercase_variables)]

extern crate lua;

use std::{io, os};
use std::iter::range_inclusive;

fn main() {
    let mut L = lua::State::new();
    L.openlibs(); // Load Lua libraries

    // Load the file containing the script we are going to run
    let path = Path::new("simpleapi.lua");
    match L.loadfile(Some(&path)) {
        Ok(_) => (),
        Err(_) => {
            // If something went wrong, error message is at the top of the stack
            let _ = writeln!(&mut io::stderr(),
                             "Couldn't load file: {}", L.describe(-1));
            os::set_exit_status(1);
            return;
        }
    }

    /*
     * Ok, now here we go: We pass data to the lua script on the stack.
     * That is, we first have to prepare Lua's virtual stack the way we
     * want the script to receive it, then ask Lua to run it.
     */
    L.newtable(); // We will pass a table

    /*
     * To put values into the table, we first push the index, then the
     * value, and then call rawset() with the index of the table in the
     * stack. Let's see why it's -3: In Lua, the value -1 always refers to
     * the top of the stack. When you create the table with newtable(),
     * the table gets pushed into the top of the stack. When you push the
     * index and then the cell value, the stack looks like:
     *
     * - [stack bottom] -- table, index, value [top]
     *
     * So the -1 will refer to the cell value, thus -3 is used to refer to
     * the table itself. Note that rawset() pops the last two elements
     * of the stack, so that after it has been called, the table is at the
     * top of the stack.
     */
    for i in range_inclusive(1, 5) {
        L.pushinteger(i);   // Push the table index
        L.pushinteger(i*2); // Push the cell value
        L.rawset(-3);       // Stores the pair in the table
    }

    // By what name is the script going to reference our table?
    L.setglobal("foo");

    // Ask Lua to run our little script
    match L.pcall(0, lua::MULTRET, 0) {
        Ok(()) => (),
        Err(_) => {
            let _ = writeln!(&mut io::stderr(),
                             "Failed to run script: {}", L.describe(-1));
            os::set_exit_status(1);
            return;
        }
    }

    // Get the returned value at the to of the stack (index -1)
    let sum = L.tonumber(-1);

    println!("Script returned: {}", sum);

    L.pop(1); // Take the returned value out of the stack

    // L's destructor will close the state for us
}
