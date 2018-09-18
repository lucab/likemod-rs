# likemod

[![Build Status](https://travis-ci.org/lucab/likemod-rs.svg?branch=master)](https://travis-ci.org/lucab/likemod-rs)
[![crates.io](https://img.shields.io/crates/v/likemod.svg)](https://crates.io/crates/likemod)
[![Documentation](https://docs.rs/likemod/badge.svg)](https://docs.rs/likemod)

A pure-Rust library to work with **Li**nux **ke**rnel **mo**dules.

It provides support for loading and unloading kernel modules on Linux.
For further details, see `init_module(2)` and `delete_module(2)` manpages.

## Example

```rust
extern crate likemod;
use likemod::errors;

fn load_modfile(fpath: &std::path::Path) -> errors::Result<()> {
    // Get a file descriptor to the kernel module object.
    let fmod = std::fs::File::open(fpath)?;

    // Assemble module parameters for loading.
    let mut params = likemod::ModParams::new();
    params.insert("bus_delay".to_string(), likemod::ModParamValue::Int(5));

    // Try to load the module. It can fail if the kernel
    // version and signature don't match.
    let loader = likemod::ModLoader::default().set_parameters(params);
    loader.load_module_file(&fmod)
}
```

Some more examples are available under [examples](examples).

## Features

This crate supports the following optional features:
 * `async`: this provides an `unload_async` method, using futures.

## License

Licensed under either of

 * MIT license - <http://opensource.org/licenses/MIT>
 * Apache License, Version 2.0 - <http://www.apache.org/licenses/LICENSE-2.0>

at your option.
