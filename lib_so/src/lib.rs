#![no_std]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(atomic_from_mut, inline_const)]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![allow(unused)]

// 声明当前(库)crate的其他所有module
#[macro_use]
pub mod config;
pub mod sharedschedule;

// 引用外部模块
extern crate alloc;
pub use config::*;
pub use sharedschedule::*;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use vdso::get_libfn;

pub fn lib_demo () -> &'static str {
    "This is from lib_so/sharedshedule/\n"
}

pub fn lib_max_prio_pid () -> usize {
    max_prio_pid()
}
// get_libfn!(
//     pub fn spawn(f: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize, tid:usize, kind: CoroutineKind) -> usize {}
// );


// get_libfn!(
//     pub fn lib_demo () -> &'static str{}
// );
