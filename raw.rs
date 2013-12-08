use std::libc;
use std::libc::c_int;
use std::ptr;

/// Type of numbers in Lua.
/// Assumes the installed liblua hasn't changed the default.
pub type lua_Number = f64;

/// Type for integer functions
pub type lua_Integer = libc::ptrdiff_t;

pub static MULTRET: c_int = -1;

pub static LUA_REGISTRYINDEX: c_int = -10000;
pub static LUA_ENVIRONINDEX: c_int = -10001;
pub static LUA_GLOBALSINDEX: c_int = -10002;
#[inline(always)]
pub fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}

// Thread statuses
pub static LUA_YIELD: c_int = 1;
pub static LUA_ERRRUN: c_int = 2;
pub static LUA_ERRSYNTAX: c_int = 3;
pub static LUA_ERRMEM: c_int = 4;
pub static LUA_ERRERR: c_int = 5;

// Basic types
pub static LUA_TNONE:          c_int = -1;

pub static LUA_TNIL:           c_int = 0;
pub static LUA_TBOOLEAN:       c_int = 1;
pub static LUA_TLIGHTUSERDATA: c_int = 2;
pub static LUA_TNUMBER:        c_int = 3;
pub static LUA_TSTRING:        c_int = 4;
pub static LUA_TTABLE:         c_int = 5;
pub static LUA_TFUNCTION:      c_int = 6;
pub static LUA_TUSERDATA:      c_int = 7;
pub static LUA_TTHREAD:        c_int = 8;

pub type lua_State = libc::c_void;

pub type lua_CFunction = extern "C" unsafe fn(L: *mut lua_State) -> c_int;

/// Function type for reading blocks when loading Lua chunks.
pub type lua_Reader = extern "C" fn(L: *mut lua_State, ud: *mut libc::c_void,
                                    sz: *mut libc::size_t) -> *libc::c_char;
/// Function type for writing blocks when dumping Lua chunks.
pub type lua_Writer = extern "C" fn(L: *mut lua_State, p: *libc::c_void, sz: libc::size_t,
                                    ud: *mut libc::c_void) -> libc::c_int;

/// Prototype for memory-allocation functions
pub type lua_Alloc = extern "C" fn(ud: *mut libc::c_void, ptr: *mut libc::c_void,
                                osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void;

// lua_State manipulation
extern {
    pub fn lua_newstate(f: lua_Alloc, ud: *mut libc::c_void) -> *mut lua_State;
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_newthread(L: *mut lua_State) -> *mut lua_State;

    pub fn lua_atpanic(L: *mut lua_State, panicf: lua_CFunction) -> lua_CFunction;
}

// Basic stack manipulation
extern {
    pub fn lua_gettop(L: *mut lua_State) -> c_int;
    pub fn lua_settop(L: *mut lua_State, idx: c_int);
    pub fn lua_pushvalue(L: *mut lua_State, idx: c_int);
    pub fn lua_remove(L: *mut lua_State, idx: c_int);
    pub fn lua_insert(L: *mut lua_State, idx: c_int);
    pub fn lua_replace(L: *mut lua_State, idx: c_int);
    pub fn lua_checkstack(L: *mut lua_State, sz: c_int) -> c_int;

    pub fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int);
}

// Access functions (stack -> C)
extern {
    pub fn lua_isnumber(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_isstring(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_iscfunction(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_isuserdata(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_type(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_typename(L: *mut lua_State, tp: c_int) -> *libc::c_char;

    pub fn lua_equal(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;
    pub fn lua_rawequal(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;
    pub fn lua_lessthan(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;

    pub fn lua_tonumber(L: *mut lua_State, idx: c_int) -> lua_Number;
    pub fn lua_tointeger(L: *mut lua_State, idx: c_int) -> lua_Integer;
    pub fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int,
                            len: *mut libc::size_t) -> *libc::c_char;
    pub fn lua_objlen(L: *mut lua_State, idx: c_int) -> libc::size_t;
    pub fn lua_tocfunction(L: *mut lua_State, idx: c_int) -> Option<lua_CFunction>;
    pub fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut libc::c_void;
    pub fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State;
    pub fn lua_topointer(L: *mut lua_State, idx: c_int) -> *libc::c_void;
}

// Push functions (C -> stack)
extern {
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlstring(L: *mut lua_State, s: *libc::c_char, l: libc::size_t);
    pub fn lua_pushstring(L: *mut lua_State, s: *libc::c_char);
    // lua_pushvfstring() .. can't represent va_list
    pub fn lua_pushfstring(L: *mut lua_State, fmt: *libc::c_char, ...) -> *libc::c_char;
    pub fn lua_pushcclosure(L: *mut lua_State, f: lua_CFunction, n: c_int);
    pub fn lua_pushboolean(L: *mut lua_State, b: c_int);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut libc::c_void);
    pub fn lua_pushthread(L: *mut lua_State) -> c_int;
}

// Get functions (Lua -> stack)
extern {
    pub fn lua_gettable(L: *mut lua_State, idx: c_int);
    pub fn lua_getfield(L: *mut lua_State, idx: c_int, k: *libc::c_char);
    pub fn lua_rawget(L: *mut lua_State, idx: c_int);
    pub fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: c_int);
    pub fn lua_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_newuserdata(L: *mut lua_State, sz: libc::size_t) -> *mut libc::c_void;
    pub fn lua_getmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_getfenv(L: *mut lua_State, idx: c_int);
}

// Set functions (stack -> Lua)
extern {
    pub fn lua_settable(L: *mut lua_State, idx: c_int);
    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *libc::c_char);
    pub fn lua_rawset(L: *mut lua_State, idx: c_int);
    pub fn lua_rawseti(L: *mut lua_State, idx: c_int, n: c_int);
    pub fn lua_setmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_setfenv(L: *mut lua_State, idx: c_int) -> c_int;
}

// `load` and `call` functions (load and run Lua code)
extern {
    pub fn lua_call(L: *mut lua_State, nargs: c_int, nresults: c_int);
    pub fn lua_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int;
    pub fn lua_cpcall(L: *mut lua_State, func: lua_CFunction, ud: *mut libc::c_void) -> c_int;
    pub fn lua_load(L: *mut lua_State, reader: lua_Reader, dt: *mut libc::c_void,
                    chunkname: *libc::c_char) -> c_int;

    pub fn lua_dump(L: *mut lua_State, writer: lua_Writer, data: *mut libc::c_void) -> c_int;
}

// Coroutine functions
extern {
    pub fn lua_yield(L: *mut lua_State, nresults: c_int) -> c_int;
    pub fn lua_resume(L: *mut lua_State, narg: c_int) -> c_int;
    pub fn lua_status(L: *mut lua_State) -> c_int;
}

// Garbage-collection function and options
pub static LUA_GCSTOP:       c_int = 0;
pub static LUA_GCRESTART:    c_int = 1;
pub static LUA_GCCOLLECT:    c_int = 2;
pub static LUA_GCCOUNT:      c_int = 3;
pub static LUA_GCCOUNTB:     c_int = 4;
pub static LUA_GCSTEP:       c_int = 5;
pub static LUA_GCSETPAUSE:   c_int = 6;
pub static LUA_GCSETSTEPMUL: c_int = 7;

extern {
    pub fn lua_gc(L: *mut lua_State, what: c_int, data: c_int) -> c_int;
}

// Miscellaneous functions
extern {
    pub fn lua_error(L: *mut lua_State) -> c_int;

    pub fn lua_next(L: *mut lua_State, idx: c_int) -> c_int;

    pub fn lua_concat(L: *mut lua_State, n: c_int);

    pub fn lua_getallocf(L: *mut lua_State, ud: *mut *mut libc::c_void) -> lua_Alloc;
    pub fn lua_setallocf(L: *mut lua_State, f: lua_Alloc, ud: *mut libc::c_void);
}

// Some useful functions (macros in C)
#[inline(always)]
pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    lua_settop(L, -n-1)
}

#[inline(always)]
pub unsafe fn lua_newtable(L: *mut lua_State) {
    lua_createtable(L, 0, 0)
}

#[inline(always)]
pub unsafe fn lua_register(L: *mut lua_State, name: *libc::c_char, f: lua_CFunction) {
    lua_pushcfunction(L, f);
    lua_setglobal(L, name)
}

#[inline(always)]
pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: lua_CFunction) {
    lua_pushcclosure(L, f, 0)
}

#[inline(always)]
pub unsafe fn lua_strlen(L: *mut lua_State, i: c_int) -> libc::size_t {
    lua_objlen(L, i)
}

#[inline(always)]
pub unsafe fn lua_isfunction(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TFUNCTION
}

#[inline(always)]
pub unsafe fn lua_istable(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TTABLE
}

#[inline(always)]
pub unsafe fn lua_islightuserdata(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TLIGHTUSERDATA
}

#[inline(always)]
pub unsafe fn lua_isnil(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TNIL
}

#[inline(always)]
pub unsafe fn lua_isboolean(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TBOOLEAN
}

#[inline(always)]
pub unsafe fn lua_isthread(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TTHREAD
}

#[inline(always)]
pub unsafe fn lua_isnone(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TNONE
}

#[inline(always)]
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) <= 0
}

// fn lua_pushliteral: can't represent this in Rust

#[inline(always)]
pub unsafe fn lua_setglobal(L: *mut lua_State, s: *libc::c_char) {
    lua_setfield(L, LUA_GLOBALSINDEX, s)
}

#[inline(always)]
pub unsafe fn lua_getglobal(L: *mut lua_State, s: *libc::c_char) {
    lua_getfield(L, LUA_GLOBALSINDEX, s)
}

#[inline(always)]
pub unsafe fn lua_tostring(L: *mut lua_State, i: c_int) -> *libc::c_char {
    lua_tolstring(L, i, ptr::mut_null())
}

// Hack
extern {
    pub fn lua_setlevel(from: *mut lua_State, to: *mut lua_State);
}

#[cfg(test)]
#[path = "rawtests.rs"]
mod tests;
