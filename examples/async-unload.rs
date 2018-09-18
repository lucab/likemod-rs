//! A simple example showing how to (async) unload a module.
//!
//! It tries to unload a module by name in an asynchronous way,
//! retrying for 5s before giving up if the module is busy.
//!
//! This is an example ONLY: do NOT panic/unwrap/assert
//! in production code!

extern crate likemod;
extern crate tokio;

use std::{num, process, time};
use tokio::runtime::current_thread;
use tokio::timer::timeout;

fn main() {
    // Get from cmdline the name of the module to unload.
    let modname = std::env::args().nth(1).expect("missing module name");

    // Assemble a future to unload the module; if busy,
    // retry every 500ms and time out after 5 seconds.
    let pause_ms = num::NonZeroU64::new(500).unwrap();
    let modunload = likemod::ModUnloader::new().unload_async(&modname, pause_ms);
    let tout = time::Duration::from_secs(15);
    let fut = timeout::Timeout::new(modunload, tout);

    // Run the future until completion.
    if let Err(err) = current_thread::block_on_all(fut) {
        eprintln!("FAILED: {:?}", err);
        process::exit(1)
    }

    // Success!
    println!("module '{}' unloaded.", modname);
}
