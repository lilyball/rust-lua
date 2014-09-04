// Sample code that shows how to traverse a table.
// We'll use it to traverse the global table

#![allow(non_snake_case)]

extern crate lua;

fn main() {
    let mut L = lua::State::new();
    L.open_base();
    L.pushnil(); // push the first key
    while L.next(lua::GLOBALSINDEX) {
        println!("{} - {}", L.describe(-2), L.typename(-1));
        L.pop(1); // remove the value, keep the key
    }
}
