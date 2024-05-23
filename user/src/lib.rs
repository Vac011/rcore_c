#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod file;
mod io;
mod lang_items;
mod net;
mod sync;
mod syscall;
mod task;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
pub use file::*;
pub use io::*;
pub use net::*;
pub use sync::*;
use syscall::*;
pub use task::*;

use lib_so::{Runtime, RunMutex};
use lib_so::{MAX_THREAD_NUM, PRIO_NUM, CidHandle};
use lib_so::*;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{vec, collections::VecDeque};

const USER_HEAP_SIZE: usize = 32768;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Runtime = Runtime::new(true);

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    unsafe {
        EXECUTOR = Runtime {
            // 初始化 currents 数组，每个元素都是 None
            currents: [None; MAX_THREAD_NUM],
            // 初始化 tasks，使用空的 BTreeMap
            tasks: BTreeMap::new(),
            // 初始化 ready_queue 数组，每个元素都是空的 Vec<VecDeque<CidHandle>>
            ready_queue: {
                const R_QUEUE_VALUE: Vec<VecDeque<CidHandle>> = Vec::new();
                [R_QUEUE_VALUE; MAX_THREAD_NUM]
            },
            // 初始化 pending_set 数组，每个元素都是空的 BTreeSet<usize>
            pending_set: {
                const P_SET_VALUE: BTreeSet<usize> = BTreeSet::new();
                [P_SET_VALUE; MAX_THREAD_NUM]
            },
            // 初始化 bitmap，使用空的 BitMap
            bitmap: BitMap::new(),
            // 初始化 max_prio，默认为 0
            max_prio: 0,
            // 初始化 thread_prio 数组，每个元素都为 0
            thread_prio: [0; MAX_THREAD_NUM],
            // 初始化 wr_lock，使用新建的 RunMutex
            wr_lock: RunMutex::new(true),
            // 初始化 waits，使用空的 Vec<usize>
            waits: Vec::new(),
        };
    }
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(argc, v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

#[macro_export]
macro_rules! vstore {
    ($var: expr, $value: expr) => {
        // unsafe { core::intrinsics::volatile_store($var_ref as *const _ as _, $value) }
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!($var), $value); }
    };
}

#[macro_export]
macro_rules! vload {
    ($var: expr) => {
        // unsafe { core::intrinsics::volatile_load($var_ref as *const _ as _) }
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!($var)) }
    };
}

#[macro_export]
macro_rules! memory_fence {
    () => {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst)
    };
}
