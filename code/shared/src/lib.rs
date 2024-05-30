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
pub mod syscall;
pub mod config;
pub mod sharedscheduler;
#[macro_use]
pub mod console;
// 引用外部模块
extern crate alloc;
pub use config::*;
pub use sharedscheduler::*;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

