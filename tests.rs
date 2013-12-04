use std::{libc, ptr};
use raw;
use aux;

// implement the same function that luaL_newstate uses, so we can test lua_newstate directly
// FIXME (#10025): We can't define this as `unsafe extern "C"`
extern "C" fn test_alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
                         nsize: libc::size_t) -> *mut libc::c_void {
    unsafe {
        if nsize == 0 {
            libc::free(ptr as *libc::c_void);
            ptr::mut_null()
        } else {
            libc::realloc(ptr, nsize)
        }
    }
}

#[test]
fn test_lua_newstate() {
    unsafe {
        let s = raw::lua_newstate(test_alloc, ptr::mut_null());
        raw::lua_pushinteger(s, 42);
        raw::lua_close(s);
    }
}

#[test]
fn test_luaL_newstate() {
    unsafe {
        let s = aux::raw::luaL_newstate();
        raw::lua_pushinteger(s, 42);
        raw::lua_close(s);
    }
}
