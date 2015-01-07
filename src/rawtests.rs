use libc;
use std::ptr;
use std::ffi::CString;
use raw;
use aux;

// implement the same function that luaL_newstate uses, so we can test lua_newstate directly
// FIXME (#10025): We can't define this as `unsafe extern "C"`
extern "C" fn alloc_helper(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
                         nsize: libc::size_t) -> *mut libc::c_void {
    unsafe {
        if nsize == 0 {
            libc::free(ptr as *mut libc::c_void);
            ptr::null_mut()
        } else {
            libc::realloc(ptr, nsize)
        }
    }
}

// panic function should panic!() so Lua doesn't abort
extern "C" fn panic_helper(_L: *mut raw::lua_State) -> libc::c_int {
    panic!("lua error");
}

#[test]
fn test_lua_newstate() {
    unsafe {
        let L = raw::lua_newstate(alloc_helper, ptr::null_mut());
        raw::lua_atpanic(L, panic_helper);
        raw::lua_pushinteger(L, 42);
        raw::lua_close(L);
    }
}

#[test]
fn test_luaL_newstate() {
    unsafe {
        let L = aux::raw::luaL_newstate();
        raw::lua_atpanic(L, panic_helper);
        raw::lua_pushinteger(L, 42);
        raw::lua_close(L);
    }
}

#[test]
#[should_fail]
fn test_lua_error() {
    unsafe {
        let L = aux::raw::luaL_newstate();
        raw::lua_atpanic(L, panic_helper);
        raw::lua_pushinteger(L, 42);
        raw::lua_error(L);
    }
}

#[test]
fn test_dostring() {
    unsafe {
        let L = aux::raw::luaL_newstate();
        raw::lua_atpanic(L, panic_helper);
        let s = "function foo(x,y) return x+y end";
        let ret = aux::raw::luaL_dostring(L, CString::from_slice(s.as_bytes()).as_ptr());
        assert_eq!(ret, 0);
        raw::lua_getglobal(L, CString::from_slice(b"foo").as_ptr());

        raw::lua_pushinteger(L, 5);
        raw::lua_pushinteger(L, 3);

        raw::lua_call(L, 2, 1);
        let val = raw::lua_tointeger(L, -1);
        assert_eq!(val, 8);
        raw::lua_close(L);
    }
}
