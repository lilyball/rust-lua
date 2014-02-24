/// Macro for defining Lua CFunctions

#[macro_export]
macro_rules! lua_extern {
    ($(unsafe fn $name:ident($arg:ident: $typ:ty) -> i32 $code:block)+) => (
        $(
            extern "C" fn $name($arg: *mut ::lua::raw::lua_State) -> ::std::libc::c_int {
                unsafe {
                    let mut $arg = ::lua::ExternState::from_lua_State($arg);
                    return inner(&mut $arg) as ::std::libc::c_int;
                }

                unsafe fn inner($arg: $typ) -> i32 $code
            }
        )+
    )
}
