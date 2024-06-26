#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(inline_const)]


//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
extern crate alloc;
extern crate shared;
#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod mm;
mod net;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

core::arch::global_asm!(include_str!("entry.asm"));

// async fn main_coroutine() {
//     panic!("into the main coroutine");
// }

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(task::run_tasks())
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use mm::KERNEL_SPACE;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    mm::init();
    UART.init();
    println!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    println!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    println!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    println!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    KERNEL_SPACE.exclusive_access().add_runtime();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    // let pid = sys_getpid() as usize;
    // let tid = sys_gettid() as usize;
    shared::spawn(create_future(), 7, 0, 0, shared::CoroutineKind::KernSche);
    shared::poll_future(0, 0);
    // task::run_tasks();
    panic!("Unreachable in rust_main!");
}
