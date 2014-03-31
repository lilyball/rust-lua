#![allow(uppercase_variables)]

extern crate lua;

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    match L.loadfile(None) {
        Ok(()) => (),
        Err(lua::LoadFileError::ErrSyntax) => fail!("syntax error"),
        Err(lua::LoadFileError::ErrMem) => fail!("memory allocation error"),
        Err(lua::LoadFileError::ErrFile) => fail!("file error (?!?)")
    }
    L.call(0, 0);
}
