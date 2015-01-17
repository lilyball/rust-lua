//! Lua Auxilliary Library

pub mod raw {
    use libc;
    use libc::c_int;
    use std::ptr;

    use raw;
    use raw::{lua_State, lua_CFunction, lua_Number, lua_Integer};
    use raw::{MULTRET, LUA_REGISTRYINDEX, LUA_ERRERR};
    use config;

    // don't bother defining luaL_getn and luaL_setn. They're obsolete functions

    // Don't define luaL_openlib either. That's also obsolete.

    // Extra error code for `luaL_load`
    pub const LUA_ERRFILE: c_int = LUA_ERRERR+1;

    pub const LUAL_BUFFERSIZE: libc::size_t = config::LUAL_BUFFERSIZE;

    pub const LUA_NOREF: c_int = -2;
    pub const LUA_REFNIL: c_int = -1;

    #[allow(non_camel_case_types)]
    #[repr(C)]
    #[allow(missing_copy_implementations)]
    pub struct luaL_Reg {
        pub name: *const libc::c_char,
        pub func: Option<lua_CFunction>
    }

    extern {
        pub fn luaL_register(L: *mut lua_State, libname: *const libc::c_char, l: *const luaL_Reg);
        pub fn luaL_getmetafield(L: *mut lua_State, obj: c_int, e: *const libc::c_char) -> c_int;
        pub fn luaL_callmeta(L: *mut lua_State, obj: c_int, e: *const libc::c_char) -> c_int;
        pub fn luaL_typerror(L: *mut lua_State, narg: c_int, tname: *const libc::c_char) -> c_int;
        pub fn luaL_argerror(L: *mut lua_State, numarg: c_int, extramsg: *const libc::c_char) -> c_int;
        pub fn luaL_checklstring(L: *mut lua_State, numArg: c_int,
                                 l: *mut libc::size_t) -> *const libc::c_char;
        pub fn luaL_optlstring(L: *mut lua_State, numArg: c_int, def: *const libc::c_char,
                               l: *mut libc::size_t) -> *const libc::c_char;
        pub fn luaL_checknumber(L: *mut lua_State, numArg: c_int) -> lua_Number;
        pub fn luaL_optnumber(L: *mut lua_State, nArg: c_int, def: lua_Number) -> lua_Number;

        pub fn luaL_checkinteger(L: *mut lua_State, numArg: c_int) -> lua_Integer;
        pub fn luaL_optinteger(L: *mut lua_State, numArg: c_int, def: lua_Integer) -> lua_Integer;

        pub fn luaL_checkstack(L: *mut lua_State, sz: c_int, msg: *const libc::c_char);
        pub fn luaL_checktype(L: *mut lua_State, narg: c_int, t: c_int);
        pub fn luaL_checkany(L: *mut lua_State, narg: c_int);

        pub fn luaL_newmetatable(L: *mut lua_State, tname: *const libc::c_char) -> c_int;
        pub fn luaL_checkudata(L: *mut lua_State, ud: c_int, tname: *const libc::c_char)
                              -> *mut libc::c_void;

        pub fn luaL_where(L: *mut lua_State, lvl: c_int);
        pub fn luaL_error(L: *mut lua_State, fmt: *const libc::c_char, ...) -> c_int;

        pub fn luaL_checkoption(L: *mut lua_State, narg: c_int, def: *const libc::c_char,
                                lst: *const *const libc::c_char) -> c_int;

        pub fn luaL_ref(L: *mut lua_State, t: c_int) -> c_int;
        pub fn luaL_unref(L: *mut lua_State, t: c_int, refid: c_int);

        pub fn luaL_loadfile(L: *mut lua_State, filename: *const libc::c_char) -> c_int;
        pub fn luaL_loadbuffer(L: *mut lua_State, buff: *const libc::c_char, sz: libc::size_t,
                               name: *const libc::c_char) -> c_int;
        pub fn luaL_loadstring(L: *mut lua_State, s: *const libc::c_char) -> c_int;

        pub fn luaL_newstate() -> *mut lua_State;

        pub fn luaL_gsub(L: *mut lua_State, s: *const libc::c_char, p: *const libc::c_char, r: *const libc::c_char)
                        -> *const libc::c_char;
    }

    // Some useful functions (macros in C)
    #[inline(always)]
    pub unsafe fn luaL_argcheck(L: *mut lua_State, cond: bool, numarg: c_int, extramsg: *const libc::c_char) {
        if !cond {
            luaL_argerror(L, numarg, extramsg);
        }
    }

    #[inline(always)]
    pub unsafe fn luaL_checkstring(L: *mut lua_State, n: c_int) -> *const libc::c_char {
        luaL_checklstring(L, n, ptr::null_mut())
    }

    #[inline(always)]
    pub unsafe fn luaL_optstring(L: *mut lua_State, n: c_int, d: *const libc::c_char) -> *const libc::c_char {
        luaL_optlstring(L, n, d, ptr::null_mut())
    }

    #[inline(always)]
    pub unsafe fn luaL_checkint(L: *mut lua_State, n: c_int) -> c_int {
        luaL_checkinteger(L, n) as c_int
    }

    #[inline(always)]
    pub unsafe fn luaL_optint(L: *mut lua_State, n: c_int, d: c_int) -> c_int {
        luaL_optinteger(L, n, d as lua_Integer) as c_int
    }

    #[inline(always)]
    pub unsafe fn luaL_checklong(L: *mut lua_State, n: c_int) -> libc::c_long {
        luaL_checkinteger(L, n) as libc::c_long
    }

    #[inline(always)]
    pub unsafe fn luaL_optlong(L: *mut lua_State, n: c_int, d: libc::c_long) -> libc::c_long {
        luaL_optinteger(L, n, d as lua_Integer) as libc::c_long
    }

    #[inline(always)]
    pub unsafe fn luaL_typename(L: *mut lua_State, i: c_int) -> *const libc::c_char {
        raw::lua_typename(L, raw::lua_type(L, i))
    }

    #[inline(always)]
    pub unsafe fn luaL_dofile(L: *mut lua_State, filename: *const libc::c_char) -> c_int {
        ((luaL_loadfile(L, filename) != 0) || (raw::lua_pcall(L, 0, MULTRET, 0) != 0)) as c_int
    }

    #[inline(always)]
    pub unsafe fn luaL_dostring(L: *mut lua_State, s: *const libc::c_char) -> c_int {
        ((luaL_loadstring(L, s) != 0) || (raw::lua_pcall(L, 0, MULTRET, 0) != 0)) as c_int
    }

    #[inline(always)]
    pub unsafe fn luaL_getmetatable(L: *mut lua_State, name: *const libc::c_char) {
        raw::lua_getfield(L, LUA_REGISTRYINDEX, name)
    }

    #[inline(always)]
    pub unsafe fn luaL_opt<T, F>(L: *mut lua_State, f: F, n: c_int, d: T) -> T
        where F: FnOnce(*mut lua_State, c_int) -> T
    {
        if raw::lua_isnoneornil(L, n) {
            d
        } else {
            f(L, n)
        }
    }

    // Generic Buffer manipulation

    #[allow(non_camel_case_types,missing_copy_implementations)]
    #[repr(C)]
    pub struct luaL_Buffer {
        pub p: *mut libc::c_char, // current position in buffer
        pub lvl: c_int, // number of strings in the stack (level)
        pub L: *mut lua_State,
        pub buffer: [libc::c_char; LUAL_BUFFERSIZE as usize]
    }

    #[inline(always)]
    pub unsafe fn luaL_addchar(B: *mut luaL_Buffer, c: libc::c_char) {
        let startp: *mut libc::c_char = &mut (*B).buffer[0];
        if (*B).p >= startp.offset(LUAL_BUFFERSIZE as isize) {
            luaL_prepbuffer(B);
        }
        *(*B).p = c;
        (*B).p = (*B).p.offset(1);
    }

    // skip luaL_putchar

    #[inline(always)]
    pub unsafe fn luaL_addsize(B: *mut luaL_Buffer, n: libc::size_t) {
        (*B).p = (*B).p.offset(n as isize);
    }

    extern {
        pub fn luaL_buffinit(L: *mut lua_State, B: *mut luaL_Buffer);
        pub fn luaL_prepbuffer(B: *mut luaL_Buffer) -> *mut libc::c_char;
        pub fn luaL_addlstring(B: *mut luaL_Buffer, s: *const libc::c_char, l: libc::size_t);
        pub fn luaL_addstring(B: *mut luaL_Buffer, s: *const libc::c_char);
        pub fn luaL_addvalue(B: *mut luaL_Buffer);
        pub fn luaL_pushresult(B: *mut luaL_Buffer);
    }

    // Omit lua_ref compatibility macros. They're undocumented in 5.1 and replaced by luaL_ref.
}
