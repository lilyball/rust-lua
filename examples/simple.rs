extern mod lua;

fn main() {
    let mut L = lua::State::new();
    L.openlibs();
    match L.loadfile(None) {
        None => (),
        Some(lua::LoadFileError::ErrSyntax) => fail!("syntax error"),
        Some(lua::LoadFileError::ErrMem) => fail!("memory allocation error"),
        Some(lua::LoadFileError::ErrFile) => fail!("file error (?!?)")
    }
    L.call(0, 0);
}
