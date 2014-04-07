pub mod raw {
    use libc::c_int;
    use raw::lua_State;

    pub static LUA_COLIBNAME: &'static str = "coroutine";
    pub static LUA_TABLIBNAME: &'static str = "table";
    pub static LUA_IOLIBNAME: &'static str = "io";
    pub static LUA_OSLIBNAME: &'static str = "os";
    pub static LUA_STRLIBNAME: &'static str = "string";
    pub static LUA_MATHLIBNAME: &'static str = "math";
    pub static LUA_DBLIBNAME: &'static str = "debug";
    pub static LUA_LOADLIBNAME: &'static str = "package";

    extern {
        pub fn luaopen_base(L: *mut lua_State) -> c_int;
        pub fn luaopen_table(L: *mut lua_State) -> c_int;
        pub fn luaopen_io(L: *mut lua_State) -> c_int;
        pub fn luaopen_os(L: *mut lua_State) -> c_int;
        pub fn luaopen_string(L: *mut lua_State) -> c_int;
        pub fn luaopen_math(L: *mut lua_State) -> c_int;
        pub fn luaopen_debug(L: *mut lua_State) -> c_int;
        pub fn luaopen_package(L: *mut lua_State) -> c_int;

        pub fn luaL_openlibs(L: *mut lua_State);
    }
}
