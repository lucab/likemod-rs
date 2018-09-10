/* from <linux/module.h> */

pub const IGNORE_MODVERSIONS: u32 = 1;
pub const IGNORE_VERMAGIC: u32 = 2;

/* from <unistd.h> */

#[cfg(target_arch = "x86")]
pub const FINIT_MODULE: i32 = 350;
#[cfg(target_arch = "x86")]
pub const DELETE_MODULE: i32 = 129;

#[cfg(target_arch = "x86_64")]
pub const FINIT_MODULE: i64 = 313;
#[cfg(target_arch = "x86_64")]
pub const DELETE_MODULE: i64 = 176;

#[cfg(target_arch = "aarch64")]
pub const FINIT_MODULE: i64 = 379;
#[cfg(target_arch = "aarch64")]
pub const DELETE_MODULE: i64 = 106;

#[cfg(target_arch = "powerpc")]
pub const FINIT_MODULE: i64 = 273;
#[cfg(target_arch = "powerpc")]
pub const DELETE_MODULE: i64 = 129;

#[cfg(target_arch = "arm")]
pub const FINIT_MODULE: i32 = 379;
#[cfg(target_arch = "arm")]
pub const DELETE_MODULE: i32 = 129;
