//! Lua 5.1 bindings for Rust

#[link(name = "lua",
       package_id = "lua",
       vers = "0.1")];

#[comment = "Lua 5.1 bindings for Rust"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[warn(missing_doc)];

use std::libc;
use std::libc::c_int;
use std::ptr;

#[link(name = "lua.5.1")]
extern {}

/// Human-readable version string
pub static VERSION: &'static str = "Lua 5.1";
/// Machine-readable version number
pub static VERSION_NUM: int = 501;

/// Value for lua_call that means return all results
pub static MULTRET: i32 = raw::MULTRET as i32;

/// Minimum Lua stack available to a C function
pub static MINSTACK: i32 = 20;

#[allow(missing_doc)]
pub mod raw;
#[allow(missing_doc)]
pub mod aux;

#[path = "lualib.rs"]
#[allow(missing_doc)]
pub mod lib;

#[cfg(test)]
mod tests;

/// Lua value type
pub type Type = Type::Type;
pub mod Type {
    //! Lua value type mod
    use raw;
    /// Lua value types
    pub enum Type {
        /// Type for nil
        Nil = raw::LUA_TNIL,
        /// Type for booleans
        Boolean = raw::LUA_TBOOLEAN,
        /// Type for light userdata
        LightUserdata = raw::LUA_TLIGHTUSERDATA,
        /// Type for numbers
        Number = raw::LUA_TNUMBER,
        /// Type for strings
        String = raw::LUA_TSTRING,
        /// Type for tables
        Table = raw::LUA_TTABLE,
        /// Type for functions
        Function = raw::LUA_TFUNCTION,
        /// Type for userdata
        Userdata = raw::LUA_TUSERDATA,
        /// Type for threads
        Thread = raw::LUA_TTHREAD
    }
}

/// Garbage collection option
pub type GC = GC::GC;
pub mod GC {
    //! Garbage collection option mod
    use raw;
    /// Garbage collection options (used with State.gc())
    pub enum GC {
        /// Stops the garbage collector
        Stop = raw::LUA_GCSTOP,
        /// Restarts the garbage collector
        Restart = raw::LUA_GCRESTART,
        /// Performs a full garbage-collection cycle
        Collect = raw::LUA_GCCOLLECT,
        /// Returns the current amount of memory (in Kbytes) in use by Lua
        Count = raw::LUA_GCCOUNT,
        /// Returns the remainder of dividing the current amount of bytes in memory in use by Lua
        /// by 1024
        CountB = raw::LUA_GCCOUNTB,
        /// Performs an incremental step of garbage collection. The step "size" is controlled by
        /// `data` (larger values mean more steps) in a non-specified way. If you want to control
        /// the step size you must experimentally tune hte value of `data`. The function returns
        /// 1 if the step finished a garbage-collection cycle.
        Step = raw::LUA_GCSTEP,
        /// Sets `data` as the new value for the pause of the collector. The function returns the
        /// previous value of the pause.
        SetPause = raw::LUA_GCSETPAUSE,
        /// Sets `data` as the new value for the step multiplier of the collector. The function
        /// returns the previous value of the step multiplier.
        SetStepMul = raw::LUA_GCSETSTEPMUL
    }
}

/// The Lua state.
/// Every Lua thread is represented by a separate State.
///
/// When executing functions on the State that take acceptable indexes, these indexes are checked
/// to ensure they are within the stack space defined by the last call to State.checkstack(). If
/// they are not acceptable, the function fails without calling lua_checkstack().
/// Negative indices are checked against the current top of the stack instead of the stack space.
///
/// Unless otherwise noted, all safe functions that take indexes will fail if the index is not
/// acceptable. All unsafe functions named *_unchecked skip the index check.
#[unsafe_no_drop_flag]
pub struct State {
    priv L: *mut raw::lua_State,
    priv owned: bool,
    priv stackspace: i32
}

impl Drop for State {
    fn drop(&mut self) {
        if self.owned {
            self.owned = false;
            unsafe {
                raw::lua_close(self.L);
            }
        }
    }
}

impl State {
    /* State creation */

    /// Returns a new State, or fails if memory cannot be allocated for the state
    pub fn new() -> State {
        #[inline];
        State::new_opt().unwrap()
    }

    /// Returns a new State, or None if memory cannot be allocated for the state
    pub fn new_opt() -> Option<State> {
        return unsafe {
            let L = raw::lua_newstate(alloc, ptr::mut_null());
            if (L.is_not_null()) {
                raw::lua_atpanic(L, panic);
                Some(State::from_lua_State(L, true))
            } else {
                None
            }
        };

        extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
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
        extern "C" fn panic(L: *mut raw::lua_State) -> c_int {
            unsafe {
                let s = State::from_lua_State(L, false).describe(-1, false);
                fail!("unprotected error in call to Lua API ({})", s.unwrap_or_default());
            }
        }
    }

    /// Wraps a *raw::lua_State in a State
    /// If `owned` is true, the *lua_State will be closed when the State is dropped.
    pub unsafe fn from_lua_State(L: *mut raw::lua_State, owned: bool) -> State {
        #[inline];
        State{ L: L, owned: owned, stackspace: MINSTACK }
    }

    /* Utility functions */

    fn check_acceptable(&mut self, idx: i32) {
        #[inline];
        if idx > 0 {
            assert!(idx <= self.stackspace, "Index {} is unacceptable (stack space is {})",
                    idx, self.stackspace);
        } else if idx < 0 {
            self.check_valid(idx);
        } else {
            fail!("Index 0 is unacceptable");
        }
    }

    fn check_valid(&mut self, idx: i32) {
        #[inline];
        if idx == 0 {
            fail!("Index 0 is not valid");
        }
        let top = unsafe { raw::lua_gettop(self.L) } as i32;
        assert!(idx.abs() <= top, "Index {} is not valid (stack top is {})", idx, top);
    }

    /// Returns the textual description of the value at the given acceptable index.
    /// If the given index is non-valid, returns None.
    pub fn describe(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.describe_unchecked(idx) }
    }

    /// Unchecked variant of describe()
    pub unsafe fn describe_unchecked(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.describe_unchecked_nostack(idx, true)
    }

    /// Variant of describe_unchecked() that does not push on to the stack.
    /// describe() and describe_unchecked() may push new values onto the stack temporarily.
    /// Notably, it may do this to avoid converting the existing value's type.
    /// This method allows this behavior to be disabled.
    pub unsafe fn describe_unchecked_nostack(&mut self, idx: i32, usestack: bool) -> Option<~str> {
        match self.type_unchecked(idx) {
            None => None,
            Some(typ) => Some(match typ {
                Type::Nil => ~"nil",
                Type::Boolean => if self.toboolean_unchecked(idx) { ~"true" } else { ~"false" },
                Type::Number => {
                    // Let Lua create the string instead of us
                    if (usestack) { self.pushvalue_unchecked(idx); } // copy the value
                    let s = self.tostring_unchecked(-1);
                    if (usestack) { self.pop(1); } // remove the copied value
                    s.unwrap_or_default() // default will be ~""
                }
                _ => {
                    // TODO: flesh this out
                    ~"TODO"
                }
            })
        }
    }

    /* Basic stack manipulation */

    /// Returns the index of the top element of the stack.
    /// Indexes start at 1. 0 means the stack is empty.
    pub fn gettop(&mut self) -> i32 {
        #[inline];
        unsafe { raw::lua_gettop(self.L) as i32 }
    }

    /// Sets the stack top to the given acceptable index, or 0.
    /// If the new top is larger than the old one, new elements are filled with nil.
    /// If the index is 0, all stack elements are removed.
    pub fn settop(&mut self, idx: i32) {
        #[inline];
        if idx != 0 { self.check_acceptable(idx); }
        unsafe { self.settop_unchecked(idx) }
    }

    /// Unchecked variant of settop()
    pub unsafe fn settop_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_settop(self.L, idx as c_int)
    }

    /// Pushes a copy of the element at the given valid index onto the stack.
    /// Fails if the index is not valid.
    pub fn pushvalue(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx);
        unsafe { self.pushvalue_unchecked(idx) }
    }

    /// Unchecked variant of pushvalue()
    pub unsafe fn pushvalue_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_pushvalue(self.L, idx as c_int)
    }

    // remove
    // insert
    // replace
    // checkstack
    // xmove

    /* Access functions */

    // isnumber
    // isstring
    // iscfunction
    // isuserdata

    /// Returns the type of the value at the given acceptable index.
    /// If the given index is non-valid, returns None.
    pub fn type_(&mut self, idx: i32) -> Option<Type> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.type_unchecked(idx) }
    }

    /// Unchecked variant of type_()
    pub unsafe fn type_unchecked(&mut self, idx: i32) -> Option<Type> {
        match raw::lua_type(self.L, idx as c_int) {
            raw::LUA_TNONE => None,

            raw::LUA_TNIL           => Some(Type::Nil),
            raw::LUA_TBOOLEAN       => Some(Type::Boolean),
            raw::LUA_TLIGHTUSERDATA => Some(Type::LightUserdata),
            raw::LUA_TNUMBER        => Some(Type::Number),
            raw::LUA_TSTRING        => Some(Type::String),
            raw::LUA_TTABLE         => Some(Type::Table),
            raw::LUA_TFUNCTION      => Some(Type::Function),
            raw::LUA_TUSERDATA       => Some(Type::Userdata),
            raw::LUA_TTHREAD        => Some(Type::Thread),

            _ => fail!("Unknown return value from lua_type")
        }
    }

    // typename

    // equal
    // rawequal
    // lessthan

    // tonumber
    // tointeger

    /// Converts the value at the given acceptable index to a bool.
    /// Returns false when called with a non-valid index.
    pub fn toboolean(&mut self, idx: i32) -> bool {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.toboolean_unchecked(idx) }
    }

    /// Unchecked variant of toboolean()
    pub unsafe fn toboolean_unchecked(&mut self, idx: i32) -> bool {
        #[inline];
        raw::lua_toboolean(self.L, idx as c_int) != 0
    }

    /// Converts the value at the given acceptable index to a string.
    /// Returns None if the value is not a number or a string.
    /// Returns None if the string value is not utf-8.
    /// Note: if the value is a number, this method changes the value in the stack to a string.
    /// This may confuse lua_next if this is called during table traversal.
    pub fn tostring(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.tostring_unchecked(idx) }
    }

    /// Unchecked variant of tostring()
    pub unsafe fn tostring_unchecked(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.tostring_slice_unchecked(idx).map(|s| s.to_owned())
    }

    /// Converts the value at the given acceptable index into a string slice.
    /// See tostring() for details.
    pub fn tostring_slice<'a>(&'a mut self, idx: i32) -> Option<&'a str> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.tostring_slice_unchecked(idx) }
    }

    /// Unchecked variant of tostring_slice()
    pub unsafe fn tostring_slice_unchecked<'a>(&'a mut self, idx: i32) -> Option<&'a str> {
        let mut sz: libc::size_t = 0;
        let s = raw::lua_tolstring(self.L, idx, &mut sz);
        if s.is_null() {
            None
        } else {
            std::vec::raw::buf_as_slice(s as *u8, sz as uint, |b| {
                std::str::from_utf8_opt(b).map(|s| std::cast::transmute::<&str, &'a str>(s))
            })
        }
    }

    // objlen
    // tocfunction
    // touserdata
    // tothread
    // topointer

    /* Push functions (Rust -> stack) */

    // pushnil
    // pushnumber
    // pushinteger
    // pushlstring
    // pushstring
    // pushfstring
    // pushcclosure
    // pushboolean
    // pushlightuserdata
    // pushtthread

    /* Get functions (Lua -> stack) */

    // gettable
    // getfield
    // rawget
    // rawgeti
    // createtable
    // newuserdata
    // getmetatable
    // getfenv

    /* Set functions (stack -> Lua) */

    // settable
    // setfield
    // rawset
    // rawseti
    // setmetatable
    // setfenv

    /* `load` and `call` functions (load and run Lua code) */

    // call
    // pcall
    // cpcall
    // load
    // dump

    /* Coroutine functions */

    // yield
    // resume
    // status

    /* Garbage collection functions */

    // gc

    /* Miscellaneous functions */

    // error
    // next
    // concat
    // getallocf
    // setallocf

    /* Some useful functions (macros in C) */

    /// Pop n elements from the stack.
    /// Fails if the stack is smaller than n
    pub fn pop(&mut self, n: i32) {
        #[inline];
        if n >= 0 {
            assert!(self.gettop() >= n, "Stack underflow");
        } else {
            assert!(self.gettop() >= (n+1).abs(), "Stack underflow");
        }
        unsafe { self.pop_unchecked(n) }
    }

    /// Unchecked variant of pop()
    pub unsafe fn pop_unchecked(&mut self, n: i32) {
        raw::lua_pop(self.L, n as c_int)
    }

    // newtable
    // register
    // pushcfunction
    // strlen
    // isfunction
    // istable
    // islightuserdata
    // isnil
    // isboolean
    // isthread
    // isnone
    // isnoneornil
    // setglobal
    // getglobal

    /* Hack */

    // setlevel
}

// Functions from auxlib
impl State {
    // register
    // getmetafield
    // callmeta
    // typerror
    // argerror
    // checklstring
    // optlstring
    // checknumber
    // optnumber
    // checkinteger
    // optinteger
    // checktype
    // checkany
    // newmetadata
    // checkudata
    // where
    // error (probably call errorfmt? and implement manually with fmt!)
    // checkoption
    // ref
    // unref
    // loadfile
    // loadbuffer
    // loadstring
    // gsub

    /* Some useful functions (macros in C) */

    // argcheck
    // checkstring
    // optstring
    // checkint
    // optint
    // checklong
    // optlong
    // typename
    // dofile
    // dostring
    // getmetatable
    // opt

    /* Generic Buffer manipulation */

    // buffinit
}

pub struct Buffer<'a> {
    priv B: aux::raw::luaL_Buffer,
    priv _L: &'a State // luaL_Buffer keeps a *-ptr to the underlying state
}

pub static BUFFERSIZE: u32 = aux::raw::LUAL_BUFFERSIZE;

impl<'a> Buffer<'a> {
    // addchar
    // addsize
    // prepbuffer
    // addlstring
    // addstring
    // addvalue
    // pushresult (consume self)
}
