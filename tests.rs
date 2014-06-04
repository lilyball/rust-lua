use State;
use GLOBALSINDEX;
use Type;
use raw;

use libc;
use std::task;
use std::any::AnyRefExt;

#[test]
fn test_state_init() {
    let mut _s = State::new();
}

#[test]
#[should_fail]
fn test_error() {
    let mut s = State::new();
    s.pushinteger(42);
    s.error()
}

#[test]
fn test_errorstr() {
    let res = task::try::<()>(proc() {
        let mut s = State::new();
        s.errorstr("some err");
    });
    let err = res.unwrap_err();
    let expected = "unprotected error in call to Lua API (some err)";
    let s = err.as_ref::<String>();
    if s.is_some() {
        assert_eq!(s.unwrap().as_slice(), expected);
    } else {
        let s = err.as_ref::<&'static str>();
        if s.is_some() {
            assert_eq!(*s.unwrap(), expected);
        } else {
            fail!("unexpected failure result");
        }
    }
}

#[test]
fn test_describe() {
    let mut s = State::new();

    assert_eq!(s.typename(1), "no value");
    s.pushnil();
    assert_eq!(s.typename(-1), "nil");
    s.pushinteger(42);
    assert_eq!(s.typename(-1), "number");
    s.pushstring("test");
    assert_eq!(s.typename(-1), "string");
    s.pushboolean(true);
    assert_eq!(s.typename(-1), "boolean");
    s.pushcfunction(dummy);
    assert_eq!(s.typename(-1), "function");

    extern "C" fn dummy(_L: *mut ::raw::lua_State) -> ::libc::c_int {
        0
    }
}

#[test]
fn test_openlibs() {
    let mut s = State::new();

    s.openlibs();
    s.getfield(GLOBALSINDEX, "table");
    assert_eq!(s.type_(-1), Some(Type::Table));
}

#[deriving(PartialEq,Eq,Show)]
enum CheckOptionEnum {
    COEOne,
    COETwo,
    COEThree
}

#[test]
fn test_checkoption() {
    let lst = [("one", COEOne), ("two", COETwo), ("three", COEThree)];

    let mut s = State::new();

    for &(k,ref v) in lst.iter() {
        s.pushstring(k);
        assert_eq!(*s.checkoption(1, None, lst), *v);
        s.pop(1);
    }
    assert_eq!(*s.checkoption(1, Some("three"), lst), COEThree);

    let res = task::try(proc() {
        let mut s = State::new();
        s.checkoption(1, None, lst);
    });
    assert!(res.is_err(), "expected error from checkoption");

    let res = task::try(proc() {
        let mut s = State::new();
        s.checkoption(1, Some("four"), lst);
    });
    assert!(res.is_err(), "expected error from checkoption");
}

#[test]
fn test_tocfunction() {
    let mut s = State::new();

    // extern "C" fns don't implement Eq, so cast them to a pointer instead

    s.pushstring("foo");
    assert_eq!(s.tocfunction(1).map(|f| f as *()), None);

    s.pushcfunction(cfunc);
    assert_eq!(s.tocfunction(2).map(|f| f as *()), Some(cfunc as *()));

    extern "C" fn cfunc(_L: *mut raw::lua_State) -> libc::c_int { 0 }
}

#[test]
fn test_gsub() {
    // do some pretty basic gsub tests
    let mut L = State::new();

    assert_eq!(L.gsub("foobar", "bar", "quux"), "fooquux");
    assert_eq!(L.gsub("foo", "o", "ö"), "föö");
    assert_eq!(L.gsub("test", "a", "b"), "test");
    assert_eq!(L.gsub("a b c d e", " ", "."), "a.b.c.d.e");
}
