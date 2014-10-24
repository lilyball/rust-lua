pub mod raw {
    use libc::c_int;
    use raw::lua_State;

    pub const LUA_COLIBNAME: &'static str = "coroutine";
    pub const LUA_TABLIBNAME: &'static str = "table";
    pub const LUA_IOLIBNAME: &'static str = "io";
    pub const LUA_OSLIBNAME: &'static str = "os";
    pub const LUA_STRLIBNAME: &'static str = "string";
    pub const LUA_MATHLIBNAME: &'static str = "math";
    pub const LUA_DBLIBNAME: &'static str = "debug";
    pub const LUA_LOADLIBNAME: &'static str = "package";

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
