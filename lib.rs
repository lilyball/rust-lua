//! Lua 5.1 bindings for Rust

#[link(name = "lua",
       package_id = "lua",
       vers = "0.1")];

#[comment = "Lua 5.1 bindings for Rust"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

#[warn(missing_doc)];

use std::libc;
use std::libc::c_int;
use std::{path, ptr, str, vec};

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

/// Pseudo-index for the registry
pub static REGISTRYINDEX: i32 = raw::LUA_REGISTRYINDEX as i32;
/// Pseudo-index for the thread environment
pub static GLOBALSINDEX: i32 = raw::LUA_GLOBALSINDEX as i32;
/// Pseudo-index for the running C function environment
pub static ENVIRONINDEX: i32 = raw::LUA_ENVIRONINDEX as i32;

/// Calculates the pseudo-index for the upvalue at the given index.
/// Any index in the range [1,256] produces an acceptable index.
/// Any index outside that range will likely produce an unacceptable index.
pub fn upvalueindex(n: i32) -> i32 {
    #[inline];
    raw::lua_upvalueindex(n as c_int) as i32
}

#[allow(missing_doc)]
pub mod raw;
#[allow(missing_doc)]
pub mod aux;

#[path = "lualib.rs"]
#[allow(missing_doc)]
pub mod lib;

#[cfg(test)]
mod tests;

macro_rules! luaassert{
    ($state:expr, $cond:expr, $msg:expr) => {
        if !$cond {
            $state.errorstr($msg);
        }
    };
    ($state:expr, $cond:expr, $($arg:expr),+) => {
        if !$cond {
            let msg = format!($($arg),+);
            $state.errorstr(msg);
        }
    }
}

/// Lua value type
pub type Type = Type::Type;
pub mod Type {
    //! Lua value type mod
    use raw;
    use std::{libc, ptr, str};

    /// Lua value types
    #[deriving(Clone,Eq)]
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

    impl Type {
        /// Returns the name of the type
        pub fn name(&self) -> &'static str {
            unsafe {
                let s = raw::lua_typename(ptr::mut_null(), *self as libc::c_int);
                str::raw::c_str_to_static_slice(s)
            }
        }
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

/// Type that represents C functions that can be registered with Lua.
pub type CFunction = raw::lua_CFunction;

/// State.load() errors
pub type LoadError = LoadError::LoadError;
pub mod LoadError {
    //! State.load() error mod
    use raw;
    /// State.load() errors
    pub enum LoadError {
        /// Syntax error during pre-compilation
        ErrSyntax = raw::LUA_ERRSYNTAX,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM
    }
}

/// State.loadfile() errors
pub type LoadFileError = LoadFileError::LoadFileError;
pub mod LoadFileError {
    //! State.loadfile() error mod
    use aux;
    use raw;
    /// State.loadfile() errors
    pub enum LoadFileError {
        /// Syntax error during pre-compilation
        ErrSyntax = raw::LUA_ERRSYNTAX,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM,
        /// Cannot read/open the file
        ErrFile = aux::raw::LUA_ERRFILE
    }
}

/// State.pcall() errors
pub type PCallError = PCallError::PCallError;
pub mod PCallError {
    //! State.pcall() error mod
    use raw;
    /// State.pcall() errors
    pub enum PCallError {
        /// Runtime error
        ErrRun = raw::LUA_ERRRUN,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM,
        /// Error while running the error handler function
        ErrErr = raw::LUA_ERRERR
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
                let s = State::from_lua_State(L, false).describe_unchecked_stack(-1, false);
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

    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        self.L
    }

    /* Utility functions */

    fn check_acceptable(&mut self, idx: i32) {
        #[inline];
        if idx > 0 {
            luaassert!(self, idx <= self.stackspace,
                       "index {} is not acceptable (stack space is {})", idx, self.stackspace);
        } else if idx < 0 {
            self.check_valid(idx, true);
        } else {
            self.errorstr("index 0 is not acceptable");
        }
    }

    fn check_valid(&mut self, idx: i32, allowpseudo: bool) {
        #[inline];
        match idx {
            0 => self.errorstr("index 0 is not valid"),
            GLOBALSINDEX |
            REGISTRYINDEX |
            ENVIRONINDEX => luaassert!(self, allowpseudo,
                                       "Pseudo-indices are not valid for this call"),
            _ if idx < GLOBALSINDEX => {
                luaassert!(self, allowpseudo, "Pseudo-indices are not valid for this call");
                // we can't actually test for upvalue validity
                // at least not without using lua_Debug, which seems excessive.
                // However, I think that invalid but acceptable upvalues are treated as nil
                let upvalidx = GLOBALSINDEX - idx;
                luaassert!(self, upvalidx <= 256, "upvalue index {} is out of range", upvalidx);
            }
            _ => {
                let top = self.gettop();
                luaassert!(self, idx.abs() <= top, "index {} is not valid (stack top is {})", idx,
                           top);
            }
        }
    }

    /// Returns the textual description of the value at the given acceptable index.
    /// If the given index is non-valid, returns None.
    pub fn describe(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.check_acceptable(idx);
        self.checkstack_(1);
        unsafe { self.describe_unchecked(idx) }
    }

    /// Unchecked variant of describe()
    /// May require 1 extra slot on the stack.
    pub unsafe fn describe_unchecked(&mut self, idx: i32) -> Option<~str> {
        #[inline];
        self.describe_unchecked_stack(idx, true)
    }

    /// Variant of describe_unchecked() that does not push on to the stack.
    /// describe() and describe_unchecked() may push new values onto the stack temporarily.
    /// Notably, it may do this to avoid converting the existing value's type.
    /// This method allows this behavior to be disabled.
    /// If usestack is on, this method may require 1 free slot on the stack.
    pub unsafe fn describe_unchecked_stack(&mut self, idx: i32, usestack: bool) -> Option<~str> {
        match self.type_unchecked(idx) {
            None => None,
            Some(typ) => Some(match typ {
                Type::Nil => ~"nil",
                Type::Boolean => if self.toboolean_unchecked(idx) { ~"true" } else { ~"false" },
                Type::Number => {
                    // Let Lua create the string instead of us
                    if (usestack) { self.pushvalue_unchecked(idx); } // copy the value
                    let s = self.tostring_unchecked(-1).map(|s| s.to_owned());
                    if (usestack) { self.pop(1); } // remove the copied value
                    s.unwrap_or_default() // default will be ~""
                }
                Type::String => {
                    self.tostring_unchecked(idx).unwrap_or("<invalid utf8>").to_owned()
                }
                Type::LightUserdata |
                Type::Userdata |
                Type::Table |
                Type::Thread |
                Type::Function => {
                    let s = self.typename(idx);
                    let p = self.topointer(idx);
                    format!("<{} {:p}>", s, p)
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
        self.check_valid(idx, true);
        self.checkstack_(1);
        unsafe { self.pushvalue_unchecked(idx) }
    }

    /// Unchecked variant of pushvalue()
    pub unsafe fn pushvalue_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_pushvalue(self.L, idx as c_int)
    }

    /// Removes the element at the given valid index, shifting other elements as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn remove(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx, false);
        unsafe { self.remove_unchecked(idx) }
    }

    /// Unchecked variant of remove()
    pub unsafe fn remove_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_remove(self.L, idx as c_int)
    }

    /// Moves the top element into the given valid index, shifting existing elements as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn insert(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx, false);
        unsafe { self.insert_unchecked(idx) }
    }

    /// Unchecked variant of insert()
    pub unsafe fn insert_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_insert(self.L, idx as c_int)
    }

    /// Moves the top element into the given valid index and replaces the existing value, without
    /// shifting any other elements.
    pub fn replace(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx, true);
        unsafe { self.replace_unchecked(idx) }
    }

    /// Unchecked variant of replace()
    pub unsafe fn replace_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_replace(self.L, idx as c_int)
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Returns false if it cannot grow the stack as requested.
    pub fn checkstack(&mut self, extra: i32) -> bool {
        #[inline];
        let top = self.gettop();
        if top + extra > self.stackspace {
            if unsafe { raw::lua_checkstack(self.L, extra as c_int) } != 0 {
                self.stackspace = top + extra;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Fails if it cannot grow the stack.
    pub fn checkstack_(&mut self, extra: i32) {
        luaassert!(self, self.checkstack(extra), "cannot grow stack")
    }

    /// Exchanges values between different threads of the same global state.
    /// This method pops n values from the stack `self`, and pushes them to the stack `to`.
    ///
    /// Note: this method is unsafe because it cannot check to ensure that both threads belong
    /// to the same global state.
    ///
    /// Despite being unsafe, it still checks the validity of `n`.
    pub unsafe fn xmove(&mut self, to: &mut State, n: i32) {
        #[inline];
        luaassert!(self, self.gettop() >= n, "stack underflow");
        to.checkstack_(1);
        self.xmove_unchecked(to, n)
    }

    /// Unchecked variant of xmove()
    pub unsafe fn xmove_unchecked(&mut self, to: &mut State, n: i32) {
        #[inline];
        raw::lua_xmove(self.L, to.L, n as c_int)
    }

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
            raw::LUA_TUSERDATA      => Some(Type::Userdata),
            raw::LUA_TTHREAD        => Some(Type::Thread),

            _ => self.errorstr("Unknown return value from lua_type")
        }
    }

    /// Returns the name of the type of the value at the given acceptable index.
    pub fn typename(&mut self, idx: i32) -> &'static str {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.typename_unchecked(idx) }
    }

    /// Unchecked variant of typename()
    pub unsafe fn typename_unchecked(&mut self, idx: i32) -> &'static str {
        #[inline];
        let s = aux::raw::luaL_typename(self.L, idx as c_int);
        str::raw::c_str_to_static_slice(s)
    }

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
    ///
    /// Returns None if the value is not a number or a string.
    /// Returns None if the string value is not utf-8.
    ///
    /// Note: if the value is a number, this method changes the value in the stack to a string.
    /// This may confuse lua_next if this is called during table traversal.
    ///
    /// Note: This method borrows the state. Call .map(|s| s.to_owned()) on the result if you need
    /// to continue using the state while the string is alive.
    pub fn tostring<'a>(&'a mut self, idx: i32) -> Option<&'a str> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.tostring_unchecked(idx) }
    }

    /// Unchecked variant of tostring()
    pub unsafe fn tostring_unchecked<'a>(&'a mut self, idx: i32) -> Option<&'a str> {
        #[inline];
        self.tobytes_unchecked(idx).and_then(|v| str::from_utf8_opt(v))
    }

    /// Converts the value at the given acceptable index into a lua string, and returns it
    /// as a byte vector.
    /// Returns None if the value is not a number or a string.
    /// See tostring() for caveats.
    pub fn tobytes<'a>(&'a mut self, idx: i32) -> Option<&'a [u8]> {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.tobytes_unchecked(idx) }
    }

    /// Unchecked variant of tobytes()
    pub unsafe fn tobytes_unchecked<'a>(&'a mut self, idx: i32) -> Option<&'a [u8]> {
        #[inline];
        let mut sz: libc::size_t = 0;
        let s = raw::lua_tolstring(self.L, idx, &mut sz);
        if s.is_null() {
            None
        } else {
            vec::raw::buf_as_slice(s as *u8, sz as uint, |b| {
                Some(std::cast::transmute::<&[u8], &'a [u8]>(b))
            })
        }
    }

    // objlen
    // tocfunction
    // touserdata
    // tothread

    /// Converts the value at the given acceptable index to a pointer.
    /// The value can be a userdata, a table, a thread, or a function.
    pub fn topointer(&mut self, idx: i32) -> *libc::c_void {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.topointer_unchecked(idx) }
    }

    /// Unchecked variant of topointer()
    pub unsafe fn topointer_unchecked(&mut self, idx: i32) -> *libc::c_void {
        #[inline];
        raw::lua_topointer(self.L, idx as c_int)
    }

    /* Push functions (Rust -> stack) */

    /// Pushes a nil value onto the stack.
    pub fn pushnil(&mut self) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushnil_unchecked() }
    }

    /// Unchecked variant of pushnil().
    /// Skips the call to checkstack().
    pub unsafe fn pushnil_unchecked(&mut self) {
        #[inline];
        raw::lua_pushnil(self.L)
    }

    /// Pushes a number with value `n` onto the stack
    pub fn pushnumber(&mut self, n: f64) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushnumber_unchecked(n) }
    }

    /// Unchecked variant of pushnumber().
    /// Skips the call to checkstack().
    pub unsafe fn pushnumber_unchecked(&mut self, n: f64) {
        #[inline];
        raw::lua_pushnumber(self.L, n as raw::lua_Number)
    }

    /// Pushes a number with value `n` onto the stack.
    pub fn pushinteger(&mut self, n: int) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushinteger_unchecked(n) }
    }

    /// Unchecked variant of pushinteger().
    /// Skips the call to checkstack().
    pub unsafe fn pushinteger_unchecked(&mut self, n: int) {
        #[inline];
        raw::lua_pushinteger(self.L, n as raw::lua_Integer)
    }

    /// Pushes a string onto the stack
    pub fn pushstring(&mut self, s: &str) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushstring_unchecked(s) }
    }

    /// Unchecked variant of pushstring().
    /// Skips the call to checkstack().
    pub unsafe fn pushstring_unchecked(&mut self, s: &str) {
        #[inline];
        s.as_imm_buf(|buf, len| {
            raw::lua_pushlstring(self.L, buf as *libc::c_char, len as libc::size_t)
        })
    }

    /// Pushes a byte vector onto the stack as a lua string
    pub fn pushbytes(&mut self, bytes: &[u8]) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushbytes_unchecked(bytes) }
    }

    /// Unchecked variant of pushbytes()
    pub unsafe fn pushbytes_unchecked(&mut self, bytes: &[u8]) {
        #[inline];
        bytes.as_imm_buf(|buf, len| {
            raw::lua_pushlstring(self.L, buf as *libc::c_char, len as libc::size_t)
        })
    }

    /// Pushes a new C closure onto the stack.
    pub fn pushcclosure(&mut self, f: CFunction, n: i32) {
        #[inline];
        if n == 0 {
            self.checkstack_(1);
        }
        unsafe { self.pushcclosure_unchecked(f, n) }
    }

    /// Unchecked variant of pushcclosure().
    /// Skips the call to checkstack().
    pub unsafe fn pushcclosure_unchecked(&mut self, f: CFunction, n: i32) {
        #[inline];
        raw::lua_pushcclosure(self.L, f, n as c_int)
    }

    /// Pushes a boolean value onto the stack.
    pub fn pushboolean(&mut self, b: bool) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushboolean_unchecked(b) }
    }

    /// Unchecked variant of pushboolean().
    /// Skips the call to checkstack().
    pub unsafe fn pushboolean_unchecked(&mut self, b: bool) {
        #[inline];
        raw::lua_pushboolean(self.L, b as c_int)
    }

    // pushlightuserdata
    // pushtthread

    /* Get functions (Lua -> stack) */

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index and k is the value at the top of the stack.
    /// The key is popped from the stack.
    pub fn gettable(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "stack underflow");
        unsafe { self.gettable_unchecked(idx) }
    }

    /// Unchecked variant of gettable().
    pub unsafe fn gettable_unchecked(&mut self, idx: i32) {
        #[inline];
        raw::lua_gettable(self.L, idx as c_int)
    }

    /// Pushes onto the stack the value t[k], where t is the value at the given valid index.
    /// Raises the c_str::null_byte condition if `k` has any interior NULs.
    pub fn getfield(&mut self, idx: i32, k: &str) {
        #[inline];
        self.check_valid(idx, true);
        self.checkstack_(1);
        unsafe { self.getfield_unchecked(idx, k) }
    }

    /// Unchecked variant of getfield().
    /// Raises the c_str::null_byte condition if `k` has any interior NULs.
    pub unsafe fn getfield_unchecked(&mut self, idx: i32, k: &str) {
        #[inline];
        k.with_c_str(|s| raw::lua_getfield(self.L, idx as c_int, s))
    }
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

    /// Calls a function.
    /// The function must be pushed first, followed by its arguments. `nargs` is the number of
    /// arguments. The function and its arguments are popped automatically.
    /// The function results are adjusted to `nresults`, unless `nresults` is `MULTRET`, in which
    /// case all function results are pushed.
    pub fn call(&mut self, nargs: i32, nresults: i32) {
        #[inline];
        luaassert!(self, nargs >= 0, "invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "invalid nresults");
        luaassert!(self, self.gettop() > nargs, "stack underflow");
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        unsafe { self.call_unchecked(nargs, nresults) }
    }

    /// Unchecked variant of call().
    pub unsafe fn call_unchecked(&mut self, nargs: i32, nresults: i32) {
        #[inline];
        raw::lua_call(self.L, nargs as c_int, nresults as c_int)
    }

    /// Calls a function in protected mode.
    ///
    /// If no error occurs, this behaves identically to call() and returns Ok(()).
    /// If there is any error, the error message is pushed onto the stack, and an error code
    /// is returned. The function and its arguments are always removed from the stack.
    ///
    /// If `errfunc` is 0, then the error message returned on the stack is exactly the original
    /// error message. Otherwise, `errfunc` is the stack index of an error handler function.
    /// It must not be a pseudo-index.
    pub fn pcall(&mut self, nargs: i32, nresults: i32, errfunc: i32) -> Result<(),PCallError> {
        #[inline];
        luaassert!(self, nargs >= 0, "invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "invalid nresults");
        luaassert!(self, self.gettop() > nargs, "stack underflow");
        if errfunc != 0 {
            self.check_valid(errfunc, false)
        }
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        unsafe { self.pcall_unchecked(nargs, nresults, errfunc) }
    }

    /// Unchecked variant of pcall()
    pub unsafe fn pcall_unchecked(&mut self, nargs: i32, nresults: i32, errfunc: i32)
                                 -> Result<(),PCallError> {
        match raw::lua_pcall(self.L, nargs as c_int, nresults as c_int, errfunc as c_int) {
            0 => Ok(()),
            raw::LUA_ERRRUN => Err(PCallError::ErrRun),
            raw::LUA_ERRMEM => Err(PCallError::ErrMem),
            raw::LUA_ERRERR => Err(PCallError::ErrErr),
            _ => self.errorstr("unexpected error from lua_pcall")
        }
    }

    // Don't bother wrapping cpcall, userdatas are unsafe in Rust and the better approach is just
    // to call .pushcfunction() and .pcall().

    // load
    // dump

    /* Coroutine functions */

    // yield
    // resume
    // status

    /* Garbage collection functions */

    // gc

    /* Miscellaneous functions */

    /// Raises an error (using the value at the top of the stack)
    pub fn error(&mut self) -> ! {
        #[inline];
        luaassert!(self, self.gettop() > 0, "Stack underflow");
        unsafe { self.error_unchecked() }
    }

    /// Unchecked variant of error()
    pub unsafe fn error_unchecked(&mut self) -> ! {
        #[inline];
        raw::lua_error(self.L);
        unreachable!()
    }

    // next

    /// Concatenates the `n` values at the top of the stack, pops them, and
    /// leaves the result at the top.
    /// Fails if n is negative or larger than the stack top.
    pub fn concat(&mut self, n: i32) {
        #[inline];
        luaassert!(self, n >= 0, "Cannot concat negative elements");
        luaassert!(self, n <= self.gettop(), "Stack underflow");
        if n == 0 { self.checkstack_(1) }
        unsafe { self.concat_unchecked(n) }
    }

    /// Unchecked variant of concat()
    pub unsafe fn concat_unchecked(&mut self, n: i32) {
        #[inline];
        raw::lua_concat(self.L, n as c_int)
    }

    // getallocf
    // setallocf

    /* Some useful functions (macros in C) */

    /// Pop n elements from the stack.
    /// Fails if the stack is smaller than n
    pub fn pop(&mut self, n: i32) {
        #[inline];
        if n >= 0 {
            luaassert!(self, self.gettop() >= n, "Stack underflow");
        } else {
            luaassert!(self, self.gettop() >= (n+1).abs(), "Stack underflow");
        }
        unsafe { self.pop_unchecked(n) }
    }

    /// Unchecked variant of pop()
    pub unsafe fn pop_unchecked(&mut self, n: i32) {
        #[inline];
        raw::lua_pop(self.L, n as c_int)
    }

    // newtable
    // register
    /// Sets the C function `f` as the new value of global `name`.
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs
    pub fn register(&mut self, name: &str, f: CFunction) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.register_unchecked(name, f) }
    }

    /// Unchecked variant of register()
    pub unsafe fn register_unchecked(&mut self, name: &str, f: CFunction) {
        #[inline];
        name.with_c_str(|s| raw::lua_register(self.L, s, f) )
    }

    /// Pushes a C function onto the stack.
    pub fn pushcfunction(&mut self, f: CFunction) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushcfunction_unchecked(f) }
    }

    /// Unchecked variant of pushcfunction().
    pub unsafe fn pushcfunction_unchecked(&mut self, f: CFunction) {
        #[inline];
        raw::lua_pushcfunction(self.L, f)
    }

    // strlen
    // isfunction
    // istable
    // islightuserdata
    // isnil
    // isboolean
    // isthread
    // isnone
    // isnoneornil

    /// Pops a value from the stack and sets it as the new value of global `name`.
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs.
    pub fn setglobal(&mut self, name: &str) {
        #[inline];
        luaassert!(self, self.gettop() > 0, "stack underflow");
        unsafe { self.setglobal_unchecked(name) }
    }

    /// Unchecked variant of setglobal().
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs.
    pub unsafe fn setglobal_unchecked(&mut self, name: &str) {
        name.with_c_str(|s| raw::lua_setglobal(self.L, s))
    }

    /// Pushes onto the stack the value of the global `name`.
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs.
    pub fn getglobal(&mut self, name: &str) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.getglobal_unchecked(name) }
    }

    /// Unchecked variant of getglobal().
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs.
    pub unsafe fn getglobal_unchecked(&mut self, name: &str) {
        #[inline];
        name.with_c_str(|s| raw::lua_getglobal(self.L, s))
    }

    /* Hack */

    // setlevel
}

/// Name for the coroutine lib
pub static COLIBNAME: &'static str = lib::raw::LUA_COLIBNAME;
/// Name for the table lib
pub static TABLIBNAME: &'static str = lib::raw::LUA_TABLIBNAME;
/// Name for the io lib
pub static IOLIBNAME: &'static str = lib::raw::LUA_IOLIBNAME;
/// Name for the os lib
pub static OSLIBNAME: &'static str = lib::raw::LUA_OSLIBNAME;
/// Name for the string lib
pub static STRLIBNAME: &'static str = lib::raw::LUA_STRLIBNAME;
/// Name for the math lib
pub static MATHLIBNAME: &'static str = lib::raw::LUA_MATHLIBNAME;
/// Name for the debug lib
pub static DBLIBNAME: &'static str = lib::raw::LUA_DBLIBNAME;
/// Name for the package lib
pub static LOADLIBNAME: &'static str = lib::raw::LUA_LOADLIBNAME;

// Functions from lualib
impl State {
    /// Open the basic library.
    pub fn open_base(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_base_unchecked() }
    }

    /// Unchecked variant of open_base().
    /// Skips the call to checkstack().
    pub unsafe fn open_base_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_base);
        self.pushstring("");
        self.call(1, 0);
    }

    /// Opens the table library.
    pub fn open_table(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_table_unchecked() }
    }

    /// Unchecked variant of open_table().
    /// Skips the call to checkstack().
    pub unsafe fn open_table_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_table);
        self.pushstring(TABLIBNAME);
        self.call(1, 0);
    }

    /// Opens the io library.
    pub fn open_io(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_io_unchecked() }
    }

    /// Unchecked variant of open_io().
    /// Skips the call to checkstack().
    pub unsafe fn open_io_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_io);
        self.pushstring(IOLIBNAME);
        self.call(1, 0);
    }

    /// Opens the os library.
    pub fn open_os(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_os_unchecked() }
    }

    /// Unchecked variant of open_os().
    /// Skips the call to checkstack().
    pub unsafe fn open_os_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_os);
        self.pushstring(OSLIBNAME);
        self.call(1, 0);
    }

    /// Opens the string library.
    pub fn open_string(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_string_unchecked() }
    }

    /// Unchecked variant of open_string().
    /// Skips the call to checkstack().
    pub unsafe fn open_string_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_string);
        self.pushstring(STRLIBNAME);
        self.call(1, 0);
    }

    /// Opens the math library.
    pub fn open_math(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_math_unchecked() }
    }

    /// Unchecked variant of open_math().
    /// Skips the call to checkstack().
    pub unsafe fn open_math_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_math);
        self.pushstring(MATHLIBNAME);
        self.call(1, 0);
    }

    /// Opens the debug library.
    pub fn open_debug(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_debug_unchecked() }
    }

    /// Unchecked variant of open_debug().
    /// Skips the call to checkstack().
    pub unsafe fn open_debug_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_debug);
        self.pushstring(DBLIBNAME);
        self.call(1, 0);
    }

    /// Opens the package library.
    pub fn open_package(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.open_package_unchecked() }
    }

    /// Unchecked variant of open_package().
    /// Skips the call to checkstack().
    pub unsafe fn open_package_unchecked(&mut self) {
        #[inline];
        self.pushcfunction(lib::raw::luaopen_package);
        self.pushstring(LOADLIBNAME);
        self.call(1, 0);
    }

    /// Opens all standard Lua libraries.
    pub fn openlibs(&mut self) {
        #[inline];
        self.checkstack_(2);
        unsafe { self.openlibs_unchecked() }
    }

    /// Unchecked variant of openlibs().
    /// Skips the call to checkstack().
    pub unsafe fn openlibs_unchecked(&mut self) {
        #[inline];
        lib::raw::luaL_openlibs(self.L)
    }
}

// Functions from auxlib
impl State {
    // register
    // getmetafield
    // callmeta
    // typerror
    // argerror
    /// Raises an error with the following message, where `func` is taken from the call stack:
    ///
    ///   bad argument #<narg> to <func> (<extramsg>)
    pub fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        extramsg.with_c_str(|msg| {
            unsafe { aux::raw::luaL_argerror(self.L, narg as c_int, msg); }
            unreachable!()
        })
    }

    /// Checks whether the function argument `narg` is a string, and returns the string.
    /// This function uses lua_tolstring to get its result, so all conversions and caveats of
    /// that function apply here.
    ///
    /// If the string is not utf-8, returns None.
    ///
    /// Note: use .map(|s| s.to_owned()) if you need to use the state while the string is alive.
    pub fn checkstring<'a>(&'a mut self, narg: i32) -> Option<&'a str> {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkstring_unchecked(narg) }
    }

    /// Unchecked variant of checkstring()
    pub unsafe fn checkstring_unchecked<'a>(&'a mut self, narg: i32) -> Option<&'a str> {
        #[inline];
        str::from_utf8_opt(self.checkbytes_unchecked(narg))
    }

    /// Checks whether the function argument `narg` is a lua string, and returns it as a
    /// byte vector. See checkstring() for caveats.
    pub fn checkbytes<'a>(&'a mut self, narg: i32) -> &'a [u8] {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkbytes_unchecked(narg) }
    }

    /// Unchecked variant of checkbytes()
    pub unsafe fn checkbytes_unchecked<'a>(&'a mut self, narg: i32) -> &'a [u8] {
        let mut sz: libc::size_t = 0;
        let s = aux::raw::luaL_checklstring(self.L, narg, &mut sz);
        vec::raw::buf_as_slice(s as *u8, sz as uint, |b| {
            std::cast::transmute::<&[u8], &'a [u8]>(b)
        })
    }

    // optlstring

    /// Checks whether the function argument `narg` is a number and returns the number.
    pub fn checknumber(&mut self, narg: i32) -> f64 {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checknumber_unchecked(narg) }
    }

    /// Unchecked variant of checknumber()
    pub unsafe fn checknumber_unchecked(&mut self, narg: i32) -> f64 {
        #[inline];
        aux::raw::luaL_checknumber(self.L, narg as c_int) as f64
    }

    // optnumber

    /// Checks whether the function argument `narg` is a number and returns it as an int.
    pub fn checkint(&mut self, narg: i32) -> int {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkint_unchecked(narg) }
    }

    /// Unchecked variant of checkint()
    pub unsafe fn checkint_unchecked(&mut self, narg: i32) -> int {
        #[inline];
        aux::raw::luaL_checkinteger(self.L, narg as c_int) as int
    }

    // optinteger
    // checktype
    /// Checks whether the function has an argument of any type (including nil) at position `narg`.
    pub fn checkany(&mut self, narg: i32) {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkany_unchecked(narg) }
    }

    /// Unchecked variant of checkany()
    pub unsafe fn checkany_unchecked(&mut self, narg: i32) {
        #[inline];
        aux::raw::luaL_checkany(self.L, narg as c_int)
    }

    // newmetadata
    // checkudata

    /// Pushes onto the stack a string identifying the current position of the
    /// control at level `lvl` in the call stack.
    /// Level 0 is the running function, level 1 is the function that called
    /// the running function, etc.
    pub fn where(&mut self, lvl: i32) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.where_unchecked(lvl) }
    }

    /// Unchecked variant of where()
    pub unsafe fn where_unchecked(&mut self, lvl: i32) {
        #[inline];
        aux::raw::luaL_where(self.L, lvl as c_int)
    }

    /// Raises an error with the given string.
    /// It also adds at the beginning of the message the file name and line
    /// number where the error occurred, if this information is available.
    pub fn errorstr(&mut self, s: &str) -> ! {
        #[inline];
        self.checkstack_(2);
        unsafe { self.errorstr_unchecked(s) }
    }

    /// Unchecked variant of errorstr()
    pub unsafe fn errorstr_unchecked(&mut self, s: &str) -> ! {
        self.where_unchecked(1);
        self.pushstring(s);
        self.concat_unchecked(2);
        raw::lua_error(self.L);
        unreachable!()
    }

    /// Checks whether the function arg `narg` is a string and searches for this string in `lst`.
    /// The first element of each tuple is compared against, and if a match is found, the second
    /// element is returned.
    /// Raises an error if the argument is not a string or the string cannot be found.
    ///
    /// If `def` is not None, the function uses `def` as a default value when there is no argument
    /// `narg` or this argument is nil.
    ///
    /// Raises the `c_str::null_byte` condition if `def` or any list key has interior NULs
    pub fn checkoption<'a, T>(&mut self, narg: i32, def: Option<&str>, lst: &'a [(&str,T)])
                             -> &'a T {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkoption_unchecked(narg, def, lst) }
    }

    /// Unchecked variant of checkoption()
    pub unsafe fn checkoption_unchecked<'a, T>(&mut self, narg: i32, def: Option<&str>,
                                               lst: &'a [(&str,T)]) -> &'a T {
        let def_cstr = def.map(|d| d.to_c_str());
        let defp = def_cstr.as_ref().map_default(ptr::null(), |c| c.with_ref(|p| p));
        let mut lst_cstrs = vec::with_capacity(lst.len());
        let mut lstv = vec::with_capacity(lst.len()+1);
        for &(k,_) in lst.iter() {
            let cstr = k.to_c_str();
            let p = cstr.with_ref(|p| p);
            lst_cstrs.push(cstr);
            lstv.push(p);
        }
        lstv.push(ptr::null());
        let i = lstv.as_imm_buf(|b,_| aux::raw::luaL_checkoption(self.L, narg as c_int, defp, b));
        lst[i].second_ref()
    }

    // ref
    // unref

    /// Loads a file as a Lua chunk (but does not run it).
    /// If the `filename` is None, this loads from standard input.
    /// Raises the c_str::null_byte condition if `filename` has any interior NULs.
    pub fn loadfile(&mut self, filename: Option<&path::Path>) -> Result<(),LoadFileError> {
        #[inline];
        self.checkstack_(1);
        unsafe { self.loadfile_unchecked(filename) }
    }

    /// Unchecked variant of loadfile()
    pub unsafe fn loadfile_unchecked(&mut self, filename: Option<&path::Path>)
                                    -> Result<(),LoadFileError> {
        let cstr = filename.map(|p| p.to_c_str());
        let ptr = cstr.as_ref().map_default(ptr::null(), |cstr| cstr.with_ref(|p| p));
        match aux::raw::luaL_loadfile(self.L, ptr) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadFileError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadFileError::ErrMem),
            aux::raw::LUA_ERRFILE => Err(LoadFileError::ErrFile),
            _ => self.errorstr("unexpected error from luaL_loadfile")
        }
    }

    /// Loads a buffer as a Lua chunk (but does not run it).
    /// As far as Rust is concerned, this differ from loadstring() in that a name for the chunk
    /// is provided. It also allows for NUL bytes, but I expect Lua won't like those.
    /// Raises the c_str::null_byte condition if `name` has any interior NULs.
    pub fn loadbuffer(&mut self, buff: &str, name: &str) -> Result<(),LoadError> {
        #[inline];
        self.checkstack_(1);
        unsafe { self.loadbuffer_unchecked(buff, name) }
    }

    /// Unchecked variant of loadbuffer()
    pub unsafe fn loadbuffer_unchecked(&mut self, buff: &str, name: &str) -> Result<(),LoadError> {
        buff.as_imm_buf(|bp, bsz| {
            let bp = bp as *libc::c_char;
            let bsz = bsz as libc::size_t;
            match name.with_c_str(|name| aux::raw::luaL_loadbuffer(self.L, bp, bsz, name)) {
                0 => Ok(()),
                raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
                raw::LUA_ERRMEM => Err(LoadError::ErrMem),
                _ => self.errorstr("unexpected error from luaL_loadbuffer")
            }
        })
    }

    /// Loads a string as a Lua chunk (but does not run it).
    /// Raises the c_str::null_byte condition if `s` has any interior NULs.
    pub fn loadstring(&mut self, s: &str) -> Result<(),LoadError> {
        #[inline];
        self.checkstack_(1);
        unsafe { self.loadstring_unchecked(s) }
    }

    /// Unchecked variant of loadstring()
    pub unsafe fn loadstring_unchecked(&mut self, s: &str) -> Result<(),LoadError> {
        match s.with_c_str(|s| aux::raw::luaL_loadstring(self.L, s)) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("unexpected error from luaL_loadstring")
        }
    }
    // gsub

    /* Some useful functions (macros in C) */

    /// Checks whether `cond` is true. If not, raises an error with the following messag, where
    /// `func` is retrieved from the call stack:
    ///
    ///   bad argument #<narg> to <func> (<extramsg>)
    ///
    /// Raises the `c_str::null_byte` condition if `extramsg` has interior NULs.
    pub fn argcheck(&mut self, cond: bool, narg: i32, extramsg: &str) {
        extramsg.with_c_str(|msg| {
            unsafe { aux::raw::luaL_argcheck(self.L, cond, narg as c_int, msg) }
        })
    }
    // typename
    // dofile
    // dostring
    // getmetatable
    // opt

    /* Generic Buffer manipulation */

    // buffinit
}

/// String buffer for building Lua strings piecemeal
pub struct Buffer<'a> {
    priv B: aux::raw::luaL_Buffer,
    priv _L: &'a State // luaL_Buffer keeps a *-ptr to the underlying state
}

/// Size of the internal buffer used by Buffer and returned by prepbuffer()
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
