//! A pure-Rust library to work with **Li**nux **ke**rnel **mo**dules.
//!
//! It provides support for loading and unloading kernel
//! modules on Linux.
//! For further details, see `init_module(2)` and `delete_module(2)`
//! manpages.
//!
//! ```rust,no_run
//! use likemod;
//! # use likemod::errors::Result;
//!
//! fn load_modfile(fpath: &std::path::Path) -> Result<()> {
//!     // Get a file descriptor to the kernel module object.
//!     let fmod = std::fs::File::open(fpath)?;
//!
//!     // Assemble module parameters for loading.
//!     let mut params = likemod::ModParams::new();
//!     params.insert("bus_delay".to_string(), likemod::ModParamValue::Int(5));
//!
//!     // Try to load the module. It can fail if the kernel
//!     // version and signature don't match.
//!     let loader = likemod::ModLoader::default().set_parameters(params);
//!     loader.load_module_file(&fmod)
//! }
//! ```

#![deny(missing_docs)]

extern crate errno;
#[macro_use]
extern crate error_chain;
#[cfg(feature = "async")]
extern crate futures;
extern crate libc;

pub mod errors;
mod load;
mod nr;
mod unload;

pub use load::{ModLoader, ModParams, ModParamValue};
pub use unload::ModUnloader;
