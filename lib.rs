//! Lua 5.1 bindings for Rust

#![crate_id="github.com/kballard/rust-lua#lua:0.1"]

#![comment = "Lua 5.1 bindings for Rust"]
#![license = "MIT"]
#![crate_type = "rlib"]

#![feature(macro_rules, default_type_params)]

#![warn(missing_doc)]
#![allow(uppercase_variables,non_snake_case_functions)]

extern crate libc;

use libc::c_int;
use std::{mem, path, ptr, str, slice};
use std::c_str::CString;
use std::default::Default;

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
    #![inline]
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

mod macro;

#[cfg(test)]
mod tests;

macro_rules! luaassert{
    ($state:expr, $cond:expr, $msg:expr) => {
        if $state.stack_check.is_safe() && !$cond {
            $state.errorstr($msg.as_slice());
        }
    };
    ($state:expr, $cond:expr, $($arg:expr),+) => {
        if self.stack_check.is_safe() && !$cond {
            let msg = format!($($arg),+);
            $state.errorstr(msg.as_slice());
        }
    }
}

/// Lua value type
pub type Type = Type::Type;
pub mod Type {
    //! Lua value type mod
    use raw;
    use libc;
    use std::{ptr, str};

    /// Lua value types
    #[deriving(Clone,PartialEq,Eq,Show)]
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
    use std::fmt;
    /// State.load() errors
    pub enum LoadError {
        /// Syntax error during pre-compilation
        ErrSyntax = raw::LUA_ERRSYNTAX,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM
    }

    impl fmt::Show for LoadError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ErrSyntax => f.pad("syntax error"),
                ErrMem => f.pad("memory allocation error")
            }
        }
    }
}

/// State.loadfile() errors
pub type LoadFileError = LoadFileError::LoadFileError;
pub mod LoadFileError {
    //! State.loadfile() error mod
    use aux;
    use raw;
    use std::fmt;
    /// State.loadfile() errors
    pub enum LoadFileError {
        /// Syntax error during pre-compilation
        ErrSyntax = raw::LUA_ERRSYNTAX,
        /// Memory allocation error
        ErrMem = raw::LUA_ERRMEM,
        /// Cannot read/open the file
        ErrFile = aux::raw::LUA_ERRFILE
    }

    impl fmt::Show for LoadFileError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ErrSyntax => f.pad("syntax error"),
                ErrMem => f.pad("memory allocation error"),
                ErrFile => f.pad("file read/open error")
            }
        }
    }
}

/// State.pcall() errors
pub type PCallError = PCallError::PCallError;
pub mod PCallError {
    //! State.pcall() error mod
    use raw;
    use libc::c_int;
    use std::fmt;
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

    impl fmt::Show for PCallError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ErrRun => f.pad("runtime error"),
                ErrMem => f.pad("memory allocation error"),
                ErrErr => f.pad("error handler func error")
            }
        }
    }
}

/// The Lua state.
/// Every Lua thread is represented by a separate State.
///
/// When executing functions on the State that take acceptable indexes, these
/// indexes are checked to ensure they are within the stack space defined by
/// the last call to State.checkstack(). If they are not acceptable, the
/// function fails without calling lua_checkstack().  Negative indices are
/// checked against the current top of the stack instead of the stack space.
///
/// Unless otherwise noted, all safe functions that take indexes will fail if
/// the index is not acceptable.
///
/// There are two variant state types, ExternState and RawState, that assume
/// different behavior for thrown errors. ExternState is meant for functions
/// that are executing in a protected scope (see pcall()), and RawState is
/// meant for omitting safety in favor of performance.
///
/// Note that it is completely unsafe to pass a reference to State to a
/// function that is executing in a protected scope. Use ExternState for that.
#[unsafe_no_drop_flag]
pub struct State<B = SafeBehavior> {
    L: *mut raw::lua_State,
    stackspace: i32,
    stack_check: B,
}

#[unsafe_destructor]
impl<T> Drop for State<T> {
    fn drop(&mut self) {
        if self.L.is_not_null() {
            unsafe {
                raw::lua_close(self.L);
            }
            self.L = ptr::mut_null();
        }
    }
}

/// Describes error checking behavior.
trait CheckingBehavior {
    /// Indicates whether the indexes should be checked.
    fn is_safe(&self) -> bool;
}

/// Unless otherwise noted, all safe functions that take indexes will fail if
/// the index is not acceptable.
#[deriving(Default)]
pub struct SafeBehavior;

/// Every error-throwing function is assumed to be using longjmp instead of
/// task failure.
///
/// See State for more information.
/// RawState is a Lua State that represents raw, unchecked access. All
/// functions eschew safety in favor of speed. Like ExternState, all
/// error-throwing functions are assumed to be using longjmp.
#[deriving(Default)]
pub struct UnsafeBehavior<'a>;

impl CheckingBehavior for SafeBehavior {
    fn is_safe(&self) -> bool { #![inline] true }
}

impl<'a> CheckingBehavior for UnsafeBehavior<'a> {
    fn is_safe(&self) -> bool { #![inline] false }
}

pub type RawState<'a> = State<UnsafeBehavior<'a>>;

// State construction
impl State {
    /// Returns a new State, or fails if memory cannot be allocated for the state
    pub fn new() -> State {
        #![inline]
        State::new_opt().unwrap()
    }

    /// Returns a new State, or None if memory cannot be allocated for the state
    pub fn new_opt() -> Option<State> {
        return unsafe {
            let L = raw::lua_newstate(alloc, ptr::mut_null());
            if L.is_not_null() {
                raw::lua_atpanic(L, panic);
                Some(State{ L: L, stackspace: MINSTACK, stack_check: SafeBehavior })
            } else {
                None
            }
        };

        extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
                            nsize: libc::size_t) -> *mut libc::c_void {
            unsafe {
                if nsize == 0 {
                    libc::free(ptr as *mut libc::c_void);
                    ptr::mut_null()
                } else {
                    libc::realloc(ptr, nsize)
                }
            }
        }
        extern "C" fn panic(L: *mut raw::lua_State) -> c_int {
            unsafe {
                let s = State::<SafeBehavior>::from_lua_State(L).describe_(-1, false);
                fail!("unprotected error in call to Lua API ({})", s);
            }
        }
    }
}

impl<T: CheckingBehavior + Default> State<T> {
    /// Wraps a *raw::lua_State in a State.
    pub unsafe fn from_lua_State(L: *mut raw::lua_State) -> State<T> {
        #![inline]
        State {
            L: L,
            stackspace: MINSTACK,
            stack_check: Default::default()
        }
    }
}

// State conversion
impl<T: CheckingBehavior> State<T> {
    // Unsafe Deref doesn't exist.

    /// Passes the state to a closure
    pub unsafe fn with_unsafe(&mut self, f: |&mut RawState|) {
        f(&mut *(self as *mut State<T> as *mut RawState))
    }

    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        #![inline]
        self.L
    }
}

impl<T: CheckingBehavior> State<T> {
    /// Creates a new thread, pushes it on the stack, and returns a `State`
    /// that represents this new thread. The new state returned by this
    /// function shares with the original state all global objects (such as
    /// tables), but has an independent execution stack.
    ///
    /// This new state does not get explicitly closed. Threads are subject to
    /// garbage collection, like any Lua object.
    pub fn newthread(&mut self) -> State {
        #![inline(always)]
        unsafe {
            mem::transmute(State::<SafeBehavior>::from_lua_State(raw::lua_newthread(self.L)))
        }
    }

    /// Sets a new panic function and returns the old one.
    ///
    /// The panic function can access the error message at the top of the stack.
    ///
    /// The default panic function installed by this library calls fail!() with
    /// the error message. Your panic function should either call through to
    /// the default one, or should fail!() itself. Otherwise, the application
    /// will be terminated.
    pub fn atpanic(&mut self, panicf: CFunction) -> CFunction {
        unsafe {
            raw::lua_atpanic(self.L, panicf)
        }
    }

    pub fn describe(&mut self, idx: i32) -> String {
        #![allow(missing_doc)]
        self.check_acceptable(idx);
        self.checkstack_(1);
        self.describe_(idx, true)
    }

    pub fn describe_(&mut self, idx: i32, usestack: bool) -> String {
        #![allow(missing_doc)]
        self.check_acceptable(idx);
        if usestack { self.checkstack_(1); }

        match self.type_(idx) {
            None => "".to_string(),
            Some(typ) => match typ {
                Type::Nil => "nil".to_string(),
                Type::Boolean => if self.toboolean(idx) { "true".to_string() }
                                 else { "false".to_string() },
                Type::Number => {
                    // Let Lua create the string instead of us
                    if usestack { self.pushvalue(idx); } // copy the value
                    let s = self.tostring(-1).map(|s| s.to_string());
                    if usestack { self.pop(1); } // remove the copied value
                    s.unwrap_or_default() // default will be ~""
                }
                Type::String => {
                    self.tostring(idx).unwrap_or("<invalid utf8>").to_string()
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

    /// Returns the index of the top element of the stack.
    /// Indexes start at 1. 0 means the stack is empty.
    pub fn gettop(&mut self) -> i32 {
        unsafe { raw::lua_gettop(self.L) as i32 }
    }

    /// Sets the stack top to the given acceptable index, or 0.
    /// If the new top is larger than the old one, new elements are filled with
    /// nil.
    /// If the index is 0, all stack elements are removed.
    pub fn settop(&mut self, idx: i32) {
        if idx != 0 { self.check_acceptable(idx); }
        unsafe { raw::lua_settop(self.L, idx as c_int) }
    }

    /// Pushes a copy of the element at the given valid index onto the stack.
    pub fn pushvalue(&mut self, idx: i32) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        unsafe { raw::lua_pushvalue(self.L, idx as c_int) }
    }

    fn check_acceptable(&mut self, idx: i32) {
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
        if !self.stack_check.is_safe() {
            return;
        }

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

    /// Removes the element at the given valid index, shifting other elements
    /// as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn remove(&mut self, idx: i32) {
        self.check_valid(idx, false);
        unsafe { raw::lua_remove(self.L, idx as c_int) }
    }

    /// Moves the top element into the given valid index, shifting existing
    /// elements as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn insert(&mut self, idx: i32) {
        self.check_valid(idx, false);
        unsafe { raw::lua_insert(self.L, idx as c_int) }
    }

    /// Moves the top element into the given valid index and replaces the
    /// existing value, without shifting any other elements.
    pub fn replace(&mut self, idx: i32) {
        self.check_valid(idx, true);
        unsafe { raw::lua_replace(self.L, idx as c_int) }
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Returns false if it cannot grow the stack as requested.
    pub fn checkstack(&mut self, extra: i32) -> bool {
        let top = self.gettop();
        if top + extra > self.stackspace {
            unsafe {
                if raw::lua_checkstack(self.L, extra as c_int) != 0 {
                    self.stackspace = top + extra;
                    true
                } else {
                    false
                }
            }
        } else {
            true
        }
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Throws an error if it cannot grow the stack.
    pub fn checkstack_(&mut self, extra: i32) {
        luaassert!(self, self.checkstack(extra), "checkstack: cannot grow stack")
    }

    /// Despite being unsafe, it still checks the validity of `n`.
    pub fn xmove(&mut self, to: &mut State<T>, n: i32) {
        luaassert!(self, self.gettop() >= n, "xmove: stack underflow");
        to.checkstack_(1);
        unsafe { raw::lua_xmove(self.L, to.L, n as c_int) }
    }

    /// Returns `true` if the value at the given acceptable index is a number,
    /// or a string convertible to a number.
    pub fn isnumber(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isnumber(self.L, idx as c_int) != 0 }
    }

    /// Returns `true` if the value at the given acceptable index is a string
    /// or a number (which is always convertible to a string).
    pub fn isstring(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isstring(self.L, idx as c_int) != 0 }
    }

    /// Returns `true` if the value at the given acceptable index is a C
    /// function.
    pub fn iscfunction(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_iscfunction(self.L, idx as c_int) != 0 }
    }

    /// Returns `true` if the value at the given acceptable index is a userdata
    /// (either full or light).
    pub fn isuserdata(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isuserdata(self.L, idx as c_int) != 0 }
    }

    /// Returns the type of the value at the given acceptable index.  If the
    /// given index is non-valid, returns None.
    pub fn type_(&mut self, idx: i32) -> Option<Type> {
        self.check_acceptable(idx);
        unsafe {
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
    }

    /// Returns the name of the type of the value at the given acceptable
    /// index.
    pub fn typename(&mut self, idx: i32) -> &'static str {
        self.check_acceptable(idx);
        unsafe {
            let s = aux::raw::luaL_typename(self.L, idx as c_int);
            str::raw::c_str_to_static_slice(s)
        }
    }

    /// Returns `true` if the two values in acceptable indices `index1` and
    /// `index2` are equal, following the semantics of the Lua == operator.
    /// Returns `false` if any indices are non-valid.
    pub fn equal(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { raw::lua_equal(self.L, index1 as c_int, index2 as c_int) != 0 }
    }

    /// Returns `true` if the two values in acceptable indices `index1` and
    /// `index2` are primitively equal (that is, without calling any
    /// metamethods). Returns `false` if any indices are non-valid.
    pub fn rawequal(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { raw::lua_rawequal(self.L, index1 as c_int, index2 as c_int) != 0 }
    }

    /// Returns `true` if the value at acceptable index `index1` is smaller
    /// than the value at acceptable index `index2`, following the semantics of
    /// the Lua < operator. Returns `false` if any indices are non-valid.
    pub fn lessthan(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        unsafe { raw::lua_lessthan(self.L, index1 as c_int, index2 as c_int) != 0 }
    }

    /// Converts the Lua value at the given acceptable index to a f64. The Lua
    /// value must be a number or a string convertible to a number; otherwise,
    /// tonumber returns 0.
    pub fn tonumber(&mut self, idx: i32) -> f64 {
        self.check_acceptable(idx);
        unsafe { raw::lua_tonumber(self.L, idx as c_int) as f64 }
    }

    /// Converts the Lua value at the given acceptable index to an int. The Lua
    /// value must be a number or a string convertiable to a number; otherwise,
    /// toint returns 0.
    pub fn tointeger(&mut self, idx: i32) -> int {
        self.check_acceptable(idx);
        unsafe { raw::lua_tointeger(self.L, idx as c_int) as int }
    }

    /// Converts the value at the given acceptable index to a bool.
    /// Returns false when called with a non-valid index.
    pub fn toboolean(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_toboolean(self.L, idx as c_int) != 0 }
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// ExternState, but its lifetime is actually that of the value on the
    /// stack.
    /// Note: if the value is a number, this method changes the value in the
    /// stack to a string.  This may confuse lua_next if this is called during
    /// table traversal.
    pub fn tostring(&mut self, idx: i32) -> Option<&'static str> {
        self.check_acceptable(idx);
        self.tobytes(idx).and_then(|v| str::from_utf8(v))
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// ExternState, but its lifetime is actually that of hte value on the
    /// stack.
    /// Converts the value at the given acceptable index into a lua string, and
    /// returns it as a byte vector.
    /// Returns None if the value is not a number or a string.
    /// See tostring() for caveats.
    pub fn tobytes(&mut self, idx: i32) -> Option<&'static [u8]> {
        self.check_acceptable(idx);
        let mut sz: libc::size_t = 0;
        unsafe {
            let s = raw::lua_tolstring(self.L, idx, &mut sz);
            if s.is_null() {
                None
            } else {
                slice::raw::buf_as_slice(s as *u8, sz as uint, |b| {
                    Some(mem::transmute::<&[u8], &'static [u8]>(b))
                })
            }
        }
    }

    /// Returns the "length" of the value at the given acceptable index.
    pub fn objlen(&mut self, idx: i32) -> uint {
        self.check_acceptable(idx);
        unsafe { raw::lua_objlen(self.L, idx as c_int) as uint }
    }

    /// Converts a value at the given acceptable index to a C function. The
    /// value must be a C function; otherwise, returns None.
    pub fn tocfunction(&mut self, idx: i32) -> Option<CFunction> {
        self.check_acceptable(idx);
        unsafe { raw::lua_tocfunction(self.L, idx as c_int) }
    }

    /// If the value at the given acceptable index is a full userdata, returns
    /// its block address. If the value is a light userdata, returns its
    /// pointer. Otherwise, returns ptr::null().
    pub fn touserdata(&mut self, idx: i32) -> *mut libc::c_void {
        self.check_acceptable(idx);
        unsafe { raw::lua_touserdata(self.L, idx as c_int) }
    }

    /// Note: the State return value does not make any assumptions about the
    /// available stack space. .checkstack() must be called in order to
    /// consider any non-valid index as acceptable.
    pub fn tothread(&mut self, idx: i32) -> Option<RawState> {
        self.check_acceptable(idx);
        let s = unsafe { raw::lua_tothread(self.L, idx as c_int) };
        if s.is_null() {
            None
        } else {
            Some(State { L: s, stackspace: 0, stack_check: UnsafeBehavior })
        }
    }

    /// Converts the value at the given acceptable index to a pointer. The
    /// value can be a userdata, a table, a thread, or a function.
    pub fn topointer(&mut self, idx: i32) -> *libc::c_void {
        self.check_acceptable(idx);
        unsafe { raw::lua_topointer(self.L, idx as c_int) }
    }

    /// Pushes a nil value onto the stack.
    pub fn pushnil(&mut self) {
        self.checkstack_(1);
        unsafe { raw::lua_pushnil(self.L) }
    }

    /// Pushes a number with value `n` onto the stack
    pub fn pushnumber(&mut self, n: f64) {
        self.checkstack_(1);
        unsafe { raw::lua_pushnumber(self.L, n as raw::lua_Number) }
    }

    /// Pushes a number with value `n` onto the stack.
    pub fn pushinteger(&mut self, n: int) {
        self.checkstack_(1);
        unsafe { raw::lua_pushinteger(self.L, n as raw::lua_Integer) }
    }

    /// Pushes a string onto the stack
    pub fn pushstring(&mut self, s: &str) {
        self.checkstack_(1);
        unsafe { raw::lua_pushlstring(self.L, s.as_ptr() as *libc::c_char, s.len() as libc::size_t) }
    }

    /// Pushes a byte vector onto the stack as a lua string
    pub fn pushbytes(&mut self, bytes: &[u8]) {
        self.checkstack_(1);
        unsafe {
            raw::lua_pushlstring(self.L, bytes.as_ptr() as *libc::c_char,
                                         bytes.len() as libc::size_t)
        }
    }

    /// `n` must be in the range [0, 255]. Anything outside this range will
    /// throw an error.
    pub fn pushcclosure(&mut self, f: CFunction, n: i32) {
        if n == 0 {
            self.checkstack_(1);
        } else {
            luaassert!(self, n >= 0 && n <= 255, "pushcclosure: invalid argument n");
        }
        unsafe { raw::lua_pushcclosure(self.L, f, n as c_int) }
    }

    /// Pushes a boolean value onto the stack.
    pub fn pushboolean(&mut self, b: bool) {
        self.checkstack_(1);
        unsafe { raw::lua_pushboolean(self.L, b as c_int) }
    }

    /// Pushes a light userdata onto the stack.
    pub fn pushlightuserdata(&mut self, p: *mut libc::c_void) {
        self.checkstack_(1);
        unsafe { raw::lua_pushlightuserdata(self.L, p) }
    }

    /// Pushes the thread represented by `self` onto the stack. Returns `true`
    /// if this thread is the main thread of the state.
    pub fn pushthread(&mut self) -> bool {
        self.checkstack_(1);
        unsafe { raw::lua_pushthread(self.L) != 0 }
    }

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index and k is the value at the top of the stack. The key is
    /// popped from the stack.
    pub fn gettable(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "gettable: stack underflow");
        unsafe { raw::lua_gettable(self.L, idx as c_int) }
    }

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index. Fails the task if `k` has any interior NULs.
    pub fn getfield(&mut self, idx: i32, k: &str) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        unsafe { k.with_c_str(|s| raw::lua_getfield(self.L, idx as c_int, s)) }
    }

    /// Similar to gettable(), but does a raw access
    pub fn rawget(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "rawget: stack underflow");
        unsafe { raw::lua_rawget(self.L, idx as c_int) }
    }

    /// Pushes onto the stack the value t[n], where t is the value at the given
    /// valid index. The access is raw; that is, it does not invoke
    /// metamethods.
    pub fn rawgeti(&mut self, idx: i32, n: i32) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        unsafe { raw::lua_rawgeti(self.L, idx as c_int, n as c_int) }
    }

    /// Creates a new empty table and pushes it into the stack. The new table
    /// has space pre-allocated for `narr` array elements and `nrec` non-array
    /// elements.
    pub fn createtable(&mut self, narr: i32, nrec: i32) {
        self.checkstack_(1);
        unsafe { raw::lua_createtable(self.L, narr as c_int, nrec as c_int) }
    }

    /// This method allocates a new block of memory with the given size, pushes
    /// onto the stack a new full userdata with the block address, and returns
    /// this address.
    pub fn newuserdata(&mut self, size: uint) -> *mut libc::c_void {
        self.checkstack_(1);
        unsafe { raw::lua_newuserdata(self.L, size as libc::size_t) }
    }

    /// Pushes onto the stack the metatable of the value at the given
    /// acceptable index. If the index is not valid, or the value does not have
    /// a metatable, the function returns `false` and pushes nothing onto the
    /// stack.
    pub fn getmetatable(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.checkstack_(1);
        unsafe { raw::lua_getmetatable(self.L, idx as c_int) != 0 }
    }

    /// Pushes onto the stack the environment table of the value at the given
    /// index.
    pub fn getfenv(&mut self, idx: i32) {
        self.check_acceptable(idx);
        self.checkstack_(1);
        unsafe { raw::lua_getfenv(self.L, idx as c_int) }
    }

    /// This function pops both the key and the value from the stack.
    pub fn settable(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 2, "settable: stack underflow");
        unsafe { raw::lua_settable(self.L, idx as c_int) }
    }

    /// Fails the task if `k` contains interior NULs.
    pub fn setfield(&mut self, idx: i32, k: &str) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 1, "setfield: stack underflow");
        unsafe { k.with_c_str(|kp| raw::lua_setfield(self.L, idx as c_int, kp)) }
    }

    /// Similar to settable(), but does a raw assignment.
    pub fn rawset(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 2, "rawset: stack underflow");
        unsafe { raw::lua_rawset(self.L, idx as c_int) }
    }

    /// This function pops the value from the stack. The assignment is raw;
    /// that is, it does not invoke metamethods.
    pub fn rawseti(&mut self, idx: i32, n: i32) {
        self.check_valid(idx, true);
        unsafe { raw::lua_rawseti(self.L, idx as c_int, n as c_int) }
    }

    /// Pops a table from the stack and sets it as the new metatable for the
    /// value at the given acceptable index.
    pub fn setmetatable(&mut self, idx: i32) {
        self.check_acceptable(idx);
        luaassert!(self, self.istable(-1), "setmetatable: top stack value must be a table");
        // ignore return value of lua_setmetatable(), it appears to always be 1
        unsafe { raw::lua_setmetatable(self.L, idx as c_int); }
    }

    /// Pops a table from the stack and sets it as the new environment for the
    /// value at the given index. If the value at the given index is neither a
    /// function nor a thread nor a userdata, setfenv() returns `false`.
    /// Otherwise, returns `true`.
    pub fn setfenv(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        luaassert!(self, self.istable(-1), "setfenv: top stack value must be a table");
        unsafe { raw::lua_setfenv(self.L, idx as c_int) != 0 }
    }

    /// Calls a function.
    /// The function must be pushed first, followed by its arguments. `nargs`
    /// is the number of arguments. The function and its arguments are popped
    /// automatically.
    /// The function results are adjusted to `nresults`, unless `nresults` is
    /// `MULTRET`, in which case all function results are pushed.
    pub fn call(&mut self, nargs: i32, nresults: i32) {
        luaassert!(self, nargs >= 0, "call: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "call: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "call: stack underflow");
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        unsafe { raw::lua_call(self.L, nargs as c_int, nresults as c_int) }
    }

    /// If `errfunc` is 0, then the error message returned on the stack is
    /// exactly the original error message. Otherwise, `errfunc` is the stack
    /// index of an error handler function. It must not be a pseudo-index.
    pub fn pcall(&mut self, nargs: i32, nresults: i32, errfunc: i32)
                       -> Result<(),PCallError> {
        luaassert!(self, nargs >= 0, "pcall: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "pcall: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "pcall: stack underflow");
        if errfunc != 0 {
            self.check_valid(errfunc, false)
        }
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        unsafe {
            match raw::lua_pcall(self.L, nargs as c_int, nresults as c_int, errfunc as c_int) {
                0 => Ok(()),
                i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                    self.errorstr("pcall: unexpected error from lua_pcall")
                }))
            }
        }
    }

    /// Fails the task if `chunkname` contains interior NULs.
    pub fn load(&mut self, reader: Reader, data: *mut libc::c_void, chunkname: &str)
                      -> Result<(),LoadError> {
        self.checkstack_(1);
        unsafe {
            match chunkname.with_c_str(|name| raw::lua_load(self.L, reader, data, name)) {
                0 => Ok(()),
                raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
                raw::LUA_ERRMEM => Err(LoadError::ErrMem),
                _ => self.errorstr("load: unexpected error from lua_load")
            }
        }
    }

    /// This function does not pop the Lua function from the stack.
    pub fn dump(&mut self, writer: Writer, data: *mut libc::c_void) -> Result<(),i32> {
        luaassert!(self, self.gettop() >= 1, "dump: stack underflow");
        unsafe {
            match raw::lua_dump(self.L, writer, data) {
                0 => Ok(()),
                i => Err(i)
            }
        }
    }

    /// When a C function calls yield_() in that way, the running coroutine
    /// suspends its execution, and the call to resume() that started this
    /// coroutine returns. The parameter `nresults` is the number of values
    /// from the stack that are passed as the results to resume().
    pub fn yield_(&mut self, nresults: i32) -> c_int {
        luaassert!(self, self.gettop() >= nresults, "yield: stack underflow");
        unsafe { raw::lua_yield(self.L, nresults as c_int) }
    }

    /// To start a coroutine, you first create a new thread (see thread());
    /// then you push onto its stack the main function plus any arguments; then
    /// you call resume(), with `narg` being the number of arguments. This call
    /// returns when the coroutine suspends or finishes its execution. When it
    /// returns, the stack contains all values passed to yield_(), or all
    /// values returned by the body function. resume() returns Ok(false) if the
    /// coroutine yields, Ok(true) if the coroutine finishes its execution
    /// without errors, or Err(PCallError) in case of errors. In case of
    /// errors, the stack is not unwound, so you can use the debug API over it.
    /// The error message is on top of the stack. To restart a coroutine, you
    /// put on its stack only the values to be passed as results from yield_(),
    /// and then call resume().
    pub fn resume(&mut self, narg: i32) -> Result<bool,PCallError> {
        luaassert!(self, self.gettop() > narg, "resume: stack underflow");
        unsafe {
            match raw::lua_resume(self.L, narg as c_int) {
                raw::LUA_YIELD => Ok(false),
                0 => Ok(true),
                i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                    self.errorstr("resume: unexpected error from lua_resume")
                }))
            }
        }
    }

    /// The status can be Ok(true) for a normal thread, Ok(false) if the thread
    /// is suspended, or Err(PCallError) if the thread finished its execution
    /// with an error.
    pub fn status(&mut self) -> Result<bool,PCallError> {
        unsafe {
            match raw::lua_status(self.L) {
                raw::LUA_YIELD => Ok(false),
                0 => Ok(true),
                i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                    self.errorstr("status: unexpected error from lua_status")
                }))
            }
        }
    }

    /// This method performs several tasks, according to the value of the
    /// parameter `what`. See the `GC` enum for documentation on the various
    /// options.
    pub fn gc(&mut self, what: GC, data: i32) -> i32 {
        unsafe { raw::lua_gc(self.L, what as c_int, data as c_int) as i32 }
    }

    /// Raises an error (using the value at the top of the stack)
    pub fn error(&mut self) -> ! {
        luaassert!(self, self.gettop() > 0, "error: stack underflow");
        unsafe { raw::lua_error(self.L); }
        unreachable!()
    }

    /// While traversing a table, do not call tostring() or tobytes() directly
    /// on a key, unless you know that the key is actually a string. Recall
    /// that tostring() changes the value at the given index; this confuses the
    /// next call to next().
    pub fn next(&mut self, idx: i32) -> bool {
        self.check_valid(idx, true);
        unsafe { raw::lua_next(self.L, idx as c_int) != 0 }
    }

    /// Concatenates the `n` values at the top of the stack, pops them, and
    /// leaves the result at the top.
    /// Errors if n is negative or larger than the stack top.
    pub fn concat(&mut self, n: i32) {
        luaassert!(self, n >= 0, "concat: invalid argument n");
        luaassert!(self, n <= self.gettop(), "concat: stack underflow");
        if n == 0 { self.checkstack_(1) }
        unsafe { raw::lua_concat(self.L, n as c_int) }
    }

    /// Note: State::new() always provides NULL as the opaque pointer. It also
    /// provides a default alloc function that behaves identically to the one
    /// used by luaL_newstate().
    pub fn getallocf(&mut self, ud: *mut *mut libc::c_void) -> Alloc {
        unsafe { raw::lua_getallocf(self.L, ud) }
    }

    /// Changes the allocator function of a given state to `f` with user data
    /// `ud`.
    pub fn setallocf(&mut self, f: Alloc, ud: *mut libc::c_void) {
        unsafe { raw::lua_setallocf(self.L, f, ud) }
    }

    /// Pop n elements from the stack.
    /// Errors if the stack is smaller than n
    pub fn pop(&mut self, n: i32) {
        if n >= 0 {
            luaassert!(self, self.gettop() >= n, "pop: stack underflow");
        } else {
            luaassert!(self, self.gettop() >= (n+1).abs(), "pop: stack underflow");
        }
        unsafe { raw::lua_pop(self.L, n as c_int) }
    }

    /// Creates a new empty table and pushes it onto the stack.
    /// It is equivalent to .createtable(0, 0).
    pub fn newtable(&mut self) {
        self.checkstack_(1);
        unsafe { raw::lua_newtable(self.L) }
    }

    /// Sets the C function `f` as the new value of global `name`.
    /// Fails  the task if `name` has interior NULs.
    pub fn register(&mut self, name: &str, f: CFunction) {
        self.checkstack_(1);
        unsafe { name.with_c_str(|s| raw::lua_register(self.L, s, f) ) }
    }

    /// Pushes a C function onto the stack.
    pub fn pushcfunction(&mut self, f: CFunction) {
        self.checkstack_(1);
        unsafe { raw::lua_pushcfunction(self.L, f) }
    }

    /// Returns `true` if the value at the given acceptable index is a function
    /// (either C or Lua).
    pub fn isfunction(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isfunction(self.L, idx as c_int) }
    }

    /// Returns `true` if the value at the given acceptable index is a table.
    pub fn istable(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_istable(self.L, idx as c_int) }
    }

    /// Returns `true` if the value at the given acceptable index is a light
    /// userdata.
    pub fn islightuserdata(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_islightuserdata(self.L, idx) }
    }

    /// Returns `true` if the value at the given acceptable index is `nil`.
    pub fn isnil(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isnil(self.L, idx) }
    }

    /// Returns `true` if the value at the given acceptable index has type
    /// boolean.
    pub fn isboolean(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isboolean(self.L, idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a thread.
    pub fn isthread(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isthread(self.L, idx) }
    }

    /// Returns `true` if the given acceptable index is not valid.
    pub fn isnone(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isnone(self.L, idx) }
    }

    /// Returns `true` if the given acceptable index is not valid or if the
    /// value at this index is nil.
    pub fn isnoneornil(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        unsafe { raw::lua_isnoneornil(self.L, idx) }
    }

    /// Pops a value from the stack and sets it as the new value of global
    /// `name`. Fails the task if `name` has interior NULs.
    pub fn setglobal(&mut self, name: &str) {
        luaassert!(self, self.gettop() > 0, "setglobal: stack underflow");
        unsafe { name.with_c_str(|s| raw::lua_setglobal(self.L, s)) }
    }

    /// Pushes onto the stack the value of the global `name`.
    /// Fails the task if `name` has interior NULs.
    pub fn getglobal(&mut self, name: &str) {
        self.checkstack_(1);
        unsafe { name.with_c_str(|s| raw::lua_getglobal(self.L, s)) }
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
impl<T: CheckingBehavior> State<T> {
    /// Open the basic library.
    pub fn open_base(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_base);
        self.pushstring("");
        self.call(1, 0);
    }

    /// Opens the table library.
    pub fn open_table(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_table);
        self.pushstring(TABLIBNAME);
        self.call(1, 0);
    }

    /// Opens the io library.
    pub fn open_io(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_io);
        self.pushstring(IOLIBNAME);
        self.call(1, 0);
    }

    /// Opens the os library.
    pub fn open_os(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_os);
        self.pushstring(OSLIBNAME);
        self.call(1, 0);
    }

    /// Opens the string library.
    pub fn open_string(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_string);
        self.pushstring(STRLIBNAME);
        self.call(1, 0);
    }

    /// Opens the math library.
    pub fn open_math(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_math);
        self.pushstring(MATHLIBNAME);
        self.call(1, 0);
    }

    /// Opens the debug library.
    pub fn open_debug(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_debug);
        self.pushstring(DBLIBNAME);
        self.call(1, 0);
    }

    /// Opens the package library.
    pub fn open_package(&mut self) {
        self.checkstack_(2);
        self.pushcfunction(lib::raw::luaopen_package);
        self.pushstring(LOADLIBNAME);
        self.call(1, 0);
    }

    /// Opens all standard Lua libraries.
    pub fn openlibs(&mut self) {
        self.checkstack_(2);
        unsafe { lib::raw::luaL_openlibs(self.L) }
    }
}

pub static NoRef: i32 = aux::raw::LUA_NOREF as i32;
pub static RefNil: i32 = aux::raw::LUA_REFNIL as i32;

impl<T: CheckingBehavior> State<T> {
    /// In any case the function leaves the table on the top of the stack.
    pub fn registerlib(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        // internally, luaL_registerlib seems to use 4 stack slots
        self.checkstack_(4);
        if libname.is_none() {
            luaassert!(self, self.gettop() >= 1, "registerlib: stack underflow");
        }
        let mut cstrs = Vec::with_capacity(l.len());
        let mut l_ = Vec::with_capacity(l.len()+1);
        for &(name, func) in l.iter() {
            let cstr = name.to_c_str();
            cstr.with_ref(|name| l_.push(aux::raw::luaL_Reg{ name: name, func: Some(func) }));
            cstrs.push(cstr);
        }
        l_.push(aux::raw::luaL_Reg{ name: ptr::null(), func: None });
        let libcstr = libname.map(|s| s.to_c_str());
        let libname_ = libcstr.map_or(ptr::null(), |cstr| cstr.with_ref(|p| p));
        unsafe { aux::raw::luaL_register(self.L, libname_, l_.as_ptr()) }
    }

    /// Pushes onto the stack the field `e` from the metatable of the object at
    /// index `obj`. If the object does not have a metatable, or if the
    /// metatable does not have this field, returns `false` and pushes nothing.
    pub fn getmetafield(&mut self, obj: i32, e: &str) -> bool {
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_getmetafield uses 2 stack slots
        unsafe {
            e.with_c_str(|e| aux::raw::luaL_getmetafield(self.L, obj as c_int, e)) != 0
        }
    }

    /// If the object at index `obj` has a metatable and this metatable has a
    /// field `e`, this method calls this field and passes the object as its
    /// only argument. In this case this method returns `true` and pushes onto
    /// the stack the value returned by the call. If there is no metatable or
    /// no metamethod, this method returns `false` (without pushing any value
    /// on the stack).
    pub fn callmeta(&mut self, obj: i32, e: &str) -> bool {
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_callmeta uses 2 stack slots
        unsafe {
            e.with_c_str(|e| aux::raw::luaL_callmeta(self.L, obj as c_int, e)) != 0
        }
    }

    /// where `location` is produced by where(), `func` is the name of the
    /// current function, and `rt` is the type name of the actual argument.
    pub fn typerror(&mut self, narg: i32, tname: &str) -> ! {
        self.check_acceptable(narg);
        // NB: stack checking is not necessary
        unsafe {
            tname.with_c_str(|tname| aux::raw::luaL_typerror(self.L, narg as c_int, tname));
        }
        unreachable!()
    }

    ///   bad argument #<narg> to <func> (<extramsg>)
    pub fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        // NB: stack checking is not necessary
        extramsg.with_c_str(|msg| {
            unsafe { aux::raw::luaL_argerror(self.L, narg as c_int, msg); }
            unreachable!()
        })
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    /// If the string is not utf-8, returns None.
    pub fn checkstring(&mut self, narg: i32) -> Option<&'static str> {
        self.check_acceptable(narg);
        str::from_utf8(self.checkbytes(narg))
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    /// Checks whether the function argument `narg` is a lua string, and
    /// returns it as a byte vector. See checkstring() for caveats.
    pub fn checkbytes(&mut self, narg: i32) -> &'static [u8] {
        self.check_acceptable(narg);
        let mut sz: libc::size_t = 0;
        unsafe {
            let s = aux::raw::luaL_checklstring(self.L, narg, &mut sz);
            slice::raw::buf_as_slice(s as *u8, sz as uint, |b| {
                mem::transmute::<&[u8], &'static [u8]>(b)
            })
        }
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    /// If the argument is a string, but is not utf-8, returns None.
    pub fn optstring(&mut self, narg: i32, d: &'static str) -> Option<&'static str> {
        self.check_acceptable(narg);
        str::from_utf8(self.optbytes(narg, d.as_bytes()))
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    /// If the function argument `narg` is a lua string, returns this string
    /// asa byte vector.  See optstring() for more information.
    pub fn optbytes(&mut self, narg: i32, d: &'static [u8]) -> &'static [u8] {
        self.check_acceptable(narg);
        let mut sz: libc::size_t = 0;
        unsafe {
            let s = d.with_c_str(|d| aux::raw::luaL_optlstring(self.L, narg, d, &mut sz));
            slice::raw::buf_as_slice(s as *u8, sz as uint, |b| {
                mem::transmute::<&[u8], &'static [u8]>(b)
            })
        }
    }

    /// Checks whether the function argument `narg` is a number and returns the
    /// number.
    pub fn checknumber(&mut self, narg: i32) -> f64 {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_checknumber(self.L, narg as c_int) as f64
        }
    }

    /// If the function argument `narg` is a number, returns this number. If
    /// the argument is absent or is nil, returns `d`. Otherwise, throws an
    /// error.
    pub fn optnumber(&mut self, narg: i32, d: f64) -> f64 {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_optnumber(self.L, narg as c_int, d as raw::lua_Number) as f64
        }
    }

    /// Checks whether the function argument `narg` is a number and returns it
    /// as an int.
    pub fn checkinteger(&mut self, narg: i32) -> int {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_checkinteger(self.L, narg as c_int) as int
        }
    }

    /// If the function argument `narg` is a number, returns this number cast
    /// to an int. If this argument is absent or nil, returns `d`. Otherwise,
    /// raises an error.
    pub fn optinteger(&mut self, narg: i32, d: int) -> int {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_optinteger(self.L, narg as c_int, d as raw::lua_Integer) as int
        }
    }

    /// Checks whether the function argument `narg` has type `t`.
    pub fn checktype(&mut self, narg: i32, t: Type) {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_checktype(self.L, narg as c_int, t as c_int)
        }
    }

    /// Checks whether the function has an argument of any type (including nil)
    /// at position `narg`.
    pub fn checkany(&mut self, narg: i32) {
        self.check_acceptable(narg);
        unsafe {
            aux::raw::luaL_checkany(self.L, narg as c_int)
        }
    }

    /// In both cases pushes onto the stack the final value associated with
    /// `tname` in the registry.
    pub fn newmetatable(&mut self, tname: &str) -> bool {
        self.checkstack_(2); // uses 1 or 2 stack slots internally
        unsafe {
            tname.with_c_str(|tname| aux::raw::luaL_newmetatable(self.L, tname)) != 0
        }
    }

    /// Checks whether the function argument `narg` is a userdata of the type
    /// `tname` (see newmetatable()). The userdata pointer is returned.
    pub fn checkudata(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        self.check_acceptable(narg);
        self.checkstack_(2); // uses 2 stack slots internally
        unsafe {
            tname.with_c_str(|tname| aux::raw::luaL_checkudata(self.L, narg as c_int, tname))
        }
    }

    /// Pushes onto the stack a string identifying the current position of the
    /// control at level `lvl` in the call stack.
    /// Level 0 is the running function, level 1 is the function that called
    /// the running function, etc.
    pub fn where(&mut self, lvl: i32) {
        // luaL_where() internally uses lua_pushfstring(), which manages stack size itself
        // so we don't need to call checkstack()
        unsafe { aux::raw::luaL_where(self.L, lvl as c_int) }
    }

    /// Raises an error with the given string.
    /// It also adds at the beginning of the message the file name and line
    /// number where the error occurred, if this information is available.
    pub fn errorstr(&mut self, s: &str) -> ! {
        self.checkstack_(2);
        self.where(1);
        self.pushstring(s);
        self.concat(2);
        unsafe { raw::lua_error(self.L); }
        unreachable!()
    }

    /// Fails the task if `def` or any list key has interior NULs
    pub fn checkoption<'a, T>(&mut self, narg: i32, def: Option<&str>, lst: &'a [(&str,T)])
                                    -> &'a T {
        self.check_acceptable(narg);
        unsafe {
            let def_cstr = def.map(|d| d.to_c_str());
            let defp = def_cstr.as_ref().map_or(ptr::null(), |c| c.with_ref(|p| p));
            let mut lst_cstrs = Vec::with_capacity(lst.len());
            let mut lstv = Vec::with_capacity(lst.len()+1);
            for &(k,_) in lst.iter() {
                let cstr = k.to_c_str();
                let p = cstr.with_ref(|p| p);
                lst_cstrs.push(cstr);
                lstv.push(p);
            }
            lstv.push(ptr::null());
            let i = aux::raw::luaL_checkoption(self.L, narg as c_int, defp, lstv.as_ptr()) as uint;
            lst[i].ref1()
        }
    }

    /// If the object at the top of the stack is nil, ref_() returns the
    /// constant RefNil. The constant NoRef is guaranteed to be different from
    /// any reference returned by ref_().
    pub fn ref_(&mut self, t: i32) -> i32 {
        self.check_valid(t, true);
        self.checkstack_(1); // luaL_ref internally uses 1 stack slot
        unsafe { aux::raw::luaL_ref(self.L, t as c_int) as i32 }
    }

    /// If ref is NoRef or RefNil, unref() does nothing.
    pub fn unref(&mut self, t: i32, r: i32) {
        self.check_acceptable(t);
        self.checkstack_(1); // luaL_unref internally uses 1 stack slot
        unsafe { aux::raw::luaL_unref(self.L, t as c_int, r as c_int) }
    }

    /// Loads a file as a Lua chunk (but does not run it).
    /// If the `filename` is None, this loads from standard input.
    /// Fails the task if `filename` has any interior NULs.
    pub fn loadfile(&mut self, filename: Option<&path::Path>) -> Result<(),LoadFileError> {
        self.checkstack_(1);
        let cstr = filename.map(|p| p.to_c_str());
        let ptr = cstr.as_ref().map_or(ptr::null(), |cstr| cstr.with_ref(|p| p));
        unsafe {
            match aux::raw::luaL_loadfile(self.L, ptr) {
                0 => Ok(()),
                raw::LUA_ERRSYNTAX => Err(LoadFileError::ErrSyntax),
                raw::LUA_ERRMEM => Err(LoadFileError::ErrMem),
                aux::raw::LUA_ERRFILE => Err(LoadFileError::ErrFile),
                _ => self.errorstr("loadfile: unexpected error from luaL_loadfile")
            }
        }
    }

    /// Loads a buffer as a Lua chunk (but does not run it).
    /// As far as Rust is concerned, this differ from loadstring() in that a
    /// name for the chunk is provided. It also allows for NUL bytes, but I
    /// expect Lua won't like those.
    /// Fails the task if `name` has any interior NULs.
    pub fn loadbuffer(&mut self, buf: &str, name: &str) -> Result<(),LoadError> {
        self.checkstack_(1);
        let bp = buf.as_ptr() as *libc::c_char;
        let bsz = buf.len() as libc::size_t;
        unsafe {
            match name.with_c_str(|name| aux::raw::luaL_loadbuffer(self.L, bp, bsz, name)) {
                0 => Ok(()),
                raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
                raw::LUA_ERRMEM => Err(LoadError::ErrMem),
                _ => self.errorstr("loadbuffer: unexpected error from luaL_loadbuffer")
            }
        }
    }

    /// Loads a string as a Lua chunk (but does not run it).
    /// Fails the task if `s` has any interior NULs.
    pub fn loadstring(&mut self, s: &str) -> Result<(),LoadError> {
        self.checkstack_(1);
        unsafe {
            match s.with_c_str(|s| aux::raw::luaL_loadstring(self.L, s)) {
                0 => Ok(()),
                raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
                raw::LUA_ERRMEM => Err(LoadError::ErrMem),
                _ => self.errorstr("loadstring: unexpected error from luaL_loadstring")
            }
        }
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    /// Creates a copy of string `s` by replacing any occurrence of the string
    /// `p` with the string `r`. Pushes the resulting string on the stack and
    /// returns it.
    pub fn gsub(&mut self, s: &str, p: &str, r: &str) -> &'static str {
        self.checkstack_(MINSTACK/2);
        let s_ = s.to_c_str();
        let p_ = p.to_c_str();
        let r_ = r.to_c_str();
        let sp = s_.with_ref(|p| p);
        let pp = p_.with_ref(|p| p);
        let rp = r_.with_ref(|p| p);
        unsafe {
            let res = aux::raw::luaL_gsub(self.L, sp, pp, rp);
            let cstr = CString::new(res, false);
            let res = cstr.as_str().unwrap();
            mem::transmute::<&str,&'static str>(res)
        }
    }

    /// Fails the task if `extramsg` has interior NULs.
    pub fn argcheck(&mut self, cond: bool, narg: i32, extramsg: &str) {
        // NB: stack checking is not necessary
        extramsg.with_c_str(|msg| unsafe {
            aux::raw::luaL_argcheck(self.L, cond, narg as c_int, msg)
        })
    }

    /// Loads and runs the given file. It returns `true` if there are no errors
    /// or `false` in case of errors.
    pub fn dofile(&mut self, filename: Option<&path::Path>) -> bool {
        self.checkstack_(1);
        unsafe {
            let cstr = filename.map(|p| p.to_c_str());
            let name = cstr.map_or(ptr::null(), |c| c.with_ref(|p| p));
            aux::raw::luaL_dofile(self.L, name) == 0
        }
    }

    /// Loads and runs the given string. It returns `true` if there are no
    /// errors or `false` in case of errors.
    pub fn dostring(&mut self, s: &str) -> bool {
        self.checkstack_(1);
        unsafe { s.with_c_str(|s| aux::raw::luaL_dostring(self.L, s)) == 0 }
    }

    /// Pushes onto the stack the metatable associated with the name `tname` in
    /// the registry (see newmetatable()).
    pub fn getmetatable_reg(&mut self, tname: &str) {
        self.getfield(REGISTRYINDEX, tname)
    }

    /// Initializes and returns a Buffer
    pub fn buffinit<'a>(&'a mut self) -> Buffer<'a, T> {
        #![inline]
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

/// String buffer for building Lua strings piecemeal.
///
/// The Buffer assumes it needs longjmp safety, like ExternState.
pub struct Buffer<'a, T> {
    B: aux::raw::luaL_Buffer,
    /// A &mut pointer to the ExternState that created this Buffer.
    /// The buffer internally holds on to the *lua_Buffer that the State wraps,
    /// so to ensure safety it also borrows the &mut ExternState. Use this
    /// field to get mutable access to the State while the buffer is alive.
    pub L: &'a mut State<T>
}

/// Size of the internal buffer used by Buffer and returned by prepbuffer()
pub static BUFFERSIZE: uint = aux::raw::LUAL_BUFFERSIZE as uint;

impl<'a, T: CheckingBehavior> Buffer<'a, T> {
    /// Adds the byte `c` to the buffer.
    pub fn addbyte(&mut self, c: u8) {
        #![inline]
        // don't call through to luaL_addchar, because we want to insert a call to checkstack()
        // iff we have to prep the buffer.
        unsafe {
            let startp: *mut libc::c_char = &mut self.B.buffer[0];
            if self.B.p >= startp.offset(aux::raw::LUAL_BUFFERSIZE as int) {
                self.L.checkstack_(1);
                aux::raw::luaL_prepbuffer(&mut self.B);
            }
            *self.B.p = c as libc::c_char;
            self.B.p = self.B.p.offset(1);
        }
    }

    /// Adds the char `c` as utf-8 bytes to the buffer.
    pub fn addchar(&mut self, c: char) {
        #![inline]
        let mut buf = [0u8, ..4];
        let count = c.encode_utf8(buf);
        self.addbytes(buf.slice_to(count));
    }

    /// Adds to the buffer a string of length `n` previously copied to the
    /// buffer area (see prepbuffer()).
    pub fn addsize(&mut self, n: uint) {
        #![inline]
        unsafe { aux::raw::luaL_addsize(&mut self.B, n as libc::size_t) }
    }

    /// Returns a pointer to an array of size BUFFERSIZE where you can copy a
    /// string to be added to the buffer. After copying the string into this
    /// space you must call addsize() with the size of the string to actually
    /// add it to the buffer.
    pub fn prepbuffer(&mut self) -> &mut [u8, ..aux::raw::LUAL_BUFFERSIZE] {
        #![inline]
        self.L.checkstack_(1);
        // luaL_prepbuffer ends up returning the buffer field.
        // Rather than unsafely trying to transmute that to the array, just return the field
        // ourselves.
        unsafe {
            aux::raw::luaL_prepbuffer(&mut self.B);
            mem::transmute::<&mut [i8, ..aux::raw::LUAL_BUFFERSIZE],
                              &mut [u8, ..aux::raw::LUAL_BUFFERSIZE]>(&mut self.B.buffer)
        }
    }

    /// Adds the string to the buffer.
    pub fn addstring(&mut self, s: &str) {
        #![inline]
        self.addbytes(s.as_bytes())
    }

    /// Adds the byte vector to the buffer.
    pub fn addbytes(&mut self, bytes: &[u8]) {
        #![inline]
        // luaL_addlstring() just iterates over the string calling addchar().
        // We want our checkstack calls, so let's just do that here instead directly.
        for &b in bytes.iter() {
            self.addbyte(b);
        }
    }

    /// Adds the value at the top of the stack to the buffer. Pops the value.
    ///
    /// This is the only method on string buffers that can (and must) be called
    /// with an extra element on the stack, which is the value to be added to
    /// the buffer.
    pub fn addvalue(&mut self) {
        #![inline]
        luaassert!(self.L, self.L.gettop() >= 1, "addvalue: stack underflow");
        self.L.checkstack_(1); // luaL_addvalue() needs this if the value is too large
        unsafe { aux::raw::luaL_addvalue(&mut self.B) }
    }

    /// Finishes the use of the buffer, leaving the final string on top of the
    /// stack.
    pub fn pushresult(mut self) {
        #![inline]
        self.L.checkstack_(1); // possibly needed for the emptybuffer
        unsafe { aux::raw::luaL_pushresult(&mut self.B) }
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
        #![inline]
        std::default::Default::default()
    }
}

impl<T: CheckingBehavior> State<T> {
    /// This function returns a Debug structure with an identification of the
    /// activation record of the function executing at a given level. Level 0
    /// is the current running function, whereas level n+1 is the function that
    /// has called level n. When there are no errors, getstack() returns
    /// Some(Debug); when called with a level greater than the stack depth, it
    /// returns None.
    pub fn getstack(&mut self, level: i32) -> Option<Debug> {
        let mut ar: Debug = std::default::Default::default();
        if unsafe { raw::lua_getstack(self.L, level as c_int, &mut ar) != 0 } {
            Some(ar)
        } else {
            None
        }
    }

    /// Fails the task if `what` has interior NULs.
    pub fn getinfo(&mut self, what: &str, ar: &mut Debug) -> bool {
        if what.starts_with(">") {
            luaassert!(self, self.gettop() >= 1 && self.isfunction(-1),
                       "getinfo: top stack value is not a function");
        }
        if what.find(&['f', 'L']).is_some() {
            self.checkstack_(1);
        }
        unsafe { what.with_c_str(|w| raw::lua_getinfo(self.L, w, ar)) != 0 }
    }

    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn getlocal<'a>(&mut self, ar: &'a Debug, n: i32) -> Option<&'a [u8]> {
        self.checkstack_(1);
        unsafe {
            let res = raw::lua_getlocal(self.L, ar, n as c_int);
            c_str_to_bytes(res)
        }
    }

    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn setlocal<'a>(&mut self, ar: &'a mut Debug, n: i32) -> Option<&'a [u8]> {
        luaassert!(self, self.gettop() >= 1, "setlocal: stack underflow");
        unsafe {
            let res = raw::lua_setlocal(self.L, ar, n as c_int);
            c_str_to_bytes(res)
        }
    }

    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn getupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        self.check_acceptable(funcidx);
        self.checkstack_(1);
        unsafe {
            let res = raw::lua_getupvalue(self.L, funcidx as c_int, n as c_int);
            c_str_to_bytes(res)
        }
    }

    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn setupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        self.check_acceptable(funcidx);
        self.checkstack_(1);
        unsafe {
            let res = raw::lua_setupvalue(self.L, funcidx as c_int, n as c_int);
            c_str_to_bytes(res)
        }
    }

    /// A hook is disabled by setting `mask` to zero.
    pub fn sethook(&mut self, f: Hook, mask: i32, count: i32) {
        unsafe { raw::lua_sethook(self.L, f, mask as c_int, count as c_int); }
    }

    /// Returns the current hook function
    pub fn gethook(&mut self) -> Hook {
        unsafe { raw::lua_gethook(self.L) }
    }

    /// Returns the current hook mask
    pub fn gethookmask(&mut self) -> i32 {
        unsafe { raw::lua_gethookmask(self.L) as i32 }
    }

    /// Returns the current hook count
    pub fn gethookcount(&mut self) -> i32 {
        unsafe { raw::lua_gethookcount(self.L) as i32 }
    }
}

unsafe fn c_str_to_bytes<'a>(cstr: *libc::c_char) -> Option<&'a [u8]> {
    #![inline]
    if cstr.is_null() {
        None
    } else {
        let cstr = CString::new(cstr, false);
        let bytes = cstr.as_bytes();
        Some(mem::transmute::<&[u8],&'a [u8]>(bytes))
    }
}
