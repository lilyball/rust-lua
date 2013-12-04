//! Lua 5.1 bindings for Rust

#[link(name = "lua",
       package_id = "lua",
       vers = "0.1")];

#[comment = "Lua 5.1 bindings for Rust"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[allow(missing_doc)];

#[link(name = "lua.5.1")]
extern {}

pub static VERSION: &'static str = "Lua 5.1";
pub static VERSION_NUM: int = 501;

pub static MULTRET: int = raw::MULTRET as int;

/// Minimum Lua stack available to a C function
pub static MINSTACK: int = 20;

pub mod raw;
pub mod aux;

#[path = "lualib.rs"]
pub mod lib;
