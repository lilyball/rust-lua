//! Lua 5.1 bindings for Rust

#![crate_name = "lua"]

#![crate_type = "rlib"]

#![warn(missing_docs)]
#![allow(non_snake_case,unstable)]

extern crate libc;

use libc::c_int;
use std::{fmt, mem, path, ptr, str, slice};
use std::ffi::{self, CString};
use std::num::SignedInt;

/// Human-readable major version string
pub const VERSION: &'static str = config::LUA_VERSION;
/// Human-readable release version string
pub const RELEASE: &'static str = config::LUA_RELEASE;
/// Machine-readable version number
pub const VERSION_NUM: isize = config::LUA_VERSION_NUM as isize;

/// Value for lua_call that means return all results
pub const MULTRET: i32 = raw::MULTRET as i32;

/// Minimum Lua stack available to a C function
pub const MINSTACK: i32 = config::LUA_MINSTACK as i32;

/// Pseudo-index for the registry
pub const REGISTRYINDEX: i32 = raw::LUA_REGISTRYINDEX as i32;
/// Pseudo-index for the thread environment
pub const GLOBALSINDEX: i32 = raw::LUA_GLOBALSINDEX as i32;
/// Pseudo-index for the running C function environment
pub const ENVIRONINDEX: i32 = raw::LUA_ENVIRONINDEX as i32;

/// Calculates the pseudo-index for the upvalue at the given index.
/// Any index in the range [1,256] produces an acceptable index.
/// Any index outside that range will likely produce an unacceptable index.
pub fn upvalueindex(n: i32) -> i32 {
    #![inline]
    raw::lua_upvalueindex(n as c_int) as i32
}

include!(concat!(env!("OUT_DIR"), "/config.rs"));

#[allow(missing_docs)]
pub mod raw;
#[allow(missing_docs)]
pub mod aux;

#[path = "lualib.rs"]
#[allow(missing_docs)]
pub mod lib;

#[path="macro.rs"]
mod macros;

#[cfg(test)]
mod tests;

macro_rules! luaassert{
    ($state:expr, $cond:expr, $msg:expr) => {
        if !$cond {
            $state.errorstr($msg.as_slice());
        }
    };
    ($state:expr, $cond:expr, $($arg:expr),+) => {
        if !$cond {
            let msg = format!($($arg),+);
            $state.errorstr(msg.as_slice());
        }
    }
}

/// Lua value types
#[derive(Clone,Copy,PartialEq,Eq,Show)]
pub enum Type {
    /// Type for nil
    Nil = raw::LUA_TNIL as isize,
    /// Type for booleans
    Boolean = raw::LUA_TBOOLEAN as isize,
    /// Type for light userdata
    LightUserdata = raw::LUA_TLIGHTUSERDATA as isize,
    /// Type for numbers
    Number = raw::LUA_TNUMBER as isize,
    /// Type for strings
    String = raw::LUA_TSTRING as isize,
    /// Type for tables
    Table = raw::LUA_TTABLE as isize,
    /// Type for functions
    Function = raw::LUA_TFUNCTION as isize,
    /// Type for userdata
    Userdata = raw::LUA_TUSERDATA as isize,
    /// Type for threads
    Thread = raw::LUA_TTHREAD as isize
}

impl Type {
    /// Returns the name of the type
    pub fn name(&self) -> &'static str {
        unsafe {
            // NB: lua_typename() doesn't actually use its state parameter
            let s = raw::lua_typename(ptr::null_mut(), *self as libc::c_int);
            mem::transmute::<&str,&'static str>(str::from_utf8(ffi::c_str_to_bytes(&s)).unwrap())
        }
    }
}

/// Garbage collection options (used with State.gc())
//#[allow(dead_code)] // FIXME(rust-lang/rust#17632): dead_code warning is wrong here
#[derive(Copy)]
pub enum GC {
    /// Stops the garbage collector
    Stop = raw::LUA_GCSTOP as isize,
    /// Restarts the garbage collector
    Restart = raw::LUA_GCRESTART as isize,
    /// Performs a full garbage-collection cycle
    Collect = raw::LUA_GCCOLLECT as isize,
    /// Returns the current amount of memory (in Kbytes) in use by Lua
    Count = raw::LUA_GCCOUNT as isize,
    /// Returns the remainder of dividing the current amount of bytes in memory in use by Lua
    /// by 1024
    CountB = raw::LUA_GCCOUNTB as isize,
    /// Performs an incremental step of garbage collection. The step "size" is controlled by
    /// `data` (larger values mean more steps) in a non-specified way. If you want to control
    /// the step size you must experimentally tune hte value of `data`. The function returns
    /// 1 if the step finished a garbage-collection cycle.
    Step = raw::LUA_GCSTEP as isize,
    /// Sets `data` as the new value for the pause of the collector. The function returns the
    /// previous value of the pause.
    SetPause = raw::LUA_GCSETPAUSE as isize,
    /// Sets `data` as the new value for the step multiplier of the collector. The function
    /// returns the previous value of the step multiplier.
    SetStepMul = raw::LUA_GCSETSTEPMUL as isize
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
#[derive(Copy)]
pub enum LoadError {
    /// Syntax error during pre-compilation
    ErrSyntax = raw::LUA_ERRSYNTAX as isize,
    /// Memory allocation error
    ErrMem = raw::LUA_ERRMEM as isize
}

impl fmt::Show for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::ErrSyntax => f.pad("syntax error"),
            LoadError::ErrMem => f.pad("memory allocation error")
        }
    }
}

/// State.loadfile() errors
#[derive(Copy)]
pub enum LoadFileError {
    /// Syntax error during pre-compilation
    ErrSyntax = raw::LUA_ERRSYNTAX as isize,
    /// Memory allocation error
    ErrMem = raw::LUA_ERRMEM as isize,
    /// Cannot read/open the file
    ErrFile = aux::raw::LUA_ERRFILE as isize
}

impl fmt::Show for LoadFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadFileError::ErrSyntax => f.pad("syntax error"),
            LoadFileError::ErrMem => f.pad("memory allocation error"),
            LoadFileError::ErrFile => f.pad("file read/open error")
        }
    }
}

/// State.pcall() errors
#[derive(Copy)]
pub enum PCallError {
    /// Runtime error
    ErrRun = raw::LUA_ERRRUN as isize,
    /// Memory allocation error
    ErrMem = raw::LUA_ERRMEM as isize,
    /// Error while running the error handler function
    ErrErr = raw::LUA_ERRERR as isize
}

impl PCallError {
    /// Converts an error code from `lua_pcall()` into a PCallError
    pub fn from_code(code: c_int) -> Option<PCallError> {
        match code {
            raw::LUA_ERRRUN => Some(PCallError::ErrRun),
            raw::LUA_ERRMEM => Some(PCallError::ErrMem),
            raw::LUA_ERRERR => Some(PCallError::ErrErr),
            _ => None,
        }
    }
}

impl fmt::Show for PCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PCallError::ErrRun => f.pad("runtime error"),
            PCallError::ErrMem => f.pad("memory allocation error"),
            PCallError::ErrErr => f.pad("error handler func error")
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
#[repr(C)]
pub struct State {
    L: *mut raw::lua_State,
    _stackspace: i32
}

impl Drop for State {
    fn drop(&mut self) {
        if !self.L.is_null() {
            unsafe {
                raw::lua_close(self.L);
            }
            self.L = ptr::null_mut();
        }
    }
}

/// ExternState is a Lua State that was created from a raw::lua_State value.
/// Every error-throwing function is assumed to be using longjmp instead of
/// task failure.
///
/// See State for more information.
// NB: layout must be identical to State
// If Drop is ever implemented, add unsafe_no_drop_flag
#[repr(C)]
pub struct ExternState<'a> {
    L: *mut raw::lua_State,
    stackspace: i32
}

/// RawState is a Lua State that represents raw, unchecked access. All
/// functions eschew safety in favor of speed. Like ExternState, all
/// error-throwing functions are assumed to be using longjmp.
// NB: layout must be identical to State
// If Drop is ever implemented, add unsafe_no_drop_flag
#[repr(C)]
pub struct RawState<'a> {
    L: *mut raw::lua_State,
    stackspace: i32
}

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
            let L = raw::lua_newstate(alloc, ptr::null_mut());
            if !L.is_null() {
                raw::lua_atpanic(L, panic);
                Some(State{ L: L, _stackspace: MINSTACK })
            } else {
                None
            }
        };

        extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
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
        extern "C" fn panic(L: *mut raw::lua_State) -> c_int {
            unsafe {
                let s = RawState::from_lua_State(L).describe_(-1, false);
                panic!("unprotected error in call to Lua API ({})", s);
            }
        }
    }
}

impl<'l> ExternState<'l> {
    /// Wraps a *raw::lua_State in a ExternState.
    pub unsafe fn from_lua_State(L: *mut raw::lua_State) -> ExternState<'static> {
        #![inline]
        ExternState{ L: L, stackspace: MINSTACK }
    }
}

impl<'l> RawState<'l> {
    /// Wraps a *raw::lua_State in a RawState.
    pub unsafe fn from_lua_State(L: *mut raw::lua_State) -> RawState<'static> {
        #![inline]
        RawState{ L: L, stackspace: MINSTACK }
    }
}

// State conversion
impl State {
    /// Returns the same state as an ExternState
    pub fn as_extern<'a>(&'a mut self) -> &'a mut ExternState<'a> {
        #![inline]
        unsafe { mem::transmute(self) }
    }

    /// Returns the same state as a RawState
    pub fn as_raw<'a>(&'a mut self) -> &'a mut RawState<'a> {
        #![inline]
        unsafe { mem::transmute(self) }
    }
}

impl<'a> ExternState<'a> {
    /// Returns the same state as a RawState
    pub fn as_raw(&mut self) -> &'a mut RawState<'a> {
        #![inline]
        unsafe { mem::transmute(self) }
    }
}

impl State {
    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        #![inline]
        self.L
    }
}

impl<'l> ExternState<'l> {
    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        #![inline]
        self.L
    }
}

impl<'l> RawState<'l> {
    /// Provides unsafe access to the underlying *lua_State
    pub unsafe fn get_lua_State(&mut self) -> *mut raw::lua_State {
        #![inline]
        self.L
    }
}


impl State {
    /// Creates a new thread, pushes it on the stack, and returns a `State`
    /// that represents this new thread. The new state returned by this
    /// function shares with the original state all global objects (such as
    /// tables), but has an independent execution stack.
    ///
    /// This new state does not get explicitly closed. Threads are subject to
    /// garbage collection, like any Lua object.
    pub fn newthread(&mut self) -> State {
        #![inline(always)]
        unsafe { self.as_raw().newthread() }
    }

    /// Sets a new panic function and returns the old one.
    ///
    /// The panic function can access the error message at the top of the stack.
    ///
    /// The default panic function installed by this library calls panic!() with
    /// the error message. Your panic function should either call through to
    /// the default one, or should panic!() itself. Otherwise, the application
    /// will be terminated.
    pub unsafe fn atpanic(&mut self, panicf: CFunction) -> CFunction {
        #![inline(always)]
        self.as_raw().atpanic(panicf)
    }

    /// Returns the textual description of the value at the given acceptable index.
    /// Returns "" if the given index is non-valid.
    pub fn describe(&mut self, idx: i32) -> String {
        #![inline(always)]
        unsafe { self.as_extern().describe(idx) }
    }

    /// Variant of describe_() that does not push on to the stack. describe()
    /// may push new values onto the stack temporarily. Notably, it may do this
    /// to avoid converting the existing value's type. This method allows this
    /// behavior to be disabled. If usestack is true, this method may require 1
    /// free slot on the stack.
    pub fn describe_(&mut self, idx: i32, usestack: bool) -> String {
        #![inline(always)]
        unsafe { self.as_extern().describe_(idx, usestack) }
    }

    /// Returns the index of the top element of the stack.
    /// Indexes start at 1. 0 means the stack is empty.
    pub fn gettop(&mut self) -> i32 {
        #![inline(always)]
        self.as_extern().gettop()
    }

    /// Sets the stack top to the given acceptable index, or 0.
    /// If the new top is larger than the old one, new elements are filled with
    /// nil.
    /// If the index is 0, all stack elements are removed.
    pub fn settop(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().settop(idx) }
    }

    /// Pushes a copy of the element at the given valid index onto the stack.
    pub fn pushvalue(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().pushvalue(idx) }
    }

    /// Removes the element at the given valid index, shifting other elements
    /// as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn remove(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().remove(idx) }
    }

    /// Moves the top element into the given valid index, shifting existing
    /// elements as needed.
    /// Pseudo-indices are not valid for this call.
    pub fn insert(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().insert(idx) }
    }

    /// Moves the top element into the given valid index and replaces the
    /// existing value, without shifting any other elements.
    pub fn replace(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().replace(idx) }
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Returns false if it cannot grow the stack as requested.
    pub fn checkstack(&mut self, extra: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().checkstack(extra) }
    }

    /// Ensures the stack contains at least `extra` free slots on the stack.
    /// Throws an error if it cannot grow the stack.
    pub fn checkstack_(&mut self, extra: i32) {
        #![inline(always)]
        unsafe { self.as_extern().checkstack_(extra) }
    }

    /// Exchanges values between different threads of the same global state.
    /// This method pops n values from the stack `self`, and pushes them to the
    /// stack `to`.
    ///
    /// Note: this method is unsafe because it cannot check to ensure that both
    /// threads belong to the same global state.
    ///
    /// Despite being unsafe, it still checks the validity of `n`.
    pub unsafe fn xmove(&mut self, to: &mut State, n: i32) {
        #![inline(always)]
        self.as_extern().xmove(to.as_extern(), n)
    }

    /// Returns `true` if the value at the given acceptable index is a number,
    /// or a string convertible to a number.
    pub fn isnumber(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isnumber(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a string
    /// or a number (which is always convertible to a string).
    pub fn isstring(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isstring(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a C
    /// function.
    pub fn iscfunction(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().iscfunction(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a userdata
    /// (either full or light).
    pub fn isuserdata(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isuserdata(idx) }
    }

    /// Returns the type of the value at the given acceptable index.  If the
    /// given index is non-valid, returns None.
    pub fn type_(&mut self, idx: i32) -> Option<Type> {
        #![inline(always)]
        unsafe { self.as_extern().type_(idx) }
    }

    /// Returns the name of the type of the value at the given acceptable
    /// index.
    pub fn typename(&mut self, idx: i32) -> &'static str {
        #![inline(always)]
        unsafe { self.as_extern().typename(idx) }
    }

    /// Returns `true` if the two values in acceptable indices `index1` and
    /// `index2` are equal, following the semantics of the Lua == operator.
    /// Returns `false` if any indices are non-valid.
    pub fn equal(&mut self, index1: i32, index2: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().equal(index1, index2) }
    }

    /// Returns `true` if the two values in acceptable indices `index1` and
    /// `index2` are primitively equal (that is, without calling any
    /// metamethods). Returns `false` if any indices are non-valid.
    pub fn rawequal(&mut self, index1: i32, index2: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().rawequal(index1, index2) }
    }

    /// Returns `true` if the value at acceptable index `index1` is smaller
    /// than the value at acceptable index `index2`, following the semantics of
    /// the Lua < operator. Returns `false` if any indices are non-valid.
    pub fn lessthan(&mut self, index1: i32, index2: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().lessthan(index1, index2) }
    }

    /// Converts the Lua value at the given acceptable index to a f64. The Lua
    /// value must be a number or a string convertible to a number; otherwise,
    /// tonumber returns 0.
    pub fn tonumber(&mut self, idx: i32) -> f64 {
        #![inline(always)]
        unsafe { self.as_extern().tonumber(idx) }
    }

    /// Converts the Lua value at the given acceptable index to an isize. The Lua
    /// value must be a number or a string convertiable to a number; otherwise,
    /// toint returns 0.
    pub fn tointeger(&mut self, idx: i32) -> isize {
        #![inline(always)]
        unsafe { self.as_extern().tointeger(idx) }
    }

    /// Converts the value at the given acceptable index to a bool.
    /// Returns false when called with a non-valid index.
    pub fn toboolean(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().toboolean(idx) }
    }

    /// Converts the value at the given acceptable index to a string.
    ///
    /// Returns None if the value is not a number or a string.
    /// Returns None if the string value is not utf-8.
    ///
    /// Note: if the value is a number, this method changes the value in the
    /// stack to a string.  This may confuse lua_next if this is called during
    /// table traversal.
    pub fn tostring<'a>(&'a mut self, idx: i32) -> Option<&'a str> {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().tostring(idx)) }
    }

    /// Converts the value at the given acceptable index into a lua string, and
    /// returns it as a byte vector.
    /// Returns None if the value is not a number or a string.
    /// See tostring() for caveats.
    pub fn tobytes<'a>(&'a mut self, idx: i32) -> Option<&'a [u8]> {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().tobytes(idx)) }
    }

    /// Returns the "length" of the value at the given acceptable index.
    pub fn objlen(&mut self, idx: i32) -> usize {
        #![inline(always)]
        unsafe { self.as_extern().objlen(idx) }
    }

    /// Converts a value at the given acceptable index to a C function. The
    /// value must be a C function; otherwise, returns None.
    pub fn tocfunction(&mut self, idx: i32) -> Option<CFunction> {
        #![inline(always)]
        unsafe { self.as_extern().tocfunction(idx) }
    }

    /// If the value at the given acceptable index is a full userdata, returns
    /// its block address. If the value is a light userdata, returns its
    /// pointer. Otherwise, returns ptr::null().
    pub fn touserdata(&mut self, idx: i32) -> *mut libc::c_void {
        #![inline(always)]
        unsafe { self.as_extern().touserdata(idx) }
    }

    /// Converts the value at the given acceptable index to a Lua thread
    /// (represented as a State). This value must be a thread; otherwise, the
    /// method returns None.
    ///
    /// Note: the State return value does not make any assumptions about the
    /// available stack space. .checkstack() must be called in order to
    /// consider any non-valid index as acceptable.
    pub fn tothread(&mut self, idx: i32) -> Option<State> {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().tothread(idx)) }
    }

    /// Converts the value at the given acceptable index to a pointer. The
    /// value can be a userdata, a table, a thread, or a function.
    pub fn topointer(&mut self, idx: i32) -> *const libc::c_void {
        #![inline(always)]
        unsafe { self.as_extern().topointer(idx) }
    }

    /// Pushes a nil value onto the stack.
    pub fn pushnil(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().pushnil() }
    }

    /// Pushes a number with value `n` onto the stack
    pub fn pushnumber(&mut self, n: f64) {
        #![inline(always)]
        unsafe { self.as_extern().pushnumber(n) }
    }

    /// Pushes a number with value `n` onto the stack.
    pub fn pushinteger(&mut self, n: isize) {
        #![inline(always)]
        unsafe { self.as_extern().pushinteger(n) }
    }

    /// Pushes a string onto the stack
    pub fn pushstring(&mut self, s: &str) {
        #![inline(always)]
        unsafe { self.as_extern().pushstring(s) }
    }

    /// Pushes a byte vector onto the stack as a lua string
    pub fn pushbytes(&mut self, bytes: &[u8]) {
        #![inline(always)]
        unsafe { self.as_extern().pushbytes(bytes) }
    }

    /// Pushes a new C closure onto the stack.
    ///
    /// When a C function is created, it is possible to associate some values
    /// with it, thus creating a C closure; these values are then accessible to
    /// the function whenever it is called. These values must be pushed onto
    /// the stack (in order), then pushclosure() is called to create and push
    /// the C closure onto the stack. The argument `n` is the number of values
    /// that should be associated with the function. These values are popped
    /// from the stack.
    ///
    /// `n` must be in the range [0, 255]. Anything outside this range will
    /// throw an error.
    pub fn pushcclosure(&mut self, f: CFunction, n: i32) {
        #![inline(always)]
        unsafe { self.as_extern().pushcclosure(f, n) }
    }

    /// Pushes a boolean value onto the stack.
    pub fn pushboolean(&mut self, b: bool) {
        #![inline(always)]
        unsafe { self.as_extern().pushboolean(b) }
    }

    /// Pushes a light userdata onto the stack.
    pub fn pushlightuserdata(&mut self, p: *mut libc::c_void) {
        #![inline(always)]
        unsafe { self.as_extern().pushlightuserdata(p) }
    }

    /// Pushes the thread represented by `self` onto the stack. Returns `true`
    /// if this thread is the main thread of the state.
    pub fn pushthread(&mut self) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().pushthread() }
    }

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index and k is the value at the top of the stack. The key is
    /// popped from the stack.
    pub fn gettable(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().gettable(idx) }
    }

    /// Pushes onto the stack the value t[k], where t is the value at the given
    /// valid index. Fails the task if `k` has any interior NULs.
    pub fn getfield(&mut self, idx: i32, k: &str) {
        #![inline(always)]
        unsafe { self.as_extern().getfield(idx, k) }
    }

    /// Similar to gettable(), but does a raw access
    pub fn rawget(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().rawget(idx) }
    }

    /// Pushes onto the stack the value t[n], where t is the value at the given
    /// valid index. The access is raw; that is, it does not invoke
    /// metamethods.
    pub fn rawgeti(&mut self, idx: i32, n: i32) {
        #![inline(always)]
        unsafe { self.as_extern().rawgeti(idx, n) }
    }

    /// Creates a new empty table and pushes it into the stack. The new table
    /// has space pre-allocated for `narr` array elements and `nrec` non-array
    /// elements.
    pub fn createtable(&mut self, narr: i32, nrec: i32) {
        #![inline(always)]
        unsafe { self.as_extern().createtable(narr, nrec) }
    }

    /// This method allocates a new block of memory with the given size, pushes
    /// onto the stack a new full userdata with the block address, and returns
    /// this address.
    pub fn newuserdata(&mut self, size: usize) -> *mut libc::c_void {
        #![inline(always)]
        unsafe { self.as_extern().newuserdata(size) }
    }

    /// Pushes onto the stack the metatable of the value at the given
    /// acceptable index. If the index is not valid, or the value does not have
    /// a metatable, the function returns `false` and pushes nothing onto the
    /// stack.
    pub fn getmetatable(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().getmetatable(idx) }
    }

    /// Pushes onto the stack the environment table of the value at the given
    /// index.
    pub fn getfenv(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().getfenv(idx) }
    }

    /// Does the equivalent to t[k] = v, where t is the value at the given
    /// valid index, v is the value at the top of the stack, and k is the value
    /// just below the top.
    ///
    /// This function pops both the key and the value from the stack.
    pub fn settable(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().settable(idx) }
    }

    /// Does the equivalent to t[k] = v, where t is the value at the given
    /// valid index and v is the value at the top of the stack.
    ///
    /// This function pops the value from the stack.
    ///
    /// Fails the task if `k` contains interior NULs.
    pub fn setfield(&mut self, idx: i32, k: &str) {
        #![inline(always)]
        unsafe { self.as_extern().setfield(idx, k) }
    }

    /// Similar to settable(), but does a raw assignment.
    pub fn rawset(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().rawset(idx) }
    }

    /// Does the equivalent of t[n] = v, where t is the value at the given
    /// valid index and v is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. The assignment is raw;
    /// that is, it does not invoke metamethods.
    pub fn rawseti(&mut self, idx: i32, n: i32) {
        #![inline(always)]
        unsafe { self.as_extern().rawseti(idx, n) }
    }

    /// Pops a table from the stack and sets it as the new metatable for the
    /// value at the given acceptable index.
    pub fn setmetatable(&mut self, idx: i32) {
        #![inline(always)]
        unsafe { self.as_extern().setmetatable(idx) }
    }

    /// Pops a table from the stack and sets it as the new environment for the
    /// value at the given index. If the value at the given index is neither a
    /// function nor a thread nor a userdata, setfenv() returns `false`.
    /// Otherwise, returns `true`.
    pub fn setfenv(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().setfenv(idx) }
    }

    /// Calls a function.
    /// The function must be pushed first, followed by its arguments. `nargs`
    /// is the number of arguments. The function and its arguments are popped
    /// automatically.
    /// The function results are adjusted to `nresults`, unless `nresults` is
    /// `MULTRET`, in which case all function results are pushed.
    pub fn call(&mut self, nargs: i32, nresults: i32) {
        #![inline(always)]
        unsafe { self.as_extern().call(nargs, nresults) }
    }

    /// Calls a function in protected mode.
    ///
    /// If no error occurs, this behaves identically to call() and returns
    /// Ok(()). If there is any error, the error message is pushed onto the
    /// stack, and an error code is returned. The function and its arguments
    /// are always removed from the stack.
    ///
    /// If `errfunc` is 0, then the error message returned on the stack is
    /// exactly the original error message. Otherwise, `errfunc` is the stack
    /// index of an error handler function. It must not be a pseudo-index.
    pub fn pcall(&mut self, nargs: i32, nresults: i32, errfunc: i32) -> Result<(),PCallError> {
        #![inline(always)]
        unsafe { self.as_extern().pcall(nargs, nresults, errfunc) }
    }

    /// Loads a Lua chunk. If there are no errors, load() pushes the compiled
    /// chunk as a Lua function on top of the stack. Otherwise, it pushes an
    /// error message.
    ///
    /// This method only loads a chunk; it does not run it.
    ///
    /// load() automatically detects whether the chunk is text or binary, and
    /// loads it accordingly.
    ///
    /// The load() method uses a user-supplied `reader` function to read the
    /// chunk. The `data` argument is an opaque value passed to the reader
    /// function.
    ///
    /// The `chunkname` argument gives a name to the chunk, which is used for
    /// error messages and in debug information.
    ///
    /// Fails the task if `chunkname` contains interior NULs.
    pub fn load(&mut self, reader: Reader, data: *mut libc::c_void, chunkname: &str)
               -> Result<(),LoadError> {
        #![inline(always)]
        unsafe { self.as_extern().load(reader, data, chunkname) }
    }

    /// Dumps a function as a binary chunk. Receives a Lua function on the top
    /// of the stack and produces a binary chunk that, if loaded again, results
    /// in a function equivalent to the one dumped. As it produces parts of the
    /// chunk, dump() calls function `writer` with the given `data` to write
    /// them.
    ///
    /// The value returned is the error code returned by the last call to the
    /// writer; Ok(()) means no errors.
    ///
    /// This function does not pop the Lua function from the stack.
    pub fn dump(&mut self, writer: Writer, data: *mut libc::c_void) -> Result<(),i32> {
        #![inline(always)]
        unsafe { self.as_extern().dump(writer, data) }
    }

    /// Yields a coroutine.
    ///
    /// This function should only be called as the return expression of a C
    /// function, as follows:
    ///
    ///   return L.yield_(nresults);
    ///
    /// When a C function calls yield_() in that way, the running coroutine
    /// suspends its execution, and the call to resume() that started this
    /// coroutine returns. The parameter `nresults` is the number of values
    /// from the stack that are passed as the results to resume().
    pub fn yield_(&mut self, nresults: i32) -> c_int {
        #![inline(always)]
        unsafe { self.as_extern().yield_(nresults) }
    }

    /// Starts and resumes a coroutine in a given thread.
    ///
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
        #![inline(always)]
        unsafe { self.as_extern().resume(narg) }
    }

    /// Returns the status of the receiving thread.
    ///
    /// The status can be Ok(true) for a normal thread, Ok(false) if the thread
    /// is suspended, or Err(PCallError) if the thread finished its execution
    /// with an error.
    pub fn status(&mut self) -> Result<bool,PCallError> {
        #![inline(always)]
        unsafe { self.as_extern().status() }
    }

    /// Controls the garbage collector.
    ///
    /// This method performs several tasks, according to the value of the
    /// parameter `what`. See the `GC` enum for documentation on the various
    /// options.
    pub fn gc(&mut self, what: GC, data: i32) -> i32 {
        #![inline(always)]
        unsafe { self.as_extern().gc(what, data) }
    }

    /// Raises an error (using the value at the top of the stack)
    pub fn error(&mut self) -> ! {
        #![inline(always)]
        unsafe { self.as_extern().error() }
    }

    /// Pops a key from the stack, and pushes a key-value pair from the table
    /// at the given index (the "next" pair after the given key). If there are
    /// no more elements in the table, then next() returns false (and pushes
    /// nothing).
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
    /// While traversing a table, do not call tostring() or tobytes() directly
    /// on a key, unless you know that the key is actually a string. Recall
    /// that tostring() changes the value at the given index; this confuses the
    /// next call to next().
    pub fn next(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().next(idx) }
    }

    /// Concatenates the `n` values at the top of the stack, pops them, and
    /// leaves the result at the top.
    /// Errors if n is negative or larger than the stack top.
    pub fn concat(&mut self, n: i32) {
        #![inline(always)]
        unsafe { self.as_extern().concat(n) }
    }

    /// Returns the memory-allocation function of a given state. If `ud` is not
    /// NULL, Lua stores in `*ud` the opaque pointer passed to lua_newstate().
    ///
    /// Note: State::new() always provides NULL as the opaque pointer. It also
    /// provides a default alloc function that behaves identically to the one
    /// used by luaL_newstate().
    pub unsafe fn getallocf(&mut self, ud: *mut *mut libc::c_void) -> Alloc {
        #![inline(always)]
        self.as_extern().getallocf(ud)
    }

    /// Changes the allocator function of a given state to `f` with user data
    /// `ud`.
    pub unsafe fn setallocf(&mut self, f: Alloc, ud: *mut libc::c_void) {
        #![inline(always)]
        self.as_extern().setallocf(f, ud)
    }

    /// Pop n elements from the stack.
    /// Errors if the stack is smaller than n
    pub fn pop(&mut self, n: i32) {
        #![inline(always)]
        unsafe { self.as_extern().pop(n) }
    }

    /// Creates a new empty table and pushes it onto the stack.
    /// It is equivalent to .createtable(0, 0).
    pub fn newtable(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().newtable() }
    }

    /// Sets the C function `f` as the new value of global `name`.
    /// Fails  the task if `name` has interior NULs.
    pub fn register(&mut self, name: &str, f: CFunction) {
        #![inline(always)]
        unsafe { self.as_extern().register(name, f) }
    }

    /// Pushes a C function onto the stack.
    pub fn pushcfunction(&mut self, f: CFunction) {
        #![inline(always)]
        unsafe { self.as_extern().pushcfunction(f) }
    }

    /// Returns `true` if the value at the given acceptable index is a function
    /// (either C or Lua).
    pub fn isfunction(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isfunction(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a table.
    pub fn istable(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().istable(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a light
    /// userdata.
    pub fn islightuserdata(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().islightuserdata(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is `nil`.
    pub fn isnil(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isnil(idx) }
    }

    /// Returns `true` if the value at the given acceptable index has type
    /// boolean.
    pub fn isboolean(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isboolean(idx) }
    }

    /// Returns `true` if the value at the given acceptable index is a thread.
    pub fn isthread(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isthread(idx) }
    }

    /// Returns `true` if the given acceptable index is not valid.
    pub fn isnone(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isnone(idx) }
    }

    /// Returns `true` if the given acceptable index is not valid or if the
    /// value at this index is nil.
    pub fn isnoneornil(&mut self, idx: i32) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().isnoneornil(idx) }
    }

    /// Pops a value from the stack and sets it as the new value of global
    /// `name`. Fails the task if `name` has interior NULs.
    pub fn setglobal(&mut self, name: &str) {
        #![inline(always)]
        unsafe { self.as_extern().setglobal(name) }
    }

    /// Pushes onto the stack the value of the global `name`.
    /// Fails the task if `name` has interior NULs.
    pub fn getglobal(&mut self, name: &str) {
        #![inline(always)]
        unsafe { self.as_extern().getglobal(name) }
    }
}

#[allow(missing_docs)]
impl<'l> ExternState<'l> {
    pub unsafe fn newthread(&mut self) -> State {
        self.as_raw().newthread()
    }

    pub unsafe fn atpanic(&mut self, panicf: CFunction) -> CFunction {
        self.as_raw().atpanic(panicf)
    }

    unsafe fn check_acceptable(&mut self, idx: i32) {
        if idx > 0 {
            luaassert!(self, idx <= self.stackspace,
                       "index {} is not acceptable (stack space is {})", idx, self.stackspace);
        } else if idx < 0 {
            self.check_valid(idx, true);
        } else {
            self.errorstr("index 0 is not acceptable");
        }
    }

    unsafe fn check_valid(&mut self, idx: i32, allowpseudo: bool) {
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

    pub unsafe fn describe(&mut self, idx: i32) -> String {
        self.check_acceptable(idx);
        self.checkstack_(1);
        self.as_raw().describe(idx)
    }

    pub unsafe fn describe_(&mut self, idx: i32, usestack: bool) -> String {
        self.check_acceptable(idx);
        if usestack { self.checkstack_(1); }
        self.as_raw().describe_(idx, usestack)
    }

    pub fn gettop(&mut self) -> i32 {
        self.as_raw().gettop()
    }

    pub unsafe fn settop(&mut self, idx: i32) {
        if idx != 0 { self.check_acceptable(idx); }
        self.as_raw().settop(idx);
    }

    pub unsafe fn pushvalue(&mut self, idx: i32) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        self.as_raw().pushvalue(idx)
    }

    pub unsafe fn remove(&mut self, idx: i32) {
        self.check_valid(idx, false);
        self.as_raw().remove(idx)
    }

    pub unsafe fn insert(&mut self, idx: i32) {
        self.check_valid(idx, false);
        self.as_raw().insert(idx)
    }

    pub unsafe fn replace(&mut self, idx: i32) {
        self.check_valid(idx, true);
        self.as_raw().replace(idx)
    }

    pub unsafe fn checkstack(&mut self, extra: i32) -> bool {
        self.as_raw().checkstack(extra)
    }

    pub unsafe fn checkstack_(&mut self, extra: i32) {
        self.as_raw().checkstack_(extra)
    }

    pub unsafe fn xmove(&mut self, to: &mut ExternState, n: i32) {
        luaassert!(self, self.gettop() >= n, "xmove: stack underflow");
        to.checkstack_(1);
        self.as_raw().xmove(to.as_raw(), n)
    }

    pub unsafe fn isnumber(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isnumber(idx)
    }

    pub unsafe fn isstring(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isstring(idx)
    }

    pub unsafe fn iscfunction(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().iscfunction(idx)
    }

    pub unsafe fn isuserdata(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isuserdata(idx)
    }

    pub unsafe fn type_(&mut self, idx: i32) -> Option<Type> {
        self.check_acceptable(idx);
        self.as_raw().type_(idx)
    }

    pub unsafe fn typename(&mut self, idx: i32) -> &'static str {
        self.check_acceptable(idx);
        self.as_raw().typename(idx)
    }

    pub unsafe fn equal(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        self.as_raw().equal(index1, index2)
    }

    pub unsafe fn rawequal(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        self.as_raw().rawequal(index1, index2)
    }

    pub unsafe fn lessthan(&mut self, index1: i32, index2: i32) -> bool {
        self.check_acceptable(index1);
        self.check_acceptable(index2);
        self.as_raw().lessthan(index1, index2)
    }

    pub unsafe fn tonumber(&mut self, idx: i32) -> f64 {
        self.check_acceptable(idx);
        self.as_raw().tonumber(idx)
    }

    pub unsafe fn tointeger(&mut self, idx: i32) -> isize {
        self.check_acceptable(idx);
        self.as_raw().tointeger(idx)
    }

    pub unsafe fn toboolean(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().toboolean(idx)
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// ExternState, but its lifetime is actually that of the value on the
    /// stack.
    pub unsafe fn tostring(&mut self, idx: i32) -> Option<&'static str> {
        self.check_acceptable(idx);
        self.as_raw().tostring(idx)
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// ExternState, but its lifetime is actually that of hte value on the
    /// stack.
    pub unsafe fn tobytes(&mut self, idx: i32) -> Option<&'static [u8]> {
        self.check_acceptable(idx);
        self.as_raw().tobytes(idx)
    }

    pub unsafe fn objlen(&mut self, idx: i32) -> usize {
        self.check_acceptable(idx);
        self.as_raw().objlen(idx)
    }

    pub unsafe fn tocfunction(&mut self, idx: i32) -> Option<CFunction> {
        self.check_acceptable(idx);
        self.as_raw().tocfunction(idx)
    }

    pub unsafe fn touserdata(&mut self, idx: i32) -> *mut libc::c_void {
        self.check_acceptable(idx);
        self.as_raw().touserdata(idx)
    }

    pub unsafe fn tothread(&mut self, idx: i32) -> Option<ExternState> {
        self.check_acceptable(idx);
        self.as_raw().tothread(idx)
    }

    pub unsafe fn topointer(&mut self, idx: i32) -> *const libc::c_void {
        self.check_acceptable(idx);
        self.as_raw().topointer(idx)
    }

    pub unsafe fn pushnil(&mut self) {
        self.checkstack_(1);
        self.as_raw().pushnil()
    }

    pub unsafe fn pushnumber(&mut self, n: f64) {
        self.checkstack_(1);
        self.as_raw().pushnumber(n)
    }

    pub unsafe fn pushinteger(&mut self, n: isize) {
        self.checkstack_(1);
        self.as_raw().pushinteger(n)
    }

    pub unsafe fn pushstring(&mut self, s: &str) {
        self.checkstack_(1);
        self.as_raw().pushstring(s)
    }

    pub unsafe fn pushbytes(&mut self, bytes: &[u8]) {
        self.checkstack_(1);
        self.as_raw().pushbytes(bytes)
    }

    pub unsafe fn pushcclosure(&mut self, f: CFunction, n: i32) {
        if n == 0 {
            self.checkstack_(1);
        } else {
            luaassert!(self, n >= 0 && n <= 255, "pushcclosure: invalid argument n");
        }
        self.as_raw().pushcclosure(f, n)
    }

    pub unsafe fn pushboolean(&mut self, b: bool) {
        self.checkstack_(1);
        self.as_raw().pushboolean(b)
    }

    pub unsafe fn pushlightuserdata(&mut self, p: *mut libc::c_void) {
        self.checkstack_(1);
        self.as_raw().pushlightuserdata(p)
    }

    pub unsafe fn pushthread(&mut self) -> bool {
        self.checkstack_(1);
        self.as_raw().pushthread()
    }

    pub unsafe fn gettable(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "gettable: stack underflow");
        self.as_raw().gettable(idx)
    }

    pub unsafe fn getfield(&mut self, idx: i32, k: &str) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        self.as_raw().getfield(idx, k)
    }

    pub unsafe fn rawget(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() > 0, "rawget: stack underflow");
        self.as_raw().rawget(idx)
    }

    pub unsafe fn rawgeti(&mut self, idx: i32, n: i32) {
        self.check_valid(idx, true);
        self.checkstack_(1);
        self.as_raw().rawgeti(idx, n)
    }

    pub unsafe fn createtable(&mut self, narr: i32, nrec: i32) {
        self.checkstack_(1);
        self.as_raw().createtable(narr, nrec)
    }

    pub unsafe fn newuserdata(&mut self, size: usize) -> *mut libc::c_void {
        self.checkstack_(1);
        self.as_raw().newuserdata(size)
    }

    pub unsafe fn getmetatable(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.checkstack_(1);
        self.as_raw().getmetatable(idx)
    }

    pub unsafe fn getfenv(&mut self, idx: i32) {
        self.check_acceptable(idx);
        self.checkstack_(1);
        self.as_raw().getfenv(idx)
    }

    pub unsafe fn settable(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 2, "settable: stack underflow");
        self.as_raw().settable(idx)
    }

    pub unsafe fn setfield(&mut self, idx: i32, k: &str) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 1, "setfield: stack underflow");
        self.as_raw().setfield(idx, k)
    }

    pub unsafe fn rawset(&mut self, idx: i32) {
        self.check_valid(idx, true);
        luaassert!(self, self.gettop() >= 2, "rawset: stack underflow");
        self.as_raw().rawset(idx)
    }

    pub unsafe fn rawseti(&mut self, idx: i32, n: i32) {
        self.check_valid(idx, true);
        self.as_raw().rawseti(idx, n)
    }

    pub unsafe fn setmetatable(&mut self, idx: i32) {
        self.check_acceptable(idx);
        luaassert!(self, self.istable(-1), "setmetatable: top stack value must be a table");
        self.as_raw().setmetatable(idx)
    }

    pub unsafe fn setfenv(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        luaassert!(self, self.istable(-1), "setfenv: top stack value must be a table");
        self.as_raw().setfenv(idx)
    }

    pub unsafe fn call(&mut self, nargs: i32, nresults: i32) {
        luaassert!(self, nargs >= 0, "call: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "call: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "call: stack underflow");
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        self.as_raw().call(nargs, nresults)
    }

    pub unsafe fn pcall(&mut self, nargs: i32, nresults: i32, errfunc: i32)
                       -> Result<(),PCallError> {
        luaassert!(self, nargs >= 0, "pcall: invalid nargs");
        luaassert!(self, nresults == MULTRET || nresults >= 0, "pcall: invalid nresults");
        luaassert!(self, self.gettop() > nargs, "pcall: stack underflow");
        if errfunc != 0 {
            self.check_valid(errfunc, false)
        }
        if nresults > nargs + 1 { self.checkstack_(nargs - nresults - 1) }
        self.as_raw().pcall(nargs, nresults, errfunc)
    }

    pub unsafe fn load(&mut self, reader: Reader, data: *mut libc::c_void, chunkname: &str)
                      -> Result<(),LoadError> {
        self.checkstack_(1);
        self.as_raw().load(reader, data, chunkname)
    }

    pub unsafe fn dump(&mut self, writer: Writer, data: *mut libc::c_void) -> Result<(),i32> {
        luaassert!(self, self.gettop() >= 1, "dump: stack underflow");
        self.as_raw().dump(writer, data)
    }

    pub unsafe fn yield_(&mut self, nresults: i32) -> c_int {
        luaassert!(self, self.gettop() >= nresults, "yield: stack underflow");
        self.as_raw().yield_(nresults)
    }

    pub unsafe fn resume(&mut self, narg: i32) -> Result<bool,PCallError> {
        luaassert!(self, self.gettop() > narg, "resume: stack underflow");
        self.as_raw().resume(narg)
    }

    pub unsafe fn status(&mut self) -> Result<bool,PCallError> {
        self.as_raw().status()
    }

    pub unsafe fn gc(&mut self, what: GC, data: i32) -> i32 {
        self.as_raw().gc(what, data)
    }

    pub unsafe fn error(&mut self) -> ! {
        luaassert!(self, self.gettop() > 0, "error: stack underflow");
        self.as_raw().error()
    }

    pub unsafe fn next(&mut self, idx: i32) -> bool {
        self.check_valid(idx, true);
        self.as_raw().next(idx)
    }

    pub unsafe fn concat(&mut self, n: i32) {
        luaassert!(self, n >= 0, "concat: invalid argument n");
        luaassert!(self, n <= self.gettop(), "concat: stack underflow");
        if n == 0 { self.checkstack_(1) }
        self.as_raw().concat(n)
    }

    pub unsafe fn getallocf(&mut self, ud: *mut *mut libc::c_void) -> Alloc {
        self.as_raw().getallocf(ud)
    }

    pub unsafe fn setallocf(&mut self, f: Alloc, ud: *mut libc::c_void) {
        self.as_raw().setallocf(f, ud)
    }

    pub unsafe fn pop(&mut self, n: i32) {
        if n >= 0 {
            luaassert!(self, self.gettop() >= n, "pop: stack underflow");
        } else {
            luaassert!(self, self.gettop() >= (n+1).abs(), "pop: stack underflow");
        }
        self.as_raw().pop(n)
    }

    pub unsafe fn newtable(&mut self) {
        self.checkstack_(1);
        self.as_raw().newtable()
    }

    pub unsafe fn register(&mut self, name: &str, f: CFunction) {
        self.checkstack_(1);
        self.as_raw().register(name, f)
    }

    pub unsafe fn pushcfunction(&mut self, f: CFunction) {
        self.checkstack_(1);
        self.as_raw().pushcfunction(f)
    }

    pub unsafe fn isfunction(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isfunction(idx)
    }

    pub unsafe fn istable(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().istable(idx)
    }

    pub unsafe fn islightuserdata(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().islightuserdata(idx)
    }

    pub unsafe fn isnil(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isnil(idx)
    }

    pub unsafe fn isboolean(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isboolean(idx)
    }

    pub unsafe fn isthread(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isthread(idx)
    }

    pub unsafe fn isnone(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isnone(idx)
    }

    pub unsafe fn isnoneornil(&mut self, idx: i32) -> bool {
        self.check_acceptable(idx);
        self.as_raw().isnoneornil(idx)
    }

    pub unsafe fn setglobal(&mut self, name: &str) {
        luaassert!(self, self.gettop() > 0, "setglobal: stack underflow");
        self.as_raw().setglobal(name)
    }

    pub unsafe fn getglobal(&mut self, name: &str) {
        self.checkstack_(1);
        self.as_raw().getglobal(name)
    }
}

#[allow(missing_docs)]
impl<'l> RawState<'l> {
    pub unsafe fn newthread(&mut self) -> State {
        #![inline]
        mem::transmute(ExternState::from_lua_State(raw::lua_newthread(self.L)))
    }

    pub unsafe fn atpanic(&mut self, panicf: CFunction) -> CFunction {
        #![inline]
        raw::lua_atpanic(self.L, panicf)
    }

    pub unsafe fn describe(&mut self, idx: i32) -> String {
        self.describe_(idx, true)
    }

    pub unsafe fn describe_(&mut self, idx: i32, usestack: bool) -> String {
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

    pub fn gettop(&mut self) -> i32 {
        #![inline]
        unsafe { raw::lua_gettop(self.L) as i32 }
    }

    pub unsafe fn settop(&mut self, idx: i32) {
        #![inline]
        raw::lua_settop(self.L, idx as c_int)
    }

    pub unsafe fn pushvalue(&mut self, idx: i32) {
        #![inline]
        raw::lua_pushvalue(self.L, idx as c_int)
    }

    pub unsafe fn remove(&mut self, idx: i32) {
        #![inline]
        raw::lua_remove(self.L, idx as c_int)
    }

    pub unsafe fn insert(&mut self, idx: i32) {
        #![inline]
        raw::lua_insert(self.L, idx as c_int)
    }

    pub unsafe fn replace(&mut self, idx: i32) {
        #![inline]
        raw::lua_replace(self.L, idx as c_int)
    }

    pub unsafe fn checkstack(&mut self, extra: i32) -> bool {
        #![inline]
        let top = self.gettop();
        if top + extra > self.stackspace {
            if raw::lua_checkstack(self.L, extra as c_int) != 0 {
                self.stackspace = top + extra;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub unsafe fn checkstack_(&mut self, extra: i32) {
        #![inline]
        luaassert!(self, self.checkstack(extra), "checkstack: cannot grow stack")
    }

    pub unsafe fn xmove(&mut self, to: &mut RawState, n: i32) {
        #![inline]
        raw::lua_xmove(self.L, to.L, n as c_int)
    }

    pub unsafe fn isnumber(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isnumber(self.L, idx as c_int) != 0
    }

    pub unsafe fn isstring(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isstring(self.L, idx as c_int) != 0
    }

    pub unsafe fn iscfunction(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_iscfunction(self.L, idx as c_int) != 0
    }

    pub unsafe fn isuserdata(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isuserdata(self.L, idx as c_int) != 0
    }

    pub unsafe fn type_(&mut self, idx: i32) -> Option<Type> {
        #![inline]
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

    pub unsafe fn typename(&mut self, idx: i32) -> &'static str {
        #![inline]
        let s = aux::raw::luaL_typename(self.L, idx as c_int);
        mem::transmute::<&str, &'static str>(str::from_utf8(ffi::c_str_to_bytes(&s)).unwrap())
    }

    pub unsafe fn equal(&mut self, index1: i32, index2: i32) -> bool {
        #![inline]
        raw::lua_equal(self.L, index1 as c_int, index2 as c_int) != 0
    }

    pub unsafe fn rawequal(&mut self, index1: i32, index2: i32) -> bool {
        #![inline]
        raw::lua_rawequal(self.L, index1 as c_int, index2 as c_int) != 0
    }

    pub unsafe fn lessthan(&mut self, index1: i32, index2: i32) -> bool {
        #![inline]
        raw::lua_lessthan(self.L, index1 as c_int, index2 as c_int) != 0
    }

    pub unsafe fn tonumber(&mut self, idx: i32) -> f64 {
        #![inline]
        raw::lua_tonumber(self.L, idx as c_int) as f64
    }

    pub unsafe fn tointeger(&mut self, idx: i32) -> isize {
        #![inline]
        raw::lua_tointeger(self.L, idx as c_int) as isize
    }

    pub unsafe fn toboolean(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_toboolean(self.L, idx as c_int) != 0
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    // TODO: change return type to use core::str::Utf8Error
    pub unsafe fn tostring(&mut self, idx: i32) -> Option<&'static str> {
        #![inline]
        self.tobytes(idx).and_then(|v| str::from_utf8(v).ok())
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    pub unsafe fn tobytes(&mut self, idx: i32) -> Option<&'static [u8]> {
        #![inline]
        let mut sz: libc::size_t = 0;
        let s = raw::lua_tolstring(self.L, idx, &mut sz);
        if s.is_null() {
            None
        } else {
            let buf = s as *const u8;
            Some(mem::transmute::<&[u8], &'static [u8]>(slice::from_raw_buf(&buf, sz as usize)))
        }
    }

    pub unsafe fn objlen(&mut self, idx: i32) -> usize {
        #![inline]
        raw::lua_objlen(self.L, idx as c_int) as usize
    }

    pub unsafe fn tocfunction(&mut self, idx: i32) -> Option<CFunction> {
        #![inline]
        raw::lua_tocfunction(self.L, idx as c_int)
    }

    pub unsafe fn touserdata(&mut self, idx: i32) -> *mut libc::c_void {
        #![inline]
        raw::lua_touserdata(self.L, idx as c_int)
    }

    pub unsafe fn tothread(&mut self, idx: i32) -> Option<ExternState> {
        #![inline]
        let s = raw::lua_tothread(self.L, idx as c_int);
        if s.is_null() {
            None
        } else {
            Some(ExternState { L: s, stackspace: 0 })
        }
    }

    pub unsafe fn topointer(&mut self, idx: i32) -> *const libc::c_void {
        #![inline]
        raw::lua_topointer(self.L, idx as c_int)
    }

    pub unsafe fn pushnil(&mut self) {
        #![inline]
        raw::lua_pushnil(self.L)
    }

    pub unsafe fn pushnumber(&mut self, n: f64) {
        #![inline]
        raw::lua_pushnumber(self.L, n as raw::lua_Number)
    }

    pub unsafe fn pushinteger(&mut self, n: isize) {
        #![inline]
        raw::lua_pushinteger(self.L, n as raw::lua_Integer)
    }

    pub unsafe fn pushstring(&mut self, s: &str) {
        #![inline]
        raw::lua_pushlstring(self.L, s.as_ptr() as *const libc::c_char, s.len() as libc::size_t)
    }

    pub unsafe fn pushbytes(&mut self, bytes: &[u8]) {
        #![inline]
        raw::lua_pushlstring(self.L, bytes.as_ptr() as *const libc::c_char, bytes.len() as libc::size_t)
    }

    pub unsafe fn pushcclosure(&mut self, f: CFunction, n: i32) {
        #![inline]
        raw::lua_pushcclosure(self.L, f, n as c_int)
    }

    pub unsafe fn pushboolean(&mut self, b: bool) {
        #![inline]
        raw::lua_pushboolean(self.L, b as c_int)
    }

    pub unsafe fn pushlightuserdata(&mut self, p: *mut libc::c_void) {
        #![inline]
        raw::lua_pushlightuserdata(self.L, p)
    }

    pub unsafe fn pushthread(&mut self) -> bool {
        #![inline]
        raw::lua_pushthread(self.L) != 0
    }

    pub unsafe fn gettable(&mut self, idx: i32) {
        #![inline]
        raw::lua_gettable(self.L, idx as c_int)
    }

    pub unsafe fn getfield(&mut self, idx: i32, k: &str) {
        #![inline]
        raw::lua_getfield(self.L, idx as c_int, CString::from_slice(k.as_bytes()).as_ptr())
    }

    pub unsafe fn rawget(&mut self, idx: i32) {
        #![inline]
        raw::lua_rawget(self.L, idx as c_int)
    }

    pub unsafe fn rawgeti(&mut self, idx: i32, n: i32) {
        #![inline]
        raw::lua_rawgeti(self.L, idx as c_int, n as c_int)
    }

    pub unsafe fn createtable(&mut self, narr: i32, nrec: i32) {
        #![inline]
        raw::lua_createtable(self.L, narr as c_int, nrec as c_int)
    }

    pub unsafe fn newuserdata(&mut self, size: usize) -> *mut libc::c_void {
        #![inline]
        raw::lua_newuserdata(self.L, size as libc::size_t)
    }

    pub unsafe fn getmetatable(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_getmetatable(self.L, idx as c_int) != 0
    }

    pub unsafe fn getfenv(&mut self, idx: i32) {
        #![inline]
        raw::lua_getfenv(self.L, idx as c_int)
    }

    pub unsafe fn settable(&mut self, idx: i32) {
        #![inline]
        raw::lua_settable(self.L, idx as c_int)
    }

    pub unsafe fn setfield(&mut self, idx: i32, k: &str) {
        #![inline]
        raw::lua_setfield(self.L, idx as c_int, CString::from_slice(k.as_bytes()).as_ptr())
    }

    pub unsafe fn rawset(&mut self, idx: i32) {
        #![inline]
        raw::lua_rawset(self.L, idx as c_int)
    }

    pub unsafe fn rawseti(&mut self, idx: i32, n: i32) {
        #![inline]
        raw::lua_rawseti(self.L, idx as c_int, n as c_int)
    }

    pub unsafe fn setmetatable(&mut self, idx: i32) {
        #![inline]
        // ignore return value of lua_setmetatable(), it appears to always be 1
        raw::lua_setmetatable(self.L, idx as c_int);
    }

    pub unsafe fn setfenv(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_setfenv(self.L, idx as c_int) != 0
    }

    pub unsafe fn call(&mut self, nargs: i32, nresults: i32) {
        #![inline]
        raw::lua_call(self.L, nargs as c_int, nresults as c_int)
    }

    pub unsafe fn pcall(&mut self, nargs: i32, nresults: i32, errfunc: i32)
                       -> Result<(),PCallError> {
        match raw::lua_pcall(self.L, nargs as c_int, nresults as c_int, errfunc as c_int) {
            0 => Ok(()),
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("pcall: unexpected error from lua_pcall")
            }))
        }
    }

    pub unsafe fn load(&mut self, reader: Reader, data: *mut libc::c_void, chunkname: &str)
                      -> Result<(),LoadError> {
        let cstr = CString::from_slice(chunkname.as_bytes());
        match raw::lua_load(self.L, reader, data, cstr.as_ptr()) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("load: unexpected error from lua_load")
        }
    }

    pub unsafe fn dump(&mut self, writer: Writer, data: *mut libc::c_void) -> Result<(),i32> {
        #![inline]
        match raw::lua_dump(self.L, writer, data) {
            0 => Ok(()),
            i => Err(i)
        }
    }

    pub unsafe fn yield_(&mut self, nresults: i32) -> c_int {
        #![inline]
        raw::lua_yield(self.L, nresults as c_int)
    }

    pub unsafe fn resume(&mut self, narg: i32) -> Result<bool,PCallError> {
        #![inline]
        match raw::lua_resume(self.L, narg as c_int) {
            raw::LUA_YIELD => Ok(false),
            0 => Ok(true),
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("resume: unexpected error from lua_resume")
            }))
        }
    }

    pub unsafe fn status(&mut self) -> Result<bool,PCallError> {
        #![inline]
        match raw::lua_status(self.L) {
            raw::LUA_YIELD => Ok(false),
            0 => Ok(true),
            i => Err(PCallError::from_code(i).unwrap_or_else(|| {
                self.errorstr("status: unexpected error from lua_status")
            }))
        }
    }

    pub unsafe fn gc(&mut self, what: GC, data: i32) -> i32 {
        #![inline]
        raw::lua_gc(self.L, what as c_int, data as c_int) as i32
    }

    pub unsafe fn error(&mut self) -> ! {
        #![inline]
        raw::lua_error(self.L);
        unreachable!()
    }

    pub unsafe fn next(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_next(self.L, idx as c_int) != 0
    }

    pub unsafe fn concat(&mut self, n: i32) {
        #![inline]
        raw::lua_concat(self.L, n as c_int)
    }

    pub unsafe fn getallocf(&mut self, ud: *mut *mut libc::c_void) -> Alloc {
        #![inline]
        raw::lua_getallocf(self.L, ud)
    }

    pub unsafe fn setallocf(&mut self, f: Alloc, ud: *mut libc::c_void) {
        #![inline]
        raw::lua_setallocf(self.L, f, ud)
    }

    pub unsafe fn pop(&mut self, n: i32) {
        #![inline]
        raw::lua_pop(self.L, n as c_int)
    }

    pub unsafe fn newtable(&mut self) {
        #![inline]
        raw::lua_newtable(self.L)
    }

    pub unsafe fn register(&mut self, name: &str, f: CFunction) {
        #![inline]
        raw::lua_register(self.L, CString::from_slice(name.as_bytes()).as_ptr(), f)
    }

    pub unsafe fn pushcfunction(&mut self, f: CFunction) {
        #![inline]
        raw::lua_pushcfunction(self.L, f)
    }

    pub unsafe fn isfunction(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isfunction(self.L, idx as c_int)
    }

    pub unsafe fn istable(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_istable(self.L, idx as c_int)
    }

    pub unsafe fn islightuserdata(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_islightuserdata(self.L, idx)
    }

    pub unsafe fn isnil(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isnil(self.L, idx)
    }

    pub unsafe fn isboolean(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isboolean(self.L, idx)
    }

    pub unsafe fn isthread(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isthread(self.L, idx)
    }

    pub unsafe fn isnone(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isnone(self.L, idx)
    }

    pub unsafe fn isnoneornil(&mut self, idx: i32) -> bool {
        #![inline]
        raw::lua_isnoneornil(self.L, idx)
    }

    pub unsafe fn setglobal(&mut self, name: &str) {
        #![inline]
        raw::lua_setglobal(self.L, CString::from_slice(name.as_bytes()).as_ptr())
    }

    pub unsafe fn getglobal(&mut self, name: &str) {
        #![inline]
        raw::lua_getglobal(self.L, CString::from_slice(name.as_bytes()).as_ptr())
    }
}

/// Name for the coroutine lib
pub const COLIBNAME: &'static str = lib::raw::LUA_COLIBNAME;
/// Name for the table lib
pub const TABLIBNAME: &'static str = lib::raw::LUA_TABLIBNAME;
/// Name for the io lib
pub const IOLIBNAME: &'static str = lib::raw::LUA_IOLIBNAME;
/// Name for the os lib
pub const OSLIBNAME: &'static str = lib::raw::LUA_OSLIBNAME;
/// Name for the string lib
pub const STRLIBNAME: &'static str = lib::raw::LUA_STRLIBNAME;
/// Name for the math lib
pub const MATHLIBNAME: &'static str = lib::raw::LUA_MATHLIBNAME;
/// Name for the debug lib
pub const DBLIBNAME: &'static str = lib::raw::LUA_DBLIBNAME;
/// Name for the package lib
pub const LOADLIBNAME: &'static str = lib::raw::LUA_LOADLIBNAME;

// Functions from lualib
impl State {
    /// Open the basic library.
    pub fn open_base(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_base() }
    }

    /// Opens the table library.
    pub fn open_table(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_table() }
    }

    /// Opens the io library.
    pub fn open_io(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_io() }
    }

    /// Opens the os library.
    pub fn open_os(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_os() }
    }

    /// Opens the string library.
    pub fn open_string(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_string() }
    }

    /// Opens the math library.
    pub fn open_math(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_math() }
    }

    /// Opens the debug library.
    pub fn open_debug(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_debug() }
    }

    /// Opens the package library.
    pub fn open_package(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().open_package() }
    }

    /// Opens all standard Lua libraries.
    pub fn openlibs(&mut self) {
        #![inline(always)]
        unsafe { self.as_extern().openlibs() }
    }
}

#[allow(missing_docs)]
impl<'l> ExternState<'l> {
    pub unsafe fn open_base(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_base()
    }

    pub unsafe fn open_table(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_table()
    }

    pub unsafe fn open_io(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_io()
    }

    pub unsafe fn open_os(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_os()
    }

    pub unsafe fn open_string(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_string()
    }

    pub unsafe fn open_math(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_math()
    }

    pub unsafe fn open_debug(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_debug()
    }

    pub unsafe fn open_package(&mut self) {
        self.checkstack_(2);
        self.as_raw().open_package()
    }

    pub unsafe fn openlibs(&mut self) {
        self.checkstack_(2);
        self.as_raw().openlibs()
    }
}

#[allow(missing_docs)]
impl<'l> RawState<'l> {
    pub unsafe fn open_base(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_base);
        self.pushstring("");
        self.call(1, 0);
    }

    pub unsafe fn open_table(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_table);
        self.pushstring(TABLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_io(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_io);
        self.pushstring(IOLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_os(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_os);
        self.pushstring(OSLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_string(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_string);
        self.pushstring(STRLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_math(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_math);
        self.pushstring(MATHLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_debug(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_debug);
        self.pushstring(DBLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn open_package(&mut self) {
        #![inline]
        self.pushcfunction(lib::raw::luaopen_package);
        self.pushstring(LOADLIBNAME);
        self.call(1, 0);
    }

    pub unsafe fn openlibs(&mut self) {
        #![inline]
        lib::raw::luaL_openlibs(self.L)
    }
}

pub const NOREF: i32 = aux::raw::LUA_NOREF as i32;
pub const REFNIL: i32 = aux::raw::LUA_REFNIL as i32;

// Functions from auxlib
impl State {
    /// Opens a library.
    ///
    /// When called with `libname` equal to None, it simply registers all
    /// functions in the list `l` into the table on the top of the stack.
    ///
    /// When called with a `libname` of Some(_), registerlib() creates a new
    /// table `t`, sets it as the value of the global variable `libname`, sets
    /// it as the value of `package.loaded[libname]`, and registers on it all
    /// functions in the list `l`. If there is a table in
    /// `package.loaded[libname]` or in variable `libname`, reuses this table
    /// instead of creating a new one.
    ///
    /// In any case the function leaves the table on the top of the stack.
    pub fn registerlib(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        #![inline(always)]
        unsafe { self.as_extern().registerlib(libname, l) }
    }

    /// Pushes onto the stack the field `e` from the metatable of the object at
    /// index `obj`. If the object does not have a metatable, or if the
    /// metatable does not have this field, returns `false` and pushes nothing.
    pub fn getmetafield(&mut self, obj: i32, e: &str) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().getmetafield(obj, e) }
    }

    /// Calls a metamethod.
    ///
    /// If the object at index `obj` has a metatable and this metatable has a
    /// field `e`, this method calls this field and passes the object as its
    /// only argument. In this case this method returns `true` and pushes onto
    /// the stack the value returned by the call. If there is no metatable or
    /// no metamethod, this method returns `false` (without pushing any value
    /// on the stack).
    pub fn callmeta(&mut self, obj: i32, e: &str) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().callmeta(obj, e) }
    }

    /// Generates an error with a message like the following:
    ///
    ///   <location>: bad argument <narg> to '<func>' (<tname> expected, got <rt>)
    ///
    /// where `location` is produced by where(), `func` is the name of the
    /// current function, and `rt` is the type name of the actual argument.
    pub fn typerror(&mut self, narg: i32, tname: &str) -> ! {
        #![inline(always)]
        unsafe { self.as_extern().typerror(narg, tname) }
    }

    /// Raises an error with the following message, where `func` is taken from
    /// the call stack:
    ///
    ///   bad argument #<narg> to <func> (<extramsg>)
    pub fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        #![inline(always)]
        unsafe { self.as_extern().argerror(narg, extramsg) }
    }

    /// Checks whether the function argument `narg` is a string, and returns
    /// the string.  This function uses lua_tolstring to get its result, so all
    /// conversions and caveats of that function apply here.
    ///
    /// If the string is not utf-8, returns None.
    pub fn checkstring<'a>(&'a mut self, narg: i32) -> Option<&'a str> {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().checkstring(narg)) }
    }

    /// Checks whether the function argument `narg` is a lua string, and
    /// returns it as a byte vector. See checkstring() for caveats.
    pub fn checkbytes<'a>(&'a mut self, narg: i32) -> &'a [u8] {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().checkbytes(narg)) }
    }

    /// If the function argument `narg` is a string, returns this string. If
    /// this argument is absent or is nil, returns `d`. Otherwise, raises an
    /// error.
    ///
    /// If the argument is a string, but is not utf-8, returns None.
    pub fn optstring<'a>(&'a mut self, narg: i32, d: &'a str) -> Option<&'a str> {
        #![inline(always)]
        unsafe {
            let d = mem::transmute::<&'a str, &'static str>(d);
            mem::transmute(self.as_extern().optstring(narg, d))
        }
    }

    /// If the function argument `narg` is a lua string, returns this string
    /// asa byte vector.  See optstring() for more information.
    pub fn optbytes<'a>(&'a mut self, narg: i32, d: &'a [u8]) -> &'a [u8] {
        #![inline(always)]
        unsafe {
            let d = mem::transmute::<&'a [u8], &'static [u8]>(d);
            mem::transmute(self.as_extern().optbytes(narg, d))
        }
    }

    /// Checks whether the function argument `narg` is a number and returns the
    /// number.
    pub fn checknumber(&mut self, narg: i32) -> f64 {
        #![inline(always)]
        unsafe { self.as_extern().checknumber(narg) }
    }

    /// If the function argument `narg` is a number, returns this number. If
    /// the argument is absent or is nil, returns `d`. Otherwise, throws an
    /// error.
    pub fn optnumber(&mut self, narg: i32, d: f64) -> f64 {
        #![inline(always)]
        unsafe { self.as_extern().optnumber(narg, d) }
    }

    /// Checks whether the function argument `narg` is a number and returns it
    /// as an isize.
    pub fn checkinteger(&mut self, narg: i32) -> isize {
        #![inline(always)]
        unsafe { self.as_extern().checkinteger(narg) }
    }

    /// If the function argument `narg` is a number, returns this number cast
    /// to an isize. If this argument is absent or nil, returns `d`. Otherwise,
    /// raises an error.
    pub fn optinteger(&mut self, narg: i32, d: isize) -> isize {
        #![inline(always)]
        unsafe { self.as_extern().optinteger(narg, d) }
    }

    /// Checks whether the function argument `narg` has type `t`.
    pub fn checktype(&mut self, narg: i32, t: Type) {
        #![inline(always)]
        unsafe { self.as_extern().checktype(narg, t) }
    }

    /// Checks whether the function has an argument of any type (including nil)
    /// at position `narg`.
    pub fn checkany(&mut self, narg: i32) {
        #![inline(always)]
        unsafe { self.as_extern().checkany(narg) }
    }

    /// If the registry already has the key `tname`, returns `false`.
    /// Otherwise, creates a new table to be used as a metatable for userdata,
    /// adds it to the registry with key `tname`, and returns `true`.
    ///
    /// In both cases pushes onto the stack the final value associated with
    /// `tname` in the registry.
    pub fn newmetatable(&mut self, tname: &str) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().newmetatable(tname) }
    }

    /// Checks whether the function argument `narg` is a userdata of the type
    /// `tname` (see newmetatable()). The userdata pointer is returned.
    pub fn checkudata(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        #![inline(always)]
        unsafe { self.as_extern().checkudata(narg, tname) }
    }

    /// Pushes onto the stack a string identifying the current position of the
    /// control at level `lvl` in the call stack.
    /// Level 0 is the running function, level 1 is the function that called
    /// the running function, etc.
    pub fn where_(&mut self, lvl: i32) {
        #![inline(always)]
        unsafe { self.as_extern().where_(lvl) }
    }

    /// Raises an error with the given string.
    /// It also adds at the beginning of the message the file name and line
    /// number where the error occurred, if this information is available.
    pub fn errorstr(&mut self, s: &str) -> ! {
        #![inline(always)]
        unsafe { self.as_extern().errorstr(s) }
    }

    /// Checks whether the function arg `narg` is a string and searches for
    /// this string in `lst`.  The first element of each tuple is compared
    /// against, and if a match is found, the second element is returned.
    /// Raises an error if the argument is not a string or the string cannot be
    /// found.
    ///
    /// If `def` is not None, the function uses `def` as a default value when
    /// there is no argument `narg` or this argument is nil.
    ///
    /// Fails the task if `def` or any list key has interior NULs
    pub fn checkoption<'a, T>(&mut self, narg: i32, def: Option<&str>, lst: &'a [(&str,T)])
                             -> &'a T {
        #![inline(always)]
        unsafe { self.as_extern().checkoption(narg, def, lst) }
    }

    /// Creates and returns a reference, in the table at index `t`, for the
    /// object at the top of the stack (and pops the object).
    ///
    /// A reference is a unique integer key. As long as you do not manually add
    /// integer keys into table `t`, ref_() ensures the uniqueness of the key
    /// it returns. You can retrieve an object referred by reference `r` by
    /// calling `L.rawget(t, r)`. Method unref() frees a reference and its
    /// associated object.
    ///
    /// If the object at the top of the stack is nil, ref_() returns the
    /// constant RefNil. The constant NoRef is guaranteed to be different from
    /// any reference returned by ref_().
    pub fn ref_(&mut self, t: i32) -> i32 {
        #![inline(always)]
        unsafe { self.as_extern().ref_(t) }
    }

    /// Releases reference `r` from the table at index `t` (see ref_()). The
    /// entry is removed from the table, so that the referred object can be
    /// collected. The reference `r` is also freed to be used again.
    ///
    /// If ref is NoRef or RefNil, unref() does nothing.
    pub fn unref(&mut self, t: i32, r: i32) {
        #![inline(always)]
        unsafe { self.as_extern().unref(t, r) }
    }

    /// Loads a file as a Lua chunk (but does not run it).
    /// If the `filename` is None, this loads from standard input.
    /// Fails the task if `filename` has any interior NULs.
    pub fn loadfile(&mut self, filename: Option<&path::Path>) -> Result<(),LoadFileError> {
        #![inline(always)]
        unsafe { self.as_extern().loadfile(filename) }
    }

    /// Loads a buffer as a Lua chunk (but does not run it).
    /// As far as Rust is concerned, this differ from loadstring() in that a
    /// name for the chunk is provided. It also allows for NUL bytes, but I
    /// expect Lua won't like those.
    /// Fails the task if `name` has any interior NULs.
    pub fn loadbuffer(&mut self, buf: &str, name: &str) -> Result<(),LoadError> {
        #![inline(always)]
        unsafe { self.as_extern().loadbuffer(buf, name) }
    }

    /// Loads a string as a Lua chunk (but does not run it).
    /// Fails the task if `s` has any interior NULs.
    pub fn loadstring(&mut self, s: &str) -> Result<(),LoadError> {
        #![inline(always)]
        unsafe { self.as_extern().loadstring(s) }
    }

    /// Creates a copy of string `s` by replacing any occurrence of the string
    /// `p` with the string `r`. Pushes the resulting string on the stack and
    /// returns it.
    pub fn gsub<'a>(&'a mut self, s: &str, p: &str, r: &str) -> &'a str {
        #![inline(always)]
        unsafe { mem::transmute(self.as_extern().gsub(s, p, r)) }
    }

    /// Checks whether `cond` is true. If not, raises an error with the
    /// following message, where `func` is retrieved from the call stack:
    ///
    ///   bad argument #<narg> to <func> (<extramsg>)
    ///
    /// Fails the task if `extramsg` has interior NULs.
    pub fn argcheck(&mut self, cond: bool, narg: i32, extramsg: &str) {
        #![inline(always)]
        unsafe { self.as_extern().argcheck(cond, narg, extramsg) }
    }

    /// Loads and runs the given file. It returns `true` if there are no errors
    /// or `false` in case of errors.
    pub fn dofile(&mut self, filename: Option<&path::Path>) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().dofile(filename) }
    }

    /// Loads and runs the given string. It returns `true` if there are no
    /// errors or `false` in case of errors.
    pub fn dostring(&mut self, s: &str) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().dostring(s) }
    }

    /// Pushes onto the stack the metatable associated with the name `tname` in
    /// the registry (see newmetatable()).
    pub fn getmetatable_reg(&mut self, tname: &str) {
        #![inline(always)]
        unsafe { self.as_extern().getmetatable_reg(tname) }
    }

    /// Initializes and returns a Buffer
    pub fn buffinit<'a>(&'a mut self) -> Buffer<'a> {
        #![inline(always)]
        self.as_extern().buffinit()
    }
}

#[allow(missing_docs)]
impl<'l> ExternState<'l> {
    pub unsafe fn registerlib(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        // internally, luaL_registerlib seems to use 4 stack slots
        self.checkstack_(4);
        if libname.is_none() {
            luaassert!(self, self.gettop() >= 1, "registerlib: stack underflow");
        }
        self.as_raw().registerlib(libname, l)
    }

    pub unsafe fn getmetafield(&mut self, obj: i32, e: &str) -> bool {
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_getmetafield uses 2 stack slots
        self.as_raw().getmetafield(obj, e)
    }

    pub unsafe fn callmeta(&mut self, obj: i32, e: &str) -> bool {
        self.check_acceptable(obj);
        self.checkstack_(2); // internally, luaL_callmeta uses 2 stack slots
        self.as_raw().callmeta(obj, e)
    }

    pub unsafe fn typerror(&mut self, narg: i32, tname: &str) -> ! {
        self.check_acceptable(narg);
        // NB: stack checking is not necessary
        self.as_raw().typerror(narg, tname)
    }

    pub unsafe fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        // NB: stack checking is not necessary
        self.as_raw().argerror(narg, extramsg)
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    pub unsafe fn checkstring(&mut self, narg: i32) -> Option<&'static str> {
        self.check_acceptable(narg);
        self.as_raw().checkstring(narg)
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    pub unsafe fn checkbytes(&mut self, narg: i32) -> &'static [u8] {
        self.check_acceptable(narg);
        self.as_raw().checkbytes(narg)
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    pub unsafe fn optstring(&mut self, narg: i32, d: &'static str) -> Option<&'static str> {
        self.check_acceptable(narg);
        self.as_raw().optstring(narg, d)
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    pub unsafe fn optbytes(&mut self, narg: i32, d: &'static [u8]) -> &'static [u8] {
        self.check_acceptable(narg);
        self.as_raw().optbytes(narg, d)
    }

    pub unsafe fn checknumber(&mut self, narg: i32) -> f64 {
        self.check_acceptable(narg);
        self.as_raw().checknumber(narg)
    }

    pub unsafe fn optnumber(&mut self, narg: i32, d: f64) -> f64 {
        self.check_acceptable(narg);
        self.as_raw().optnumber(narg, d)
    }

    pub unsafe fn checkinteger(&mut self, narg: i32) -> isize {
        self.check_acceptable(narg);
        self.as_raw().checkinteger(narg)
    }

    pub unsafe fn optinteger(&mut self, narg: i32, d: isize) -> isize {
        self.check_acceptable(narg);
        self.as_raw().optinteger(narg, d)
    }

    pub unsafe fn checktype(&mut self, narg: i32, t: Type) {
        self.check_acceptable(narg);
        self.as_raw().checktype(narg, t)
    }

    pub unsafe fn checkany(&mut self, narg: i32) {
        self.check_acceptable(narg);
        self.as_raw().checkany(narg)
    }

    pub unsafe fn newmetatable(&mut self, tname: &str) -> bool {
        self.checkstack_(2); // uses 1 or 2 stack slots internally
        self.as_raw().newmetatable(tname)
    }

    pub unsafe fn checkudata(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        self.check_acceptable(narg);
        self.checkstack_(2); // uses 2 stack slots internally
        self.as_raw().checkudata(narg, tname)
    }

    pub unsafe fn where_(&mut self, lvl: i32) {
        // luaL_where() internally uses lua_pushfstring(), which manages stack size itself
        // so we don't need to call checkstack()
        self.as_raw().where_(lvl)
    }

    pub unsafe fn errorstr(&mut self, s: &str) -> ! {
        self.checkstack_(2);
        self.as_raw().errorstr(s)
    }

    pub unsafe fn checkoption<'a, T>(&mut self, narg: i32, def: Option<&str>, lst: &'a [(&str,T)])
                                    -> &'a T {
        self.check_acceptable(narg);
        self.as_raw().checkoption(narg, def, lst)
    }

    pub unsafe fn ref_(&mut self, t: i32) -> i32 {
        self.check_valid(t, true);
        self.checkstack_(1); // luaL_ref internally uses 1 stack slot
        self.as_raw().ref_(t)
    }

    pub unsafe fn unref(&mut self, t: i32, r: i32) {
        self.check_acceptable(t);
        self.checkstack_(1); // luaL_unref internally uses 1 stack slot
        self.as_raw().unref(t, r)
    }

    pub unsafe fn loadfile(&mut self, filename: Option<&path::Path>) -> Result<(),LoadFileError> {
        self.checkstack_(1);
        self.as_raw().loadfile(filename)
    }

    pub unsafe fn loadbuffer(&mut self, buf: &str, name: &str) -> Result<(),LoadError> {
        self.checkstack_(1);
        self.as_raw().loadbuffer(buf, name)
    }

    pub unsafe fn loadstring(&mut self, s: &str) -> Result<(),LoadError> {
        self.checkstack_(1);
        self.as_raw().loadstring(s)
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    pub unsafe fn gsub(&mut self, s: &str, p: &str, r: &str) -> &'static str {
        self.checkstack_(MINSTACK/2);
        self.as_raw().gsub(s, p, r)
    }

    pub unsafe fn argcheck(&mut self, cond: bool, narg: i32, extramsg: &str) {
        // NB: stack checking is not necessary
        self.as_raw().argcheck(cond, narg, extramsg)
    }

    pub unsafe fn dofile(&mut self, filename: Option<&path::Path>) -> bool {
        self.checkstack_(1);
        self.as_raw().dofile(filename)
    }

    pub unsafe fn dostring(&mut self, s: &str) -> bool {
        self.checkstack_(1);
        self.as_raw().dostring(s)
    }

    pub unsafe fn getmetatable_reg(&mut self, tname: &str) {
        self.as_raw().getmetatable_reg(tname)
    }

    pub fn buffinit<'a>(&'a mut self) -> Buffer<'a> {
        #![inline]
        let mut B = aux::raw::luaL_Buffer{
            p: ptr::null_mut(),
            lvl: 0,
            L: self.L,
            buffer: [0; aux::raw::LUAL_BUFFERSIZE as usize]
        };
        unsafe { aux::raw::luaL_buffinit(self.L, &mut B); }
        Buffer{ B: B, L: self }
    }
}

#[allow(missing_docs)]
impl<'l> RawState<'l> {
    pub unsafe fn registerlib(&mut self, libname: Option<&str>, l: &[(&str,CFunction)]) {
        #![inline]
        let mut cstrs = Vec::with_capacity(l.len());
        let mut l_ = Vec::with_capacity(l.len()+1);
        for &(name, func) in l.iter() {
            let cstr = CString::from_slice(name.as_bytes());
            l_.push(aux::raw::luaL_Reg{ name: cstr.as_ptr(), func: Some(func) });
            cstrs.push(cstr);
        }
        l_.push(aux::raw::luaL_Reg{ name: ptr::null(), func: None });
        let libcstr = libname.map(|s| CString::from_slice(s.as_bytes()));
        let libname_ = libcstr.map_or(ptr::null(), |cstr| cstr.as_ptr());
        aux::raw::luaL_register(self.L, libname_, l_.as_ptr())
    }

    pub unsafe fn getmetafield(&mut self, obj: i32, e: &str) -> bool {
        #![inline]
        let cstr = CString::from_slice(e.as_bytes());
        aux::raw::luaL_getmetafield(self.L, obj as c_int, cstr.as_ptr()) != 0
    }

    pub unsafe fn callmeta(&mut self, obj: i32, e: &str) -> bool {
        #![inline]
        let cstr = CString::from_slice(e.as_bytes());
        aux::raw::luaL_callmeta(self.L, obj as c_int, cstr.as_ptr()) != 0
    }

    pub unsafe fn typerror(&mut self, narg: i32, tname: &str) -> ! {
        #![inline]
        let cstr = CString::from_slice(tname.as_bytes());
        aux::raw::luaL_typerror(self.L, narg as c_int, cstr.as_ptr());
        unreachable!()
    }

    pub unsafe fn argerror(&mut self, narg: i32, extramsg: &str) -> ! {
        #![inline]
        let cstr = CString::from_slice(extramsg.as_bytes());
        aux::raw::luaL_argerror(self.L, narg as c_int, cstr.as_ptr());
        unreachable!()
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    // TODO: change return type to use core::str::Utf8Error
    pub unsafe fn checkstring(&mut self, narg: i32) -> Option<&'static str> {
        #![inline]
        str::from_utf8(self.checkbytes(narg)).ok()
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of hte value on the stack.
    pub unsafe fn checkbytes(&mut self, narg: i32) -> &'static [u8] {
        #![inline]
        let mut sz: libc::size_t = 0;
        let s = aux::raw::luaL_checklstring(self.L, narg, &mut sz);
        let buf = s as *const u8;
        mem::transmute::<&[u8], &'static [u8]>(slice::from_raw_buf(&buf, sz as usize))
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    // TODO: change return type to use core::str::Utf8Error
    pub unsafe fn optstring(&mut self, narg: i32, d: &'static str) -> Option<&'static str> {
        #![inline]
        str::from_utf8(self.optbytes(narg, d.as_bytes())).ok()
    }

    /// Note: the byte vector is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    pub unsafe fn optbytes(&mut self, narg: i32, d: &'static [u8]) -> &'static [u8] {
        #![inline]
        let mut sz: libc::size_t = 0;
        let cstr = CString::from_slice(d);
        let s = aux::raw::luaL_optlstring(self.L, narg, cstr.as_ptr(), &mut sz);
        let buf = s as *const u8;
        mem::transmute::<&[u8], &'static [u8]>(slice::from_raw_buf(&buf, sz as usize))
    }

    pub unsafe fn checknumber(&mut self, narg: i32) -> f64 {
        #![inline]
        aux::raw::luaL_checknumber(self.L, narg as c_int) as f64
    }

    pub unsafe fn optnumber(&mut self, narg: i32, d: f64) -> f64 {
        #![inline]
        aux::raw::luaL_optnumber(self.L, narg as c_int, d as raw::lua_Number) as f64
    }

    pub unsafe fn checkinteger(&mut self, narg: i32) -> isize {
        #![inline]
        aux::raw::luaL_checkinteger(self.L, narg as c_int) as isize
    }

    pub unsafe fn optinteger(&mut self, narg: i32, d: isize) -> isize {
        #![inline]
        aux::raw::luaL_optinteger(self.L, narg as c_int, d as raw::lua_Integer) as isize
    }

    pub unsafe fn checktype(&mut self, narg: i32, t: Type) {
        #![inline]
        aux::raw::luaL_checktype(self.L, narg as c_int, t as c_int)
    }

    pub unsafe fn checkany(&mut self, narg: i32) {
        #![inline]
        aux::raw::luaL_checkany(self.L, narg as c_int)
    }

    pub unsafe fn newmetatable(&mut self, tname: &str) -> bool {
        #![inline]
        let cstr = CString::from_slice(tname.as_bytes());
        aux::raw::luaL_newmetatable(self.L, cstr.as_ptr()) != 0
    }

    pub unsafe fn checkudata(&mut self, narg: i32, tname: &str) -> *mut libc::c_void {
        #![inline]
        let cstr = CString::from_slice(tname.as_bytes());
        aux::raw::luaL_checkudata(self.L, narg as c_int, cstr.as_ptr())
    }

    pub unsafe fn where_(&mut self, lvl: i32) {
        #![inline]
        aux::raw::luaL_where(self.L, lvl as c_int)
    }

    pub unsafe fn errorstr(&mut self, s: &str) -> ! {
        #![inline]
        self.where_(1);
        self.pushstring(s);
        self.concat(2);
        raw::lua_error(self.L);
        unreachable!()
    }

    pub unsafe fn checkoption<'a, T>(&mut self, narg: i32, def: Option<&str>, lst: &'a [(&str,T)])
                                    -> &'a T {
        let def_cstr = def.map(|d| CString::from_slice(d.as_bytes()));
        let defp = def_cstr.as_ref().map_or(ptr::null(), |c| c.as_ptr());
        let mut lst_cstrs = Vec::with_capacity(lst.len());
        let mut lstv = Vec::with_capacity(lst.len()+1);
        for &(k,_) in lst.iter() {
            let cstr = CString::from_slice(k.as_bytes());
            lstv.push(cstr.as_ptr());
            lst_cstrs.push(cstr);
        }
        lstv.push(ptr::null());
        let i = aux::raw::luaL_checkoption(self.L, narg as c_int, defp, lstv.as_ptr()) as usize;
        &lst[i].1
    }

    pub unsafe fn ref_(&mut self, t: i32) -> i32 {
        #![inline]
        aux::raw::luaL_ref(self.L, t as c_int) as i32
    }

    pub unsafe fn unref(&mut self, t: i32, r: i32) {
        #![inline]
        aux::raw::luaL_unref(self.L, t as c_int, r as c_int)
    }

    pub unsafe fn loadfile(&mut self, filename: Option<&path::Path>) -> Result<(),LoadFileError> {
        #![inline]
        let cstr = filename.map(|p| CString::from_slice(p.as_vec()));
        let ptr = cstr.as_ref().map_or(ptr::null(), |cstr| cstr.as_ptr());
        match aux::raw::luaL_loadfile(self.L, ptr) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadFileError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadFileError::ErrMem),
            aux::raw::LUA_ERRFILE => Err(LoadFileError::ErrFile),
            _ => self.errorstr("loadfile: unexpected error from luaL_loadfile")
        }
    }

    pub unsafe fn loadbuffer(&mut self, buf: &str, name: &str) -> Result<(),LoadError> {
        #![inline]
        let bp = buf.as_ptr() as *const libc::c_char;
        let bsz = buf.len() as libc::size_t;
        let cstr = CString::from_slice(name.as_bytes());
        match aux::raw::luaL_loadbuffer(self.L, bp, bsz, cstr.as_ptr()) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("loadbuffer: unexpected error from luaL_loadbuffer")
        }
    }

    pub unsafe fn loadstring(&mut self, s: &str) -> Result<(),LoadError> {
        #![inline]
        let cstr = CString::from_slice(s.as_bytes());
        match aux::raw::luaL_loadstring(self.L, cstr.as_ptr()) {
            0 => Ok(()),
            raw::LUA_ERRSYNTAX => Err(LoadError::ErrSyntax),
            raw::LUA_ERRMEM => Err(LoadError::ErrMem),
            _ => self.errorstr("loadstring: unexpected error from luaL_loadstring")
        }
    }

    /// Note: the string is returned as 'static to prevent borrowing the
    /// RawState, but its lifetime is actually that of the value on the stack.
    pub unsafe fn gsub(&mut self, s: &str, p: &str, r: &str) -> &'static str {
        #![inline]
        let (s_, p_, r_) = (CString::from_slice(s.as_bytes()),
                            CString::from_slice(p.as_bytes()),
                            CString::from_slice(r.as_bytes()));
        let (sp, pp, rp) = (s_.as_ptr(), p_.as_ptr(), r_.as_ptr());
        let res = aux::raw::luaL_gsub(self.L, sp, pp, rp);
        let cstr = ffi::c_str_to_bytes(&res);
        let res = str::from_utf8(cstr).unwrap();
        mem::transmute::<&str,&'static str>(res)
    }

    pub unsafe fn argcheck(&mut self, cond: bool, narg: i32, extramsg: &str) {
        #![inline]
        let cstr = CString::from_slice(extramsg.as_bytes());
        aux::raw::luaL_argcheck(self.L, cond, narg as c_int, cstr.as_ptr())
    }

    pub unsafe fn dofile(&mut self, filename: Option<&path::Path>) -> bool {
        #![inline]
        let cstr = filename.map(|p| CString::from_slice(p.as_vec()));
        let name = cstr.map_or(ptr::null(), |c| c.as_ptr());
        aux::raw::luaL_dofile(self.L, name) == 0
    }

    pub unsafe fn dostring(&mut self, s: &str) -> bool {
        #![inline]
        aux::raw::luaL_dostring(self.L, CString::from_slice(s.as_bytes()).as_ptr()) == 0
    }

    pub unsafe fn getmetatable_reg(&mut self, tname: &str) {
        #![inline]
        self.getfield(REGISTRYINDEX, tname)
    }
}

/// String buffer for building Lua strings piecemeal.
///
/// The Buffer assumes it needs longjmp safety, like ExternState.
pub struct Buffer<'a> {
    B: aux::raw::luaL_Buffer,
    /// A &mut pointer to the ExternState that created this Buffer.
    /// The buffer internally holds on to the *lua_Buffer that the State wraps,
    /// so to ensure safety it also borrows the &mut ExternState. Use this
    /// field to get mutable access to the State while the buffer is alive.
    pub L: &'a mut ExternState<'a>
}

/// Size of the internal buffer used by Buffer and returned by prepbuffer()
pub const BUFFERSIZE: usize = aux::raw::LUAL_BUFFERSIZE as usize;

impl<'a> Buffer<'a> {
    /// Adds the byte `c` to the buffer.
    pub unsafe fn addbyte(&mut self, c: u8) {
        #![inline]
        // don't call through to luaL_addchar, because we want to insert a call to checkstack()
        // iff we have to prep the buffer.
        let startp: *mut libc::c_char = &mut self.B.buffer[0];
        if self.B.p >= startp.offset(aux::raw::LUAL_BUFFERSIZE as isize) {
            self.L.checkstack_(1);
            aux::raw::luaL_prepbuffer(&mut self.B);
        }
        *self.B.p = c as libc::c_char;
        self.B.p = self.B.p.offset(1);
    }

    /// Adds the char `c` as utf-8 bytes to the buffer.
    pub unsafe fn addchar(&mut self, c: char) {
        #![inline]
        let mut buf = [0u8; 4];
        let count = c.encode_utf8(&mut buf).unwrap();
        self.addbytes(buf.slice_to(count));
    }

    /// Adds to the buffer a string of length `n` previously copied to the
    /// buffer area (see prepbuffer()).
    pub unsafe fn addsize(&mut self, n: usize) {
        #![inline]
        aux::raw::luaL_addsize(&mut self.B, n as libc::size_t)
    }

    /// Returns a pointer to an array of size BUFFERSIZE where you can copy a
    /// string to be added to the buffer. After copying the string into this
    /// space you must call addsize() with the size of the string to actually
    /// add it to the buffer.
    pub unsafe fn prepbuffer(&mut self) -> &mut [u8; aux::raw::LUAL_BUFFERSIZE as usize] {
        #![inline]
        self.L.checkstack_(1);
        // luaL_prepbuffer ends up returning the buffer field.
        // Rather than unsafely trying to transmute that to the array, just return the field
        // ourselves.
        aux::raw::luaL_prepbuffer(&mut self.B);
        mem::transmute::<&mut [i8; aux::raw::LUAL_BUFFERSIZE as usize],
                          &mut [u8; aux::raw::LUAL_BUFFERSIZE as usize]>(&mut self.B.buffer)
    }

    /// Adds the string to the buffer.
    pub unsafe fn addstring(&mut self, s: &str) {
        #![inline]
        self.addbytes(s.as_bytes())
    }

    /// Adds the byte vector to the buffer.
    pub unsafe fn addbytes(&mut self, bytes: &[u8]) {
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
    pub unsafe fn addvalue(&mut self) {
        #![inline]
        luaassert!(self.L, self.L.gettop() >= 1, "addvalue: stack underflow");
        self.L.checkstack_(1); // luaL_addvalue() needs this if the value is too large
        aux::raw::luaL_addvalue(&mut self.B)
    }

    /// Finishes the use of the buffer, leaving the final string on top of the
    /// stack.
    pub unsafe fn pushresult(mut self) {
        #![inline]
        self.L.checkstack_(1); // possibly needed for the emptybuffer
        aux::raw::luaL_pushresult(&mut self.B)
    }
}

/* Debug API */
/// Event codes
#[derive(Copy)]
pub enum DebugEvent {
    /// The call hook is called when the interpreter calls a function. The hook is called
    /// just after Lua enters the new function, before the function gets its arguments.
    HookCall = raw::LUA_HOOKCALL as isize,
    /// The return hook is called when the interpreter returns from a function. The hook is
    /// called just before Lua leaves the function. You have no access to the values to be
    /// returned by the function.
    HookRet = raw::LUA_HOOKRET as isize,
    /// The line hook is called when the interpreter is about to start the execution of a new
    /// line of code, or when it jumps back in the code (even to the same line).
    /// (This event only happens while Lua is executing a Lua function.)
    HookLine = raw::LUA_HOOKLINE as isize,
    /// The count hook is called after the interpreter executes every `count` instructions.
    /// (This event only happens while Lua is executing a Lua function.)
    HookCount = raw::LUA_HOOKCOUNT as isize,
    /// The tailret event is used when a HookRet hook is called while simulating a return from
    /// a function that did a tail call; in this case, it is useless to call getinfo().
    HookTailRet = raw::LUA_HOOKTAILRET as isize
}

impl DebugEvent {
    /// Converts a c_int event code to a DebugEvent.
    pub fn from_event(event: c_int) -> Option<DebugEvent> {
        match event {
            raw::LUA_HOOKCALL => Some(DebugEvent::HookCall),
            raw::LUA_HOOKRET => Some(DebugEvent::HookRet),
            raw::LUA_HOOKLINE => Some(DebugEvent::HookLine),
            raw::LUA_HOOKCOUNT => Some(DebugEvent::HookCount),
            raw::LUA_HOOKTAILRET => Some(DebugEvent::HookTailRet),
            _ => None
        }
    }
}

/// Event mask for HookCall
pub const MASKCALL: i32 = raw::LUA_MASKCALL as i32;
/// Event mask for HookRet
pub const MASKRET: i32 = raw::LUA_MASKRET as i32;
/// Event mask for HookLine
pub const MASKLINE: i32 = raw::LUA_MASKLINE as i32;
/// Event mask for HookCount
pub const MASKCOUNT: i32 = raw::LUA_MASKCOUNT as i32;

/// Type for functions to be called by the debugger in specific events
pub type Hook = raw::lua_Hook;

/// A structure used to carry different peices of information about an active function.
/// getstack() fills only the private part of this structure, for later use. To fill the other
/// fields of lua_Debug with useful information, call getinfo().
pub type Debug = raw::lua_Debug;

impl Debug {
    /// Returns a newly-zeroed instance of Debug
    pub fn new() -> Debug {
        #![inline]
        std::default::Default::default()
    }
}

impl State {
    /// Gets information about the interpreter runtime stack.
    ///
    /// This function returns a Debug structure with an identification of the
    /// activation record of the function executing at a given level. Level 0
    /// is the current running function, whereas level n+1 is the function that
    /// has called level n. When there are no errors, getstack() returns
    /// Some(Debug); when called with a level greater than the stack depth, it
    /// returns None.
    pub fn getstack(&mut self, level: i32) -> Option<Debug> {
        #![inline(always)]
        self.as_extern().getstack(level)
    }

    /// Returns information about a specific function or function invocation.
    ///
    /// To get information about a function invocation, the parameter `ar` must
    /// ve a valid activation record that was returned by a previous call to
    /// getstack() or given as argument to a hook.
    ///
    /// To get information about a function you push it onto the stack and
    /// start the `what` string with the character '>'. (In that case,
    /// getinfo() pops the function in the top of the stack.) For instance, to
    /// know in which line a function `f` was defined, you can write the
    /// following code:
    ///
    ///   let ar = Debug::new();
    ///   L.getfield(GLOBALSINDEX, "f"); // get global 'f'
    ///   L.getinfo(">S", &mut ar);
    ///   println!("{}", ar.linedefined);
    ///
    /// Each character in the string `what` selects some fields of the
    /// structure `ar` to be filled or a value to be pushed on the stack:
    ///
    /// * 'n': fills in the fields `name` and `namewhat`
    /// * 'S': fills in the fields `source`, `short_src`, `linedefined`,
    ///        `lastlinedefined`, and `what`
    /// * 'l': fills in the field `currentline`
    /// * 'u': fills in the field `nups`
    /// * 'f': pushes onto the stack the function that is running at the given
    ///        level
    /// * 'L': pushes onto the stack a table whose indices are the numbers of
    ///        the lines that are valid on the function. (A valid line is a
    ///        line with some associated code, that is, a line where you can
    ///        put a break point. Non-valid lines include empty lines and
    ///        comments.)
    ///
    /// This function returns `false` on error (for instance, an invalid option
    /// in `what`).
    ///
    /// Fails the task if `what` has interior NULs.
    pub fn getinfo(&mut self, what: &str, ar: &mut Debug) -> bool {
        #![inline(always)]
        unsafe { self.as_extern().getinfo(what, ar) }
    }

    /// Gets information about a local variable of a given activation record.
    /// The parameter `ar` must be a valid activation record that was filled by
    /// a previous call to getstack() or given as an argument to a hook. The
    /// index `n` selects which local variable to inspect (1 is the first
    /// parameter or active local variable, and so on, until the last active
    /// local variable). getlocal() pushes the variable's value onto the stack
    /// and returns its name.
    ///
    /// Variable names starting with '(' represent internal variables (loop
    /// control variables, temporaries, and C function locals).
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn getlocal<'a>(&mut self, ar: &'a Debug, n: i32) -> Option<&'a [u8]> {
        #![inline(always)]
        unsafe { self.as_extern().getlocal(ar, n) }
    }

    /// Sets the value of a local variable of a given activation record.
    /// Parameters `ar` and `n` are as in getlocal(). setlocal() assigns the
    /// value at the top of the stack to the variable and returns its name. It
    /// also pops the value from the stack.
    ///
    /// Returns None (and pops nothing) when the index is greater than the
    /// number of active local variables.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn setlocal<'a>(&mut self, ar: &'a mut Debug, n: i32) -> Option<&'a [u8]> {
        #![inline(always)]
        unsafe { self.as_extern().setlocal(ar, n) }
    }

    /// Gets information about a closure's upvalue. (For Lua functions,
    /// upvalues are the external local variables that the function uses, and
    /// that are consequently included in its closure.) getupvalue() gets the
    /// index `n` of an upvalue, pushes the upvalue's value onto the stack, and
    /// returns its name. `funcindex` points to the closure in the stack.
    /// (Upvalues have no particular order, as they are active through the
    /// whole function. So, they are numbered in an arbitrary order.)
    ///
    /// Returns None (and pushes nothing) when the index is greater than the
    /// number of upvalues. For C functions, this function uses the empty
    /// string "" as a name for all upvalues.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn getupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        #![inline(always)]
        unsafe { self.as_extern().getupvalue(funcidx, n) }
    }

    /// Sets the value of a closure's upvalue. It assigns the value at the top
    /// of the stack to the upvalue and returns its name. It also pops the
    /// value from the stack. Parameters `funcindex` and `n` are as in
    /// getupvalue().
    ///
    /// Returns None (and pops nothing) when the index is greater than the
    /// number of upvalues.
    ///
    /// The name is returned as a &[u8] to avoid confusion with failed utf-8
    /// decoding vs invalid indices.
    pub fn setupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        #![inline(always)]
        unsafe { self.as_extern().setupvalue(funcidx, n) }
    }

    /// Sets the debugging hook function.
    ///
    /// Argument `f` is the hook function. `mask` specifies on which events the
    /// hook will be called: it is formed by a bitwise OR of the Mask*
    /// constants in DebugEvent. The `count` argument is only meaningful when
    /// the mask includes DebugEvent::MaskCount.
    ///
    /// A hook is disabled by setting `mask` to zero.
    pub fn sethook(&mut self, f: Hook, mask: i32, count: i32) {
        #![inline(always)]
        self.as_extern().sethook(f, mask, count)
    }

    /// Returns the current hook function
    pub fn gethook(&mut self) -> Hook {
        #![inline(always)]
        self.as_extern().gethook()
    }

    /// Returns the current hook mask
    pub fn gethookmask(&mut self) -> i32 {
        #![inline(always)]
        self.as_extern().gethookmask()
    }

    /// Returns the current hook count
    pub fn gethookcount(&mut self) -> i32 {
        #![inline(always)]
        self.as_extern().gethookcount()
    }
}

#[allow(missing_docs)]
impl<'l> ExternState<'l> {
    pub fn getstack(&mut self, level: i32) -> Option<Debug> {
        self.as_raw().getstack(level)
    }

    pub unsafe fn getinfo(&mut self, what: &str, ar: &mut Debug) -> bool {
        if what.starts_with(">") {
            luaassert!(self, self.gettop() >= 1 && self.isfunction(-1),
                       "getinfo: top stack value is not a function");
        }
        if what.find(['f', 'L'].as_slice()).is_some() {
            self.checkstack_(1);
        }
        self.as_raw().getinfo(what, ar)
    }

    pub unsafe fn getlocal<'a>(&mut self, ar: &'a Debug, n: i32) -> Option<&'a [u8]> {
        self.checkstack_(1);
        self.as_raw().getlocal(ar, n)
    }

    pub unsafe fn setlocal<'a>(&mut self, ar: &'a mut Debug, n: i32) -> Option<&'a [u8]> {
        luaassert!(self, self.gettop() >= 1, "setlocal: stack underflow");
        self.as_raw().setlocal(ar, n)
    }

    pub unsafe fn getupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        self.check_acceptable(funcidx);
        self.checkstack_(1);
        self.as_raw().getupvalue(funcidx, n)
    }

    pub unsafe fn setupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        self.check_acceptable(funcidx);
        self.checkstack_(1);
        self.as_raw().setupvalue(funcidx, n)
    }

    pub fn sethook(&mut self, f: Hook, mask: i32, count: i32) {
        self.as_raw().sethook(f, mask, count)
    }

    pub fn gethook(&mut self) -> Hook {
        self.as_raw().gethook()
    }

    pub fn gethookmask(&mut self) -> i32 {
        self.as_raw().gethookmask()
    }

    pub fn gethookcount(&mut self) -> i32 {
        self.as_raw().gethookcount()
    }
}

#[allow(missing_docs)]
impl<'l> RawState<'l> {
    pub fn getstack(&mut self, level: i32) -> Option<Debug> {
        #![inline]
        let mut ar: Debug = std::default::Default::default();
        if unsafe { raw::lua_getstack(self.L, level as c_int, &mut ar) != 0 } {
            Some(ar)
        } else {
            None
        }
    }

    pub unsafe fn getinfo(&mut self, what: &str, ar: &mut Debug) -> bool {
        #![inline]
        raw::lua_getinfo(self.L, CString::from_slice(what.as_bytes()).as_ptr(), ar) != 0
    }

    pub unsafe fn getlocal<'a>(&mut self, ar: &'a Debug, n: i32) -> Option<&'a [u8]> {
        #![inline]
        let res = raw::lua_getlocal(self.L, ar, n as c_int);
        c_str_to_bytes(res)
    }

    pub unsafe fn setlocal<'a>(&mut self, ar: &'a mut Debug, n: i32) -> Option<&'a [u8]> {
        #![inline]
        let res = raw::lua_setlocal(self.L, ar, n as c_int);
        c_str_to_bytes(res)
    }

    pub unsafe fn getupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        #![inline]
        let res = raw::lua_getupvalue(self.L, funcidx as c_int, n as c_int);
        c_str_to_bytes(res)
    }

    pub unsafe fn setupvalue<'a>(&'a mut self, funcidx: i32, n: i32) -> Option<&'a [u8]> {
        #![inline]
        let res = raw::lua_setupvalue(self.L, funcidx as c_int, n as c_int);
        c_str_to_bytes(res)
    }

    pub fn sethook(&mut self, f: Hook, mask: i32, count: i32) {
        #![inline]
        unsafe { raw::lua_sethook(self.L, f, mask as c_int, count as c_int); }
    }

    pub fn gethook(&mut self) -> Hook {
        #![inline]
        unsafe { raw::lua_gethook(self.L) }
    }

    pub fn gethookmask(&mut self) -> i32 {
        #![inline]
        unsafe { raw::lua_gethookmask(self.L) as i32 }
    }

    pub fn gethookcount(&mut self) -> i32 {
        #![inline]
        unsafe { raw::lua_gethookcount(self.L) as i32 }
    }
}

unsafe fn c_str_to_bytes<'a>(cstr: *const libc::c_char) -> Option<&'a [u8]> {
    #![inline]
    if cstr.is_null() {
        None
    } else {
        let bytes = ffi::c_str_to_bytes(&cstr);
        Some(mem::transmute::<&[u8],&'a [u8]>(bytes))
    }
}
