#![allow(non_snake_case)]

extern crate lua;

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    match L.loadfile(None) {
        Ok(()) => (),
        Err(lua::LoadFileError::ErrSyntax) => panic!("syntax error"),
        Err(lua::LoadFileError::ErrMem) => panic!("memory allocation error"),
        Err(lua::LoadFileError::ErrFile) => panic!("file error (?!?)")
    }
    L.call(0, 0);
}
