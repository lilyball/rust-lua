//! Lua 5.1 bindings for Rust

#[crate_id="github.com/kballard/rust-lua#lua:0.1"];

#[comment = "Lua 5.1 bindings for Rust"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

#[warn(missing_doc)];

use std::libc;
use std::libc::c_int;
use std::{cast, path, ptr, str, vec};
use std::c_str::CString;

/// Human-readable major version string
pub static VERSION: &'static str = config::LUA_VERSION;
/// Human-readable release version string
pub static RELEASE: &'static str = config::LUA_RELEASE;
/// Machine-readable version number
pub static VERSION_NUM: int = config::LUA_VERSION_NUM as int;

/// Value for lua_call that means return all results
pub static MULTRET: i32 = raw::MULTRET as i32;

/// Minimum Lua stack available to a C function
pub static MINSTACK: i32 = config::LUA_MINSTACK as i32;

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

pub mod config;

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
                // NB: lua_typename() doesn't actually use its state parameter
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

/// Function type for reading blocks when loading Lua chunks.
pub type Reader = raw::lua_Reader;

/// Function type for writing blocks when dumping Lua chunks.
pub type Writer = raw::lua_Writer;

/// Type that represents memory-allocation functions
pub type Alloc = raw::lua_Alloc;

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

impl ToStr for LoadError {
    fn to_str(&self) -> ~str {
        match *self {
            LoadError::ErrSyntax => ~"syntax error",
            LoadError::ErrMem => ~"memory allocation error"
        }
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

impl ToStr for LoadFileError {
    fn to_str(&self) -> ~str {
        match *self {
            LoadFileError::ErrSyntax => ~"syntax error",
            LoadFileError::ErrMem => ~"memory allocation error",
            LoadFileError::ErrFile => ~"file read/open error"
        }
    }
}

/// State.pcall() errors
pub type PCallError = PCallError::PCallError;
pub mod PCallError {
    //! State.pcall() error mod
    use raw;
    use libc::c_int;
    /// State.pcall() errors
    pub enum PCallError {
        /// Runtime error
        ErrRun = raw::LUA_ERRRUN,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM,
        /// Error while running the error handler function
        ErrErr = raw::LUA_ERRERR
    }

    /// Converts an error code from `lua_pcall()` into a PCallError
    pub fn from_code(code: c_int) -> Option<PCallError> {
        match code {
            raw::LUA_ERRRUN => Some(ErrRun),
            raw::LUA_ERRMEM => Some(ErrMem),
            raw::LUA_ERRERR => Some(ErrErr),
            _ => None,
        }
    }

    impl ToStr for PCallError {
        fn to_str(&self) -> ~str {
            match *self {
                ErrRun => ~"runtime error",
                ErrMem => ~"memory allocation error",
                ErrErr => ~"error handler func error"
            }
        }
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
    priv stackspace: i32,
    priv owned: bool
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
    /* State manipulation */

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
                Some(State{ L: L, stackspace: MINSTACK, owned: true })
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
                let s = State::from_lua_State(L).describe_unchecked_stack(-1, false);
                fail!("unprotected error in call to Lua API ({})", s);
            }
        }
    }

    /// Wraps a *raw::lua_State in a State
    pub unsafe fn from_lua_State(L: *mut raw::lua_State) -> State {
        #[inline];
        State{ L: L, stackspace: MINSTACK, owned: false}
    }

    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        self.L
    }

    /// Creates a new thread, pushes it on the stack, and returns a `State` that represents this
    /// new thread. The new state returned by this function shares with the original state all
    /// global objects (such as tables), but has an independent execution stack.
    ///
    /// This new state does not get explicitly closed. Threads are subject to garbage collection,
    /// like any Lua object.
    pub fn newthread(&mut self) -> State {
        #[inline];
        unsafe { State::from_lua_State(raw::lua_newthread(self.L)) }
    }

    /// Sets a new panic function and returns the old one.
    ///
    /// The panic function can access the error message at the top of the stack.
    ///
    /// The default panic function installed by this library calls fail!() with the error message.
    /// Your panic function should either call through to the default one, or should fail!()
    /// itself. Otherwise, the application will be terminated.
    pub unsafe fn atpanic(&mut self, panicf: CFunction) -> CFunction {
        #[inline];
        raw::lua_atpanic(self.L, panicf)
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
    /// Returns "" if the given index is non-valid.
    pub fn describe(&mut self, idx: i32) -> ~str {
        #[inline];
        self.check_acceptable(idx);
        self.checkstack_(1);
        unsafe { self.describe_unchecked(idx) }
    }

    /// Unchecked variant of describe()
    /// May require 1 extra slot on the stack.
    pub unsafe fn describe_unchecked(&mut self, idx: i32) -> ~str {
        #[inline];
        self.describe_unchecked_stack(idx, true)
    }

    /// Variant of describe_unchecked() that does not push on to the stack.
    /// describe() and describe_unchecked() may push new values onto the stack temporarily.
    /// Notably, it may do this to avoid converting the existing value's type.
    /// This method allows this behavior to be disabled.
    /// If usestack is on, this method may require 1 free slot on the stack.
    pub unsafe fn describe_unchecked_stack(&mut self, idx: i32, usestack: bool) -> ~str {
        match self.type_unchecked(idx) {
            None => ~"",
            Some(typ) => match typ {
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
            }
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
        luaassert!(self, self.checkstack(extra), "checkstack: cannot grow stack")
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
        luaassert!(self, self.gettop() >= n, "xmove: stack underflow");
        to.checkstack_(1);
        self.xmove_unchecked(to, n)
    }

    /// Unchecked variant of xmove()
    pub unsafe fn xmove_unchecked(&mut self, to: &mut State, n: i32) {
        #[inline];
        raw::lua_xmove(self.L, to.L, n as c_int)
    }

    /* Access functions */

    /// Returns `true` if the value at the given acceptable index is a number, or a string
    /// convertible to a number.
    pub fn isnumber(&mut self, idx: i32) -> bool {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.isnumber_unchecked(idx) }
    }

    /// Unchecked variant of isnumber()
    pub unsafe fn isnumber_unchecked(&mut self, idx: i32) -> bool {
        #[inline];
        raw::lua_isnumber(self.L, idx as c_int) != 0
    }

    /// Returns `true` if the value at the given acceptable index is a string or a number
    /// (which is always convertible to a string).
    pub fn isstring(&mut self, idx: i32) -> bool {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.isstring_unchecked(idx) }
    }

    /// Unchecked variant of isstring()
    pub unsafe fn isstring_unchecked(&mut self, idx: i32) -> bool {
        #[inline];
        raw::lua_isstring(self.L, idx as c_int) != 0
    }

    /// Returns `true` if the value at the given acceptable index is a C function.
    pub fn iscfunction(&mut self, idx: i32) -> bool {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.iscfunction_unchecked(idx) }
    }

    /// Unchecked variant of iscfunction()
    pub unsafe fn iscfunction_unchecked(&mut self, idx: i32) -> bool {
        #[inline];
        raw::lua_iscfunction(self.L, idx as c_int) != 0
    }

    /// Returns `true` if the value at the given acceptable index is a userdata
    /// (either full or light).
    pub fn isuserdata(&mut self, idx: i32) -> bool {
        #[inline];
        self.check_acceptable(idx);
        unsafe { self.isuserdata_unchecked(idx) }
    }

    /// Unchecked variant of isuserdata()
    pub unsafe fn isuserdata_unchecked(&mut self, idx: i32) -> bool {
        #[inline];
        raw::lua_isuserdata(self.L, idx as c_int) != 0
    }

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

            _ => self.errorstr("type: Unknown return value from lua_type")
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

    /// Returns `true` if the two values in acceptable indices `index1` and `index2` are equal,
    /// following the semantics of the Lua == operator. Returns `false` if any indices are
    /// non-valid.
    pub fn equal(&mut self, index1: i32, index2: i32) -> bool {
        #[inline];
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { self.equal_unchecked(index1, index2) }
    }

    /// Unchecked variant of equal()
    pub unsafe fn equal_unchecked(&mut self, index1: i32, index2: i32) -> bool {
        #[inline];
        raw::lua_equal(self.L, index1 as c_int, index2 as c_int) != 0
    }

    /// Returns `true` if the two values in acceptable indices `index1` and `index2` are
    /// primitively equal (that is, without calling any metamethods). Returns `false` if any
    /// indices are non-valid.
    pub fn rawequal(&mut self, index1: i32, index2: i32) -> bool {
        #[inline];
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { self.rawequal_unchecked(index1, index2) }
    }

    /// Unchecked variant of rawequal()
    pub unsafe fn rawequal_unchecked(&mut self, index1: i32, index2: i32) -> bool {
        #[inline];
        raw::lua_rawequal(self.L, index1 as c_int, index2 as c_int) != 0
    }

    /// Returns `true` if the value at acceptable index `index1` is smaller than the value at
    /// acceptable index `index2`, following the semantics of the Lua < operator. Returns `false`
    /// if any indices are non-valid.
    pub fn lessthan(&mut self, index1: i32, index2: i32) -> bool {
        #[inline];
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { self.lessthan_unchecked(index1, index2) }
    }

    /// Unchecked variant of lessthan()
    pub unsafe fn lessthan_unchecked(&mut self, index1: i32, index2: i32) -> bool {
        raw::lua_lessthan(self.L, index1 as c_int, index2 as c_int) != 0
    }

    /// Converts the Lua value at the given acceptable index to a f64. The Lua value must be a
    /// number or a string convertible to a number; otherwise, tonumber returns 0.
    pub fn tonumber(&mut self, index: i32) -> f64 {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.tonumber_unchecked(index) }
    }

    /// Unchecked variant of tonumber()
    pub unsafe fn tonumber_unchecked(&mut self, index: i32) -> f64 {
        #[inline];
        raw::lua_tonumber(self.L, index as c_int) as f64
    }

    /// Converts the Lua value at the given acceptable index to an int. The Lua value must be a
    /// number or a string convertiable to a number; otherwise, toint returns 0.
    pub fn tointeger(&mut self, index: i32) -> int {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.tointeger_unchecked(index) }
    }

    /// Unchecked variant of tointeger()
    pub unsafe fn tointeger_unchecked(&mut self, index: i32) -> int {
        #[inline];
        raw::lua_tointeger(self.L, index as c_int) as int
    }

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
                Some(cast::transmute::<&[u8], &'a [u8]>(b))
            })
        }
    }

    /// Returns the "length" of the value at the given acceptable index.
    pub fn objlen(&mut self, index: i32) -> uint {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.objlen_unchecked(index) }
    }

    /// Unchecked variant of objlen()
    pub unsafe fn objlen_unchecked(&mut self, index: i32) -> uint {
        #[inline];
        raw::lua_objlen(self.L, index as c_int) as uint
    }

    /// Converts a value at the given acceptable index to a C function. The value must be a
    /// C function; otherwise, returns None.
    pub fn tocfunction(&mut self, index: i32) -> Option<CFunction> {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.tocfunction_unchecked(index) }
    }

    /// Unchecked variant of tocfunction()
    pub unsafe fn tocfunction_unchecked(&mut self, index: i32) -> Option<CFunction> {
        #[inline];
        raw::lua_tocfunction(self.L, index as c_int)
    }

    /// If the value at the given acceptable index is a full userdata, returns its block address.
    /// If the value is a light userdata, returns its pointer. Otherwise, returns None.
    pub fn touserdata(&mut self, index: i32) -> Option<*mut libc::c_void> {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.touserdata_unchecked(index) }
    }

    /// Unchecked variant of touserdata()
    pub unsafe fn touserdata_unchecked(&mut self, index: i32) -> Option<*mut libc::c_void> {
        #[inline];
        let ud = raw::lua_touserdata(self.L, index as c_int);
        if ud.is_null() {
            None
        } else {
            Some(ud)
        }
    }

    /// Converts the value at the given acceptable index to a Lua thread (represented as a State).
    /// This value must be a thread; otherwise, the method returns None.
    ///
    /// Note: the State return value does not make any assumptions about the available stack space.
    /// .checkstack() must be called in order to consider any non-valid index as acceptable.
    pub fn tothread(&mut self, index: i32) -> Option<State> {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.tothread_unchecked(index) }
    }

    /// Unchecked variant of tothread()
    pub unsafe fn tothread_unchecked(&mut self, index: i32) -> Option<State> {
        #[inline];
        let s = raw::lua_tothread(self.L, index as c_int);
        if s.is_null() {
            None
        } else {
            Some(State::from_lua_State(s))
        }
    }

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
    pub unsafe fn pushstring_unchecked(&mut self, s: &str) {
        #[inline];
        raw::lua_pushlstring(self.L, s.as_ptr() as *libc::c_char, s.len() as libc::size_t)
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
        raw::lua_pushlstring(self.L, bytes.as_ptr() as *libc::c_char, bytes.len() as libc::size_t)
    }

    /// Pushes a new C closure onto the stack.
    ///
    /// When a C function is created, it is possible to associate some values with it, thus
    /// creating a C closure; these values are then accessible to the function whenever it is
    /// called. These values must be pushed onto the stack (in order), then pushclosure() is called
    /// to create and push the C closure onto the stack. The argument `n` is the number of values
    /// that should be associated with the function. These values are popped from the stack.
    ///
    /// `n` must be in the range [0, 255]. Anything outside this range will cause a failure.
    pub fn pushcclosure(&mut self, f: CFunction, n: i32) {
        #[inline];
        if n == 0 {
            self.checkstack_(1);
        } else {
            luaassert!(self, n >= 0 && n <= 255, "pushcclosure: invalid argument n");
        }
        unsafe { self.pushcclosure_unchecked(f, n) }
    }

    /// Unchecked variant of pushcclosure().
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
    pub unsafe fn pushboolean_unchecked(&mut self, b: bool) {
        #[inline];
        raw::lua_pushboolean(self.L, b as c_int)
    }

    /// Pushes a light userdata onto the stack.
    pub fn pushlightuserdata(&mut self, p: *mut libc::c_void) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushlightuserdata_unchecked(p) }
    }

    /// Unchecked variant of pushlightuserdata()
    pub unsafe fn pushlightuserdata_unchecked(&mut self, p: *mut libc::c_void) {
        #[inline];
        raw::lua_pushlightuserdata(self.L, p)
    }

    /// Pushes the thread represented by `self` onto the stack. Returns `true` if this thread
    /// is the main thread of the state.
    pub fn pushthread(&mut self) -> bool {
        #[inline];
        self.checkstack_(1);
        unsafe { self.pushthread_unchecked() }
    }

    /// Unchecked variant of pushthread()
    pub unsafe fn pushthread_unchecked(&mut self) -> bool {
        #[inline];
        raw::lua_pushthread(self.L) != 0
    }

    /* Get functions (Lua -> stack) */

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index and k is the value at the top of the stack.
    /// The key is popped from the stack.
    pub fn gettable(&mut self, idx: i32) {
        #[inline];
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "gettable: stack underflow");
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

    /// Similar to gettable(), but does a raw access
    pub fn rawget(&mut self, index: i32) {
        #[inline];
        self.check_valid(index, true);
        luaassert!(self, self.gettop() > 0, "rawget: stack underflow");
        unsafe { self.rawget_unchecked(index) }
    }

    /// Unchecked variant of rawget()
    pub unsafe fn rawget_unchecked(&mut self, index: i32) {
        #[inline];
        raw::lua_rawget(self.L, index as c_int)
    }

    /// Pushes onto the stack the value t[n], where t is the value at the given valid index.
    /// The access is raw; that is, it does not invoke metamethods.
    pub fn rawgeti(&mut self, index: i32, n: i32) {
        #[inline];
        self.check_valid(index, true);
        self.checkstack_(1);
        unsafe { self.rawgeti_unchecked(index, n) }
    }

    /// Unchecked variant of rawgeti()
    pub unsafe fn rawgeti_unchecked(&mut self, index: i32, n: i32) {
        #[inline];
        raw::lua_rawgeti(self.L, index as c_int, n as c_int)
    }

    /// Creates a new empty table and pushes it into the stack. The new table has space
    /// pre-allocated for `narr` array elements and `nrec` non-array elements.
    pub fn createtable(&mut self, narr: i32, nrec: i32) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.createtable_unchecked(narr, nrec) }
    }

    /// Unchecked variant of createtable()
    pub unsafe fn createtable_unchecked(&mut self, narr: i32, nrec: i32) {
        #[inline];
        raw::lua_createtable(self.L, narr as c_int, nrec as c_int)
    }

    /// This method allocates a new block of memory with the given size, pushes onto the stack a
    /// new full userdata with the block address, and returns this address.
    pub fn newuserdata(&mut self, size: uint) -> *mut libc::c_void {
        #[inline];
        self.checkstack_(1);
        unsafe { self.newuserdata_unchecked(size) }
    }

    /// Unchecked variant of newuserdata()
    pub unsafe fn newuserdata_unchecked(&mut self, size: uint) -> *mut libc::c_void {
        #[inline];
        raw::lua_newuserdata(self.L, size as libc::size_t)
    }

    /// Pushes onto the stack the metatable of the value at the given acceptable index. If the
    /// index is not valid, or the value does not have a metatable, the function returns `false`
    /// and pushes nothing onto the stack.
    pub fn getmetatable(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        self.checkstack_(1);
        unsafe { self.getmetatable_unchecked(index) }
    }

    /// Unchecked variant of getmetatable()
    pub unsafe fn getmetatable_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_getmetatable(self.L, index as c_int) != 0
    }

    /// Pushes onto the stack the environment table of the value at the given index.
    pub fn getfenv(&mut self, index: i32) {
        #[inline];
        self.check_acceptable(index);
        self.checkstack_(1);
        unsafe { self.getfenv_unchecked(index) }
    }

    /// Unchecked variant of getfenv()
    pub unsafe fn getfenv_unchecked(&mut self, index: i32) {
        #[inline];
        raw::lua_getfenv(self.L, index as c_int)
    }

    /* Set functions (stack -> Lua) */

    /// Does the equivalent to t[k] = v, where t is the value at the given valid index, v is the
    /// value at the top of the stack, and k is the value just below the top.
    ///
    /// This function pops both the key and the value from the stack.
    pub fn settable(&mut self, index: i32) {
        #[inline];
        self.check_valid(index, true);
        luaassert!(self, self.gettop() >= 2, "settable: stack underflow");
        unsafe { self.settable_unchecked(index) }
    }

    /// Unchecked variant of settable()
    pub unsafe fn settable_unchecked(&mut self, index: i32) {
        #[inline];
        raw::lua_settable(self.L, index as c_int)
    }

    /// Does the equivalent to t[k] = v, where t is the value at the given valid index and v is
    /// the value at the top of the stack.
    ///
    /// This function pops the value from the stack.
    ///
    /// Raises the `c_str::null_byte` condition if `k` contains interior NULs.
    pub fn setfield(&mut self, index: i32, k: &str) {
        #[inline];
        self.check_valid(index, true);
        luaassert!(self, self.gettop() >= 1, "setfield: stack underflow");
        unsafe { self.setfield_unchecked(index, k) }
    }

    /// Unchecked variant of setfield()
    pub unsafe fn setfield_unchecked(&mut self, index: i32, k: &str) {
        k.with_c_str(|kp| raw::lua_setfield(self.L, index as c_int, kp))
    }

    /// Similar to settable(), but does a raw assignment.
    pub fn rawset(&mut self, index: i32) {
        #[inline];
        self.check_valid(index, true);
        luaassert!(self, self.gettop() >= 2, "rawset: stack underflow");
        unsafe { self.rawset_unchecked(index) }
    }

    /// Unchecked variant of rawset()
    pub unsafe fn rawset_unchecked(&mut self, index: i32) {
        #[inline];
        raw::lua_rawset(self.L, index as c_int)
    }

    /// Does the equivalent of t[n] = v, where t is the value at the given valid index and v is
    /// the value at the top of the stack.
    ///
    /// This function pops the value from the stack. The assignment is raw; that is, it does not
    /// invoke metamethods.
    pub fn rawseti(&mut self, index: i32, n: i32) {
        #[inline];
        self.check_valid(index, true);
        unsafe { self.rawseti_unchecked(index, n) }
    }

    /// Unchecked variant of rawseti()
    pub unsafe fn rawseti_unchecked(&mut self, index: i32, n: i32) {
        #[inline];
        raw::lua_rawseti(self.L, index as c_int, n as c_int)
    }

    /// Pops a table from the stack and sets it as the new metatable for the value at the given
    /// acceptable index.
    pub fn setmetatable(&mut self, index: i32) {
        #[inline];
        self.check_acceptable(index);
        luaassert!(self, self.istable(-1), "setmetatable: top stack value must be a table")
        unsafe { self.setmetatable_unchecked(index) }
    }

    /// Unchecked variant of setmetatable()
    pub unsafe fn setmetatable_unchecked(&mut self, index: i32) {
        #[inline];
        // ignore return value of lua_setmetatable(), it appears to always be 1
        raw::lua_setmetatable(self.L, index as c_int);
    }

    /// Pops a table from the stack and sets it as the new environment for the value at the given
    /// index. If the value at the given index is neither a function nor a thread nor a userdata,
    /// setfenv() returns `false`. Otherwise, returns `true`.
    pub fn setfenv(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        luaassert!(self, self.istable(-1), "setfenv: top stack value must be a table");
        unsafe { self.setfenv_unchecked(index) }
    }

    /// Unchecked variant of setfenv()
    pub unsafe fn setfenv_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_setfenv(self.L, index as c_int) != 0
    }

    /* `load` and `call` functions (load and run Lua code) */

    /// Calls a function.
    /// The function must be pushed first, followed by its arguments. `nargs` is the number of
    /// arguments. The function and its arguments are popped automatically.
    /// The function results are adjusted to `nresults`, unless `nresults` is `MULTRET`, in which
    /// case all function results are pushed.
    pub fn call(&mut self, nargs: i32, nresults: i32) {
        #[inline];
        luaassert!(self, nargs >= 0, "call: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "call: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "call: stack underflow");
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
        luaassert!(self, nargs >= 0, "pcall: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "pcall: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "pcall: stack underflow");
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
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("pcall: unexpected error from lua_pcall")
            }))
        }
    }

    // Don't bother wrapping cpcall, userdatas are unsafe in Rust and the better approach is just
    // to call .pushcfunction() and .pcall().

    /// Loads a Lua chunk. If there are no errors, load() pushes the compiled chunk as a Lua
    /// function on top of the stack. Otherwise, it pushes an error message.
    ///
    /// This method only loads a chunk; it does not run it.
    ///
    /// load() automatically detects whether the chunk is text or binary, and loads it accordingly.
    ///
    /// The load() method uses a user-supplied `reader` function to read the chunk. The `data`
    /// argument is an opaque value passed to the reader function.
    ///
    /// The `chunkname` argument gives a name to the chunk, which is used for error messages and
    /// in debug information.
    ///
    /// Raises the `c_str::null_byte` condition if `chunkname` contains interior NULs.
    pub fn load(&mut self, reader: Reader, data: *mut libc::c_void, chunkname: &str)
               -> Result<(),LoadError> {
        #[inline];
        self.checkstack_(1);
        unsafe { self.load_unchecked(reader, data, chunkname) }
    }

    /// Unchecked variant of load()
    pub unsafe fn load_unchecked(&mut self, reader: Reader, data: *mut libc::c_void,
                                 chunkname: &str) -> Result<(),LoadError> {
        match chunkname.with_c_str(|name| raw::lua_load(self.L, reader, data, name)) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("load: unexpected error from lua_load")
        }
    }

    /// Dumps a function as a binary chunk. Receives a Lua function on the top of the stack and
    /// produces a binary chunk that, if loaded again, results in a function equivalent to the
    /// one dumped. As it produces parts of the chunk, dump() calls function `writer` with the
    /// given `data` tow rite them.
    ///
    /// The value returned is the error code returned by the last call to the writer; Ok(()) means
    /// no errors.
    ///
    /// Thisf unction does not pop the Lua function from the stack.
    pub fn dump(&mut self, writer: Writer, data: *mut libc::c_void) -> Result<(),i32> {
        #[inline];
        luaassert!(self, self.gettop() >= 1, "dump: stack underflow");
        unsafe { self.dump_unchecked(writer, data) }
    }

    /// Unchecked variant of dump()
    pub unsafe fn dump_unchecked(&mut self, writer: Writer, data: *mut libc::c_void)
                                -> Result<(),i32> {
        #[inline];
        match raw::lua_dump(self.L, writer, data) {
            0 => Ok(()),
            i => Err(i)
        }
    }

    /* Coroutine functions */

    /// Yields a coroutine.
    ///
    /// This function should only be called as the return expression of a C function, as follows:
    ///
    ///   return L.yield_(nresults);
    ///
    /// When a C function calls yield_() in that way, the running coroutine suspends its execution,
    /// and the call to resume() that started this coroutine returns. The parameter `nresults` is
    /// the number of values from the stack that are passed as the results to resume().
    pub fn yield_(&mut self, nresults: i32) -> c_int {
        #[inline];
        luaassert!(self, self.gettop() >= nresults, "yield: stack underflow");
        unsafe { self.yield_unchecked(nresults) }
    }

    /// Unchecked variant of yield_()
    pub unsafe fn yield_unchecked(&mut self, nresults: i32) -> c_int {
        #[inline];
        raw::lua_yield(self.L, nresults as c_int)
    }

    /// Starts and resumes a coroutine in a given thread.
    ///
    /// To start a coroutine, you first create a new thread (see thread()); then you push onto its
    /// stack the main function plus any arguments; then you call resume(), with `narg` being the
    /// number of arguments. This call returns when the coroutine suspends or finishes its
    /// execution. When it returns, the stack contains all values passed to yield_(), or all values
    /// returned by the body function. resume() returns Ok(false) if the coroutine yields,
    /// Ok(true) if the coroutine finishes its execution without errors, or Err(PCallError) in
    /// case of errors. In case of errors, the stack is not unwound, so you can use the debug API
    /// over it. The error message is on top of the stack. To restart a coroutine, you put on its
    /// stack only the values to be passed as results from yield_(), and then call resume().
    pub fn resume(&mut self, narg: i32) -> Result<bool,PCallError> {
        #[inline];
        luaassert!(self, self.gettop() > narg, "resume: stack underflow");
        unsafe { self.resume_unchecked(narg) }
    }

    /// Unchecked variant of resume()
    pub unsafe fn resume_unchecked(&mut self, narg: i32) -> Result<bool,PCallError> {
        #[inline];
        match raw::lua_resume(self.L, narg as c_int) {
            raw::LUA_YIELD => Ok(false),
            0 => Ok(true),
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("resume: unexpected error from lua_resume")
            }))
        }
    }

    /// Returns the status of the receiving thread.
    ///
    /// The status can be Ok(true) for a normal thread, Ok(false) if the thread is suspended, or
    /// Err(PCallError) if the thread finished its execution with an error.
    pub fn status(&mut self) -> Result<bool,PCallError> {
        #[inline];
        match unsafe { raw::lua_status(self.L) } {
            raw::LUA_YIELD => Ok(false),
            0 => Ok(true),
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("status: unexpected error from lua_status")
            }))
        }
    }

    /* Garbage collection functions */

    /// Controls the garbage collector.
    ///
    /// This method performs several tasks, according to the value of the parameter `what`.
    /// See the `GC` enum for documentation on the various options.
    pub fn gc(&mut self, what: GC, data: i32) -> i32 {
        #[inline];
        unsafe { raw::lua_gc(self.L, what as c_int, data as c_int) as i32 }
    }

    /* Miscellaneous functions */

    /// Raises an error (using the value at the top of the stack)
    pub fn error(&mut self) -> ! {
        #[inline];
        luaassert!(self, self.gettop() > 0, "error: stack underflow");
        unsafe { self.error_unchecked() }
    }

    /// Unchecked variant of error()
    pub unsafe fn error_unchecked(&mut self) -> ! {
        #[inline];
        raw::lua_error(self.L);
        unreachable!()
    }

    /// Pops a key from the stack, and pushes a key-value pair from the table at the given index
    /// (the "next" pair after the given key). If there are no more elements in the table, then
    /// next() returns false (and pushes nothing).
    ///
    /// A typical traversal looks like this:
    ///
    ///   /* table is in the stack at index 't' */
    ///   L.pushnil(); // first key
    ///   while L.next(t) {
    ///     /* uses 'key' (at index -2) and 'value' (at index -1) */
    ///     println!("{} - {}", L.typename(-2), L.typename(-1));
    ///     /* removes 'value'; keeps 'key' for next iteration */
    ///     L.pop(1);
    ///   }
    ///
    /// While traversing a table, do not call tostring() or tobytes() directly on a key, unless
    /// you know that the key is actually a string. Recall that tostring() changes the value at
    /// the given index; this confuses the next call to next().
    pub fn next(&mut self, index: i32) -> bool {
        #[inline];
        self.check_valid(index, true);
        unsafe { self.next_unchecked(index) }
    }

    /// Unchecked variant of next()
    pub unsafe fn next_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_next(self.L, index as c_int) != 0
    }

    /// Concatenates the `n` values at the top of the stack, pops them, and
    /// leaves the result at the top.
    /// Fails if n is negative or larger than the stack top.
    pub fn concat(&mut self, n: i32) {
        #[inline];
        luaassert!(self, n >= 0, "concat: invalid argument n");
        luaassert!(self, n <= self.gettop(), "concat: stack underflow");
        if n == 0 { self.checkstack_(1) }
        unsafe { self.concat_unchecked(n) }
    }

    /// Unchecked variant of concat()
    pub unsafe fn concat_unchecked(&mut self, n: i32) {
        #[inline];
        raw::lua_concat(self.L, n as c_int)
    }

    /// Returns the memory-allocation function of a given state. If `ud` is not NULL, Lua stores
    /// in `*ud` the opaque pointer passed to lua_newstate().
    ///
    /// Note: State::new() always provides NULL as the opaque pointer. It also provides a default
    /// alloc function that behaves identically to the one used by luaL_newstate().
    pub fn getallocf(&mut self, ud: *mut *mut libc::c_void) -> Alloc {
        #[inline];
        unsafe { raw::lua_getallocf(self.L, ud) }
    }

    /// Changes the allocator function of a given state to `f` with user data `ud`.
    pub unsafe fn setallocf(&mut self, f: Alloc, ud: *mut libc::c_void) {
        #[inline];
        raw::lua_setallocf(self.L, f, ud)
    }

    /* Some useful functions (macros in C) */

    /// Pop n elements from the stack.
    /// Fails if the stack is smaller than n
    pub fn pop(&mut self, n: i32) {
        #[inline];
        if n >= 0 {
            luaassert!(self, self.gettop() >= n, "pop: stack underflow");
        } else {
            luaassert!(self, self.gettop() >= (n+1).abs(), "pop: stack underflow");
        }
        unsafe { self.pop_unchecked(n) }
    }

    /// Unchecked variant of pop()
    pub unsafe fn pop_unchecked(&mut self, n: i32) {
        #[inline];
        raw::lua_pop(self.L, n as c_int)
    }

    /// Creates a new empty table and pushes it onto the stack.
    /// It is equivalent to .createtable(0, 0).
    pub fn newtable(&mut self) {
        #[inline];
        self.checkstack_(1);
        unsafe { self.newtable_unchecked() }
    }

    /// Unchecked variant of newtable()
    pub unsafe fn newtable_unchecked(&mut self) {
        #[inline];
        raw::lua_newtable(self.L)
    }

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

    /// Returns `true` if the value at the given acceptable index is a function
    /// (either C or Lua).
    pub fn isfunction(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isfunction_unchecked(index) }
    }

    /// Unchecked variant of isfunction()
    pub unsafe fn isfunction_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isfunction(self.L, index as c_int)
    }


    /// Returns `true` if the value at the given acceptable index is a table.
    pub fn istable(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.istable_unchecked(index) }
    }

    /// Unchecked variant of istable()
    pub unsafe fn istable_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_istable(self.L, index as c_int)
    }

    /// Returns `true` if the value at the given acceptable index is a light userdata.
    pub fn islightuserdata(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.islightuserdata_unchecked(index) }
    }

    /// Unchecked variant of islightuserdata()
    pub unsafe fn islightuserdata_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_islightuserdata(self.L, index)
    }

    /// Returns `true` if the value at the given acceptable index is `nil`.
    pub fn isnil(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isnil_unchecked(index) }
    }

    /// Unchecked variant of isnil()
    pub unsafe fn isnil_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isnil(self.L, index)
    }

    /// Returns `true` if the value at the given acceptable index has type boolean.
    pub fn isboolean(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isboolean_unchecked(index) }
    }

    /// Unchecked variant of isboolean()
    pub unsafe fn isboolean_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isboolean(self.L, index)
    }

    /// Returns `true` if the value at the given acceptable index is a thread.
    pub fn isthread(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isthread_unchecked(index) }
    }

    /// Unchecked variant of isthread()
    pub unsafe fn isthread_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isthread(self.L, index)
    }

    /// Returns `true` if the given acceptable index is not valid.
    pub fn isnone(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isnone_unchecked(index) }
    }

    /// Unchecked variant of isnone()
    pub unsafe fn isnone_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isnone(self.L, index)
    }

    /// Returns `true` if the given acceptable index is not valid or if the value at this index
    /// is nil.
    pub fn isnoneornil(&mut self, index: i32) -> bool {
        #[inline];
        self.check_acceptable(index);
        unsafe { self.isnoneornil_unchecked(index) }
    }

    /// Unchecked variant of isnoneornil()
    pub unsafe fn isnoneornil_unchecked(&mut self, index: i32) -> bool {
        #[inline];
        raw::lua_isnoneornil(self.L, index)
    }

    /// Pops a value from the stack and sets it as the new value of global `name`.
    /// Raises the `c_str::null_byte` condition if `name` has interior NULs.
    pub fn setglobal(&mut self, name: &str) {
        #[inline];
        luaassert!(self, self.gettop() > 0, "setglobal: stack underflow");
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
        self.pushcfunction_unchecked(lib::raw::luaopen_base);
        self.pushstring_unchecked("");
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
        self.pushcfunction_unchecked(lib::raw::luaopen_table);
        self.pushstring_unchecked(TABLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_io);
        self.pushstring_unchecked(IOLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_os);
        self.pushstring_unchecked(OSLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_string);
        self.pushstring_unchecked(STRLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_math);
        self.pushstring_unchecked(MATHLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_debug);
        self.pushstring_unchecked(DBLIBNAME);
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
        self.pushcfunction_unchecked(lib::raw::luaopen_package);
        self.pushstring_unchecked(LOADLIBNAME);
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

pub static NoRef: i32 = aux::raw::LUA_NOREF as i32;
pub static RefNil: i32 = aux::raw::LUA_REFNIL as i32;

// Functions from auxlib
impl State {
    /// Opens a library.
    ///
    /// When called with `libname` equal to None, it simply registers all functions in the list `l`
    /// into the table on the top of the stack.
    ///
    /// When called with a `libname` of Some(_), registerlib() creates a new table `t`, sets it as
    /// the value of the global variable `libname`, sets it as the value of
    /// `package.loaded[libname]`, and registers on it all functions in the list `l`. If there is
    /// a table in `package.loaded[libname]` or in variable `libname`, reuses this table instead
    /// of creating a new one.
    ///
    /// In any case the function leaves the table on the top of the stack.
    pub fn registerlib(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        #[inline];
        // internally, luaL_registerlib seems to use 4 stack slots
        self.checkstack_(4);
        if libname.is_none() {
            luaassert!(self, self.gettop() >= 1, "registerlib: stack underflow");
        }
        unsafe { self.registerlib_unchecked(libname, l) }
    }

    /// Unchecked variant of registerlib()
    pub unsafe fn registerlib_unchecked(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        let mut cstrs = vec::with_capacity(l.len());
        let mut l_ = vec::with_capacity(l.len()+1);
        for &(name, func) in l.iter() {
            let cstr = name.to_c_str();
            cstr.with_ref(|name| l_.push(aux::raw::luaL_Reg{ name: name, func: Some(func) }));
            cstrs.push(cstr);
        }
        l_.push(aux::raw::luaL_Reg{ name: ptr::null(), func: None });
        let libcstr = libname.map(|s| s.to_c_str());
        let libname_ = libcstr.map_or(ptr::null(), |cstr| cstr.with_ref(|p| p));
        aux::raw::luaL_register(self.L, libname_, l_.as_ptr())
    }

    /// Pushes onto the stack the field `e` from the metatable of the object at index `obj`. If
    /// the object does not have a metatable, or if the metatable does not have this field,
    /// returns `false` and pushes nothing.
    pub fn getmetafield(&mut self, obj: i32, e: &str) -> bool {
        #[inline];
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_getmetafield uses 2 stack slots
        unsafe { self.getmetafield_unchecked(obj, e) }
    }

    /// Unchecked variant of getmetafield()
    pub unsafe fn getmetafield_unchecked(&mut self, obj: i32, e: &str) -> bool {
        #[inline];
        e.with_c_str(|e| aux::raw::luaL_getmetafield(self.L, obj as c_int, e)) != 0
    }

    /// Calls a metamethod.
    ///
    /// If the object at index `obj` has a metatable and this metatable has a field `e`, this
    /// method calls this field and passes the object as its only argument. In this case this
    /// method returns `true` and pushes onto the stack the value returned by the call. If there
    /// is no metatable or no metamethod, this method returns `false` (without pushing any value
    /// on the stack).
    pub fn callmeta(&mut self, obj: i32, e: &str) -> bool {
        #[inline];
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_callmeta uses 2 stack slots
        unsafe { self.callmeta_unchecked(obj, e) }
    }

    /// Unchecked variant of callmeta()
    pub unsafe fn callmeta_unchecked(&mut self, obj: i32, e: &str) -> bool {
        #[inline];
        e.with_c_str(|e| aux::raw::luaL_callmeta(self.L, obj as c_int, e)) != 0
    }

    /// Generates an error with a message like the following:
    ///
    ///   <location>: bad argument <narg> to '<func>' (<tname> expected, got <rt>)
    ///
    /// where `location` is produced by where(), `func` is the name of the current function, and
    /// `rt` is the type name of the actual argument.
    pub fn typerror(&mut self, narg: i32, tname: &str) -> ! {
        #[inline];
        self.check_acceptable(narg);
        // NB: stack checking is not necessary
        unsafe { self.typerror_unchecked(narg, tname) }
    }

    /// Unchecked variant of typerror()
    pub unsafe fn typerror_unchecked(&mut self, narg: i32, tname: &str) -> ! {
        #[inline];
        tname.with_c_str(|tname| aux::raw::luaL_typerror(self.L, narg as c_int, tname));
        unreachable!()
    }

    /// Raises an error with the following message, where `func` is taken from the call stack:
    ///
    ///   bad argument #<narg> to <func> (<extramsg>)
    pub fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        #[inline];
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
            cast::transmute::<&[u8], &'a [u8]>(b)
        })
    }

    /// If the function argument `narg` is a string, returns this string. If this argument is
    /// absent or is nil, returns `d`. Otherwise, raises an error.
    ///
    /// If the argument is a string, but is not utf-8, returns None.
    pub fn optstring<'a>(&'a mut self, narg: i32, d: &'static str) -> Option<&'a str> {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.optstring_unchecked(narg, d) }
    }

    /// Unchecked variant of optstring()
    pub unsafe fn optstring_unchecked<'a>(&'a mut self, narg: i32, d: &'static str)
                                         -> Option<&'a str> {
        #[inline];
        str::from_utf8_opt(self.optbytes_unchecked(narg, d.as_bytes()))
    }

    /// If the function argument `narg` is a lua string, returns this string asa byte vector.
    /// See optstring() for more information.
    pub fn optbytes<'a>(&'a mut self, narg: i32, d: &'static [u8]) -> &'a [u8] {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.optbytes_unchecked(narg, d) }
    }

    /// Unchecked variant of optbytes()
    pub unsafe fn optbytes_unchecked<'a>(&'a mut self, narg: i32, d: &'static [u8]) -> &'a [u8] {
        let mut sz: libc::size_t = 0;
        let s = d.with_c_str(|d| aux::raw::luaL_optlstring(self.L, narg, d, &mut sz));
        vec::raw::buf_as_slice(s as *u8, sz as uint, |b| {
            cast::transmute::<&[u8], &'a [u8]>(b)
        })
    }

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

    /// If the function argument `narg` is a number, returns this number. If the argument is
    /// absent or is nil, returns `d`. Otherwise, raises an error.
    pub fn optnumber(&mut self, narg: i32, d: f64) -> f64 {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.optnumber_unchecked(narg, d) }
    }

    /// Unchecked variant of optnumber()
    pub unsafe fn optnumber_unchecked(&mut self, narg: i32, d: f64) -> f64 {
        #[inline];
        aux::raw::luaL_optnumber(self.L, narg as c_int, d as raw::lua_Number) as f64
    }

    /// Checks whether the function argument `narg` is a number and returns it as an int.
    pub fn checkinteger(&mut self, narg: i32) -> int {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checkinteger_unchecked(narg) }
    }

    /// Unchecked variant of checkinteger()
    pub unsafe fn checkinteger_unchecked(&mut self, narg: i32) -> int {
        #[inline];
        aux::raw::luaL_checkinteger(self.L, narg as c_int) as int
    }

    /// If the function argument `narg` is a number, returns this number cast to an int. If this
    /// argument is absent or nil, returns `d`. Otherwise, raises an error.
    pub fn optinteger(&mut self, narg: i32, d: int) -> int {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.optinteger_unchecked(narg, d) }
    }

    /// Unchecked variant of optinteger()
    pub unsafe fn optinteger_unchecked(&mut self, narg: i32, d: int) -> int {
        #[inline];
        aux::raw::luaL_optinteger(self.L, narg as c_int, d as raw::lua_Integer) as int
    }

    /// Checks whether the function argument `narg` has type `t`.
    pub fn checktype(&mut self, narg: i32, t: Type) {
        #[inline];
        self.check_acceptable(narg);
        unsafe { self.checktype_unchecked(narg, t) }
    }

    /// Unchecked variant of checktype()
    pub unsafe fn checktype_unchecked(&mut self, narg: i32, t: Type) {
        #[inline];
        aux::raw::luaL_checktype(self.L, narg as c_int, t as c_int)
    }

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

    /// If the registry already has the key `tname`, returns `false`. Otherwise, creates a new
    /// table to be used as a metatable for userdata, adds it to the registry with key `tname`,
    /// and returns `true`.
    ///
    /// In both cases pushes onto the stack the final value associated with `tname` in the registry.
    pub fn newmetatable(&mut self, tname: &str) -> bool {
        #[inline];
        self.checkstack_(2); // uses 1 or 2 stack slots internally
        unsafe { self.newmetatable_unchecked(tname) }
    }

    /// Unchecked variant of newmetatable()
    pub unsafe fn newmetatable_unchecked(&mut self, tname: &str) -> bool {
        #[inline];
        tname.with_c_str(|tname| aux::raw::luaL_newmetatable(self.L, tname)) != 0
    }

    /// Checks whether the function argument `narg` is a userdata of the type `tname` (see
    /// newmetatable()). The userdata pointer is returned.
    pub fn checkudata(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        #[inline];
        self.check_acceptable(narg);
        self.checkstack_(2); // uses 2 stack slots internally
        unsafe { self.checkudata_unchecked(narg, tname) }
    }

    /// Unchecked variant of checkudata()
    pub unsafe fn checkudata_unchecked(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        #[inline];
        tname.with_c_str(|tname| aux::raw::luaL_checkudata(self.L, narg as c_int, tname))
    }

    /// Pushes onto the stack a string identifying the current position of the
    /// control at level `lvl` in the call stack.
    /// Level 0 is the running function, level 1 is the function that called
    /// the running function, etc.
    pub fn where(&mut self, lvl: i32) {
        #[inline];
        // luaL_where() internally uses lua_pushfstring(), which manages stack size itself
        // so we don't need to call checkstack()
        unsafe { aux::raw::luaL_where(self.L, lvl as c_int) }
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
        self.where(1);
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
        let defp = def_cstr.as_ref().map_or(ptr::null(), |c| c.with_ref(|p| p));
        let mut lst_cstrs = vec::with_capacity(lst.len());
        let mut lstv = vec::with_capacity(lst.len()+1);
        for &(k,_) in lst.iter() {
            let cstr = k.to_c_str();
            let p = cstr.with_ref(|p| p);
            lst_cstrs.push(cstr);
            lstv.push(p);
        }
        lstv.push(ptr::null());
        let i = aux::raw::luaL_checkoption(self.L, narg as c_int, defp, lstv.as_ptr());
        lst[i].second_ref()
    }

    /// Creates and returns a reference, in the table at index `t`, for the object at the top of
    /// the stack (and pops the object).
    ///
    /// A reference is a unique integer key. As long as you do not manually add integer keys into
    /// table `t`, ref_() ensures the uniqueness of the key it returns. You can retrieve an object
    /// referred by reference `r` by calling `L.rawget(t, r)`. Method unref() frees a reference
    /// and its associated object.
    ///
    /// If the object at the top of the stack is nil, ref_() returns the constant RefNil. The
    /// constant NoRef is guaranteed to be different from any reference returned by ref_().
    pub fn ref_(&mut self, t: i32) -> i32 {
        #[inline];
        self.check_valid(t, true);
        self.checkstack_(1); // luaL_ref internally uses 1 stack slot
        unsafe { self.ref_unchecked(t) }
    }

    /// Unchecked variant of ref()
    pub unsafe fn ref_unchecked(&mut self, t: i32) -> i32 {
        #[inline];
        aux::raw::luaL_ref(self.L, t as c_int) as i32
    }

    /// Releases reference `r` from the table at index `t` (see ref_()). The entry is removed
    /// from the table, so that the referred object can be collected. The reference `r` is
    /// also freed to be used again.
    ///
    /// If ref is NoRef or RefNil, unref() does nothing.
    pub fn unref(&mut self, t: i32, r: i32) {
        #[inline];
        self.check_acceptable(t);
        self.checkstack_(1); // luaL_unref internally uses 1 stack slot
        unsafe { self.unref_unchecked(t, r) }
    }

    /// Unchecked variant of unref()
    pub unsafe fn unref_unchecked(&mut self, t: i32, r: i32) {
        #[inline];
        aux::raw::luaL_unref(self.L, t as c_int, r as c_int)
    }

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
        let ptr = cstr.as_ref().map_or(ptr::null(), |cstr| cstr.with_ref(|p| p));
        match aux::raw::luaL_loadfile(self.L, ptr) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadFileError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadFileError::ErrMem),
            aux::raw::LUA_ERRFILE => Err(LoadFileError::ErrFile),
            _ => self.errorstr("loadfile: unexpected error from luaL_loadfile")
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
        let bp = buff.as_ptr() as *libc::c_char;
        let bsz = buff.len() as libc::size_t;
        match name.with_c_str(|name| aux::raw::luaL_loadbuffer(self.L, bp, bsz, name)) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("loadbuffer: unexpected error from luaL_loadbuffer")
        }
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
            _ => self.errorstr("loadstring: unexpected error from luaL_loadstring")
        }
    }

    /// Creates a copy of string `s` by replacing any occurrence of the string `p` with the string
    /// `r`. Pushes the resulting string on the stack and returns it.
    pub fn gsub<'a>(&'a mut self, s: &str, p: &str, r: &str) -> &'a str {
        #[inline];
        // gsub uses Buffer internally, which uses up to MINSTACK/2 stack slots
        self.checkstack_(MINSTACK/2);
        unsafe { self.gsub_unchecked(s, p, r) }
    }

    /// Unchecked variant of gsub()
    pub unsafe fn gsub_unchecked<'a>(&'a mut self, s: &str, p: &str, r: &str) -> &'a str {
        let s_ = s.to_c_str();
        let p_ = p.to_c_str();
        let r_ = r.to_c_str();
        let sp = s_.with_ref(|p| p);
        let pp = p_.with_ref(|p| p);
        let rp = r_.with_ref(|p| p);
        let res = aux::raw::luaL_gsub(self.L, sp, pp, rp);
        let cstr = CString::new(res, false);
        let res = cstr.as_str().unwrap();
        cast::transmute::<&str,&'a str>(res)
    }

    /* Some useful functions (macros in C) */

    /// Checks whether `cond` is true. If not, raises an error with the following message, where
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

    /// Loads and runs the given file. It returns `true` if there are no errors or `false` in
    /// case of errors.
    pub fn dofile(&mut self, filename: Option<&path::Path>) -> bool {
        #[inline];
        self.checkstack_(1);
        unsafe { self.dofile_unchecked(filename) }
    }

    /// Unchecked variant of dofile()
    pub unsafe fn dofile_unchecked(&mut self, filename: Option<&path::Path>) -> bool {
        #[inline];
        let cstr = filename.map(|p| p.to_c_str());
        let name = cstr.map_or(ptr::null(), |c| c.with_ref(|p| p));
        aux::raw::luaL_dofile(self.L, name) == 0
    }

    /// Loads and runs the given string. It returns `true` if there are no errors or `false` in
    /// case of errors.
    pub fn dostring(&mut self, s: &str) -> bool {
        #[inline];
        self.checkstack_(1);
        unsafe { self.dostring_unchecked(s) }
    }

    /// Unchecked variant of dostring()
    pub unsafe fn dostring_unchecked(&mut self, s: &str) -> bool {
        #[inline];
        s.with_c_str(|s| aux::raw::luaL_dostring(self.L, s)) == 0
    }

    /// Pushes onto the stack the metatable associated with the name `tname` in the registry
    /// (see newmetatable()).
    pub fn getmetatable_reg(&mut self, tname: &str) {
        #[inline];
        self.getfield(REGISTRYINDEX, tname)
    }

    /* Generic Buffer manipulation */

    /// Initializes and returns a Buffer
    pub fn buffinit<'a>(&'a mut self) -> Buffer<'a> {
        #[inline];
        let mut B = aux::raw::luaL_Buffer{
            p: ptr::mut_null(),
            lvl: 0,
            L: self.L,
            buffer: [0, ..aux::raw::LUAL_BUFFERSIZE]
        };
        unsafe { aux::raw::luaL_buffinit(self.L, &mut B); }
        Buffer{ B: B, L: self }
    }
}

/// String buffer for building Lua strings piecemeal
pub struct Buffer<'a> {
    priv B: aux::raw::luaL_Buffer,
    /// A &mut pointer to the State that created this Buffer.
    /// The buffer internally holds on to the *lua_Buffer that the State wraps, so to ensure safety
    /// it also wraps the &mut State. Use this field to get mutable access to the State while
    /// the buffer is alive.
    L: &'a mut State
}

/// Size of the internal buffer used by Buffer and returned by prepbuffer()
pub static BUFFERSIZE: uint = aux::raw::LUAL_BUFFERSIZE as uint;

impl<'a> Buffer<'a> {
    /// Adds the byte `c` to the buffer.
    pub fn addbyte(&mut self, c: u8) {
        #[inline];
        // don't call through to luaL_addchar, because we want to insert a call to checkstack()
        // iff we have to prep the buffer.
        unsafe {
            if self.B.p >= ptr::mut_offset(&mut self.B.buffer[0], aux::raw::LUAL_BUFFERSIZE as int) {
                self.L.checkstack_(1);
                aux::raw::luaL_prepbuffer(&mut self.B);
            }
            *self.B.p = c as libc::c_char;
            self.B.p = ptr::mut_offset(self.B.p, 1);
        }
    }

    /// Adds the char `c` as utf-8 bytes to the buffer.
    pub fn addchar(&mut self, c: char) {
        #[inline];
        let mut buf = [0u8, ..4];
        let count = c.encode_utf8(buf);
        self.addbytes(buf.slice_to(count));
    }

    /// Adds to the buffer a string of length `n` previously copied to the buffer area
    /// (see prepbuffer()).
    pub fn addsize(&mut self, n: uint) {
        #[inline];
        unsafe { aux::raw::luaL_addsize(&mut self.B, n as libc::size_t) }
    }

    /// Returns a pointer to an array of size BUFFERSIZE where you can copy a string to be
    /// added to the buffer. After copying the string into this space you must call addsize()
    /// with the size of the string to actually add it to the buffer.
    pub fn prepbuffer(&mut self) -> &mut [u8, ..aux::raw::LUAL_BUFFERSIZE] {
        #[inline];
        self.L.checkstack_(1);
        unsafe { self.prepbuffer_unchecked() }
    }

    /// Unchecked variant of prepbuffer()
    pub unsafe fn prepbuffer_unchecked(&mut self) -> &mut [u8, ..aux::raw::LUAL_BUFFERSIZE] {
        // luaL_prepbuffer ends up returning the buffer field.
        // Rather than unsafely trying to transmute that to the array, just return the field
        // ourselves.
        aux::raw::luaL_prepbuffer(&mut self.B);
        cast::transmute::<&mut [i8, ..aux::raw::LUAL_BUFFERSIZE],
                          &mut [u8, ..aux::raw::LUAL_BUFFERSIZE]>(&mut self.B.buffer)
    }

    /// Adds the string to the buffer.
    pub fn addstring(&mut self, s: &str) {
        #[inline];
        self.addbytes(s.as_bytes())
    }

    /// Adds the byte vector to the buffer.
    pub fn addbytes(&mut self, bytes: &[u8]) {
        #[inline];
        // luaL_addlstring() just iterates over the string calling addchar().
        // We want our checkstack calls, so let's just do that here instead directly.
        for &b in bytes.iter() {
            self.addbyte(b);
        }
    }

    /// Adds the value at the top of the stack to the buffer. Pops the value.
    ///
    /// This is the only method on string buffers that can (and must) be called with an extra
    /// element on the stack, which is the value to be added to the buffer.
    pub fn addvalue(&mut self) {
        #[inline];
        luaassert!(self.L, self.L.gettop() >= 1, "addvalue: stack underflow");
        self.L.checkstack_(1); // luaL_addvalue() needs this if the value is too large
        unsafe { self.addvalue_unchecked() }
    }

    /// Unchecked variant of addvalue()
    pub unsafe fn addvalue_unchecked(&mut self) {
        #[inline];
        aux::raw::luaL_addvalue(&mut self.B)
    }

    /// Finishes the use of the buffer, leaving the final string on top of the stack.
    pub fn pushresult(mut self) {
        #[inline];
        self.L.checkstack_(1); // possibly needed for the emptybuffer
        unsafe { self.pushresult_unchecked() }
    }

    /// Unchecked variant of pushresult()
    pub unsafe fn pushresult_unchecked(mut self) {
        #[inline];
        aux::raw::luaL_pushresult(&mut self.B)
    }
}

/* Debug API */
/// Event codes
pub type DebugEvent = DebugEvent::DebugEvent;
pub mod DebugEvent {
    //! Mod for event codes
    use raw;
    use libc::c_int;
    /// Event codes
    pub enum DebugEvent {
        /// The call hook is called when the interpreter calls a function. The hook is called
        /// just after Lua enters the new function, before the function gets its arguments.
        HookCall = raw::LUA_HOOKCALL,
        /// The return hook is called when the interpreter returns from a function. The hook is
        /// called just before Lua leaves the function. You have no access to the values to be
        /// returned by the function.
        HookRet = raw::LUA_HOOKRET,
        /// The line hook is called when the interpreter is about to start the execution of a new
        /// line of code, or when it jumps back in the code (even to the same line).
        /// (This event only happens while Lua is executing a Lua function.)
        HookLine = raw::LUA_HOOKLINE,
        /// The count hook is called after the interpreter executes every `count` instructions.
        /// (This event only happens while Lua is executing a Lua function.)
        HookCount = raw::LUA_HOOKCOUNT,
        /// The tailret event is used when a HookRet hook is called while simulating a return from
        /// a function that did a tail call; in this case, it is useless to call getinfo().
        HookTailRet = raw::LUA_HOOKTAILRET
    }

    /// Converts a c_int event code to a DebugEvent.
    pub fn from_event(event: c_int) -> Option<DebugEvent> {
        match event {
            raw::LUA_HOOKCALL => Some(HookCall),
            raw::LUA_HOOKRET => Some(HookRet),
            raw::LUA_HOOKLINE => Some(HookLine),
            raw::LUA_HOOKCOUNT => Some(HookCount),
            raw::LUA_HOOKTAILRET => Some(HookTailRet),
            _ => None
        }
    }

    /// Event mask for HookCall
    pub static MaskCall: i32 = raw::LUA_MASKCALL as i32;
    /// Event mask for HookRet
    pub static MaskRet: i32 = raw::LUA_MASKRET as i32;
    /// Event mask for HookLine
    pub static MaskLine: i32 = raw::LUA_MASKLINE as i32;
    /// Event mask for HookCount
    pub static MaskCount: i32 = raw::LUA_MASKCOUNT as i32;
}

/// Type for functions to be called by the debugger in specific events
pub type Hook = raw::lua_Hook;

/// A structure used to carry different peices of information about an active function.
/// getstack() fills only the private part of this structure, for later use. To fill the other
/// fields of lua_Debug with useful information, call getinfo().
pub type Debug = raw::lua_Debug;

impl raw::lua_Debug {
    /// Returns a newly-zeroed instance of Debug
    pub fn new() -> Debug {
        #[inline];
        std::default::Default::default()
    }
}

impl State {
    /// Gets information about the interpreter runtime stack.
    ///
    /// This function returns a Debug structure with an identification of the activation
    /// record of the function executing at a given level. Level 0 is the current running
    /// function, whereas level n+1 is the function that has called level n. When there are no
    /// errors, getstack() returns Some(Debug); when called with a level greater than the stack
    /// depth, it returns None.
    pub fn getstack(&mut self, level: i32) -> Option<Debug> {
        #[inline];
        let mut ar: Debug = std::default::Default::default();
        if unsafe { raw::lua_getstack(self.L, level as c_int, &mut ar) != 0 } {
            Some(ar)
        } else {
            None
        }
    }

    /// Returns information about a specific function or function invocation.
    ///
    /// To get information about a function invocation, the parameter `ar` must ve a valid
    /// activation record that was returned by a previous call to getstack() or given as argument
    /// to a hook.
    ///
    /// To get information about a function you push it onto the stack and start the `what` string
    /// with the character '>'. (In that case, getinfo() pops the function in the top of the
    /// stack.) For instance, to know in which line a function `f` was defined, you can write
    /// the following code:
    ///
    ///   let ar = Debug::new();
    ///   L.getfield(GLOBALSINDEX, "f"); // get global 'f'
    ///   L.getinfo(">S", &mut ar);
    ///   println!("{}", ar.linedefined);
    ///
    /// Each character in the string `what` selects some fields of the structure `ar` to be
    /// filled or a value to be pushed on the stack:
    ///
    /// * 'n': fills in the fields `name` and `namewhat`
    /// * 'S': fills in the fields `source`, `short_src`, `linedefined`, `lastlinedefined`, and
    ///        `what`
    /// * 'l': fills in the field `currentline`
    /// * 'u': fills in the field `nups`
    /// * 'f': pushes onto the stack the function that is running at the given level
    /// * 'L': pushes onto the stack a table whose indices are the numbers of the lines that are
    ///        valid on the function. (A valid line is a line with some associated code, that is,
    ///        a line where you can put a break point. Non-valid lines include empty lines and
    ///        comments.)
    ///
    /// This function returns `false` on error (for instance, an invalid option in `what`).
    ///
    /// Raises the `c_str::null_byte` condition if `what` has interior NULs.
    pub fn getinfo(&mut self, what: &str, ar: &mut Debug) -> bool {
        #[inline];
        if what.starts_with(">") {
            luaassert!(self, self.gettop() >= 1 && self.isfunction(-1),
                       "getinfo: top stack value is not a function");
        }
        if what.find(&['f', 'L']).is_some() {
            self.checkstack_(1);
        }
        unsafe { self.getinfo_unchecked(what, ar) }
    }

    /// Unchecked variant of getinfo()
    pub unsafe fn getinfo_unchecked(&mut self, what: &str, ar: &mut Debug) -> bool {
        #[inline];
        what.with_c_str(|w| raw::lua_getinfo(self.L, w, ar)) != 0
    }

    /// Gets information about a local variable of a given activation record. The parameter `ar`
    /// must be a valid activation record that was filled by a previous call to getstack() or
    /// given as an argument to a hook. The index `n` selects which local variable to inspect
    /// (1 is the first parameter or active local variable, and so on, until the last active
    /// local variable). getlocal() pushes the variable's value onto the stack and returns its
    /// name.
    ///
    /// Variable names starting with '(' represent internal variables (loop control variables,
    /// temporaries, and C function locals).
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8 decoding vs invalid
    /// indices.
    pub fn getlocal<'a>(&'a mut self, ar: &mut Debug, n: i32) -> Option<&'a [u8]> {
        #[inline];
        self.checkstack_(1);
        unsafe { self.getlocal_unchecked(ar, n) }
    }

    /// Unchecked variant of getlocal()
    pub unsafe fn getlocal_unchecked<'a>(&'a mut self, ar: &mut Debug, n: i32) -> Option<&'a [u8]> {
        #[inline];
        let res = raw::lua_getlocal(self.L, ar, n as c_int);
        c_str_to_bytes(res)
    }

    /// Sets the value of a local variable of a given activation record. Parameters `ar` and `n`
    /// are as in getlocal(). setlocal() assigns the value at the top of the stack to the variable
    /// and returns its name. It also pops the value from the stack.
    ///
    /// Returns None (and pops nothing) when the index is greater than the number of active local
    /// variables.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8 decoding vs invalid
    /// indices.
    pub fn setlocal<'a>(&'a mut self, ar: &mut Debug, n: i32) -> Option<&'a [u8]> {
        #[inline];
        luaassert!(self, self.gettop() >= 1, "setlocal: stack underflow");
        unsafe { self.setlocal_unchecked(ar, n) }
    }

    /// Unchecked variant of setlocal()
    pub unsafe fn setlocal_unchecked<'a>(&'a mut self, ar: &mut Debug, n: i32) -> Option<&'a [u8]> {
        #[inline];
        let res = raw::lua_setlocal(self.L, ar, n as c_int);
        c_str_to_bytes(res)
    }

    /// Gets information about a closure's upvalue. (For Lua functions, upvalues are the external
    /// local variables that the function uses, and that are consequently included in its closure.)
    /// getupvalue() gets the index `n` of an upvalue, pushes the upvalue's value onto the stack,
    /// and returns its name. `funcindex` points to the closure in the stack. (Upvalues have no
    /// particular order, as they are active through the whole function. So, they are numbered in
    /// an arbitrary order.)
    ///
    /// Returns None (and pushes nothing) when the index is greater than the number of upvalues.
    /// For C functions, this function uses the empty string "" as a name for all upvalues.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8 decoding vs invalid
    /// indices.
    pub fn getupvalue<'a>(&'a mut self, funcindex: i32, n: i32) -> Option<&'a [u8]> {
        #[inline];
        self.check_acceptable(funcindex);
        self.checkstack_(1);
        unsafe { self.getupvalue_unchecked(funcindex, n) }
    }

    /// Unchecked variant of getupvalue()
    pub unsafe fn getupvalue_unchecked<'a>(&'a mut self, funcindex: i32, n: i32)
                                          -> Option<&'a [u8]> {
        #[inline];
        let res = raw::lua_getupvalue(self.L, funcindex as c_int, n as c_int);
        c_str_to_bytes(res)
    }

    /// Sets the value of a closure's upvalue. It assigns the value at the top of the stack to the
    /// upvalue and returns its name. It also pops the value from the stack. Parameters
    /// `funcindex` and `n` are as in getupvalue().
    ///
    /// Returns None (and pops nothing) when the index is greater than the number of upvalues.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8 decoding vs invalid
    /// indices.
    pub fn setupvalue<'a>(&'a mut self, funcindex: i32, n: i32) -> Option<&'a [u8]> {
        #[inline];
        self.check_acceptable(funcindex);
        self.checkstack_(1);
        unsafe { self.setupvalue_unchecked(funcindex, n) }
    }

    /// Unchecked variant of setupvalue()
    pub unsafe fn setupvalue_unchecked<'a>(&'a mut self, funcindex: i32, n: i32)
                                          -> Option<&'a [u8]> {
        #[inline];
        let res = raw::lua_setupvalue(self.L, funcindex as c_int, n as c_int);
        c_str_to_bytes(res)
    }

    /// Sets the debugging hook function.
    ///
    /// Argument `f` is the hook function. `mask` specifies on which events the hook will be called:
    /// it is formed by a bitwise OR of the Mask* constants in DebugEvent. The `count` argument is
    /// only meaningful when the mask includes DebugEvent::MaskCount.
    ///
    /// A hook is disabled by setting `mask` to zero.
    pub fn sethook(&mut self, f: Hook, mask: i32, count: i32) {
        #[inline];
        unsafe { raw::lua_sethook(self.L, f, mask as c_int, count as c_int); }
    }

    /// Returns the current hook function
    pub fn gethook(&mut self) -> Hook {
        #[inline];
        unsafe { raw::lua_gethook(self.L) }
    }

    /// Returns the current hook mask
    pub fn gethookmask(&mut self) -> i32 {
        #[inline];
        unsafe { raw::lua_gethookmask(self.L) as i32 }
    }

    /// Returns the current hook count
    pub fn gethookcount(&mut self) -> i32 {
        #[inline];
        unsafe { raw::lua_gethookcount(self.L) as i32 }
    }
}

unsafe fn c_str_to_bytes<'a>(cstr: *libc::c_char) -> Option<&'a [u8]> {
    #[inline];
    if cstr.is_null() {
        None
    } else {
        let cstr = CString::new(cstr, false);
        let bytes = cstr.as_bytes();
        Some(cast::transmute::<&[u8],&'a [u8]>(bytes))
    }
}
