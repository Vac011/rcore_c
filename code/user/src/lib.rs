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

use shared::{Runtime, RunMutex};
use shared::{MAX_THREAD_NUM, PRIO_NUM, CidHandle};
use shared::*;
use alloc::collections::{BTreeMap, BTreeSet};
// use alloc::{vec, collections::VecDeque};
// use lazy_static::lazy_static;

const USER_HEAP_SIZE: usize = 1 << 21 ;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Runtime = Runtime::new(false);

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    // unsafe {
    //     EXECUTOR.ready_queue = vec![VecDeque::new(); MAX_THREAD_NUM * PRIO_NUM];
    // }
    unsafe {
        EXECUTOR = Runtime {
            // 初始化 currents 数组，每个元素都是 None
            currents: [None; MAX_THREAD_NUM],
            // 初始化 tasks，使用空的 BTreeMap
            tasks: BTreeMap::new(),
            // ready_queue: init_ready_queue(),
            // ready_queue: vec![vec![VecDeque::new(); PRIO_NUM]; MAX_THREAD_NUM],
            ready_queue: {
                const R_SET_VALUE: BTreeSet<CidHandle> = BTreeSet::new();
                [R_SET_VALUE; MAX_THREAD_NUM * PRIO_NUM]
            },
            pending_set: init_pending_set(),
            // 初始化 bitmap，使用空的 BitMap
            bitmap: BitMap::new(),
            threadmap: [BitMap::new(); MAX_THREAD_NUM],
            // 初始化 max_prio，默认为 0
            max_prio: PRIO_NUM,
            // 初始化 thread_prio 数组，每个元素都为 0
            thread_prio: [PRIO_NUM; MAX_THREAD_NUM],
            // 初始化 wr_lock，使用新建的 RunMutex
            wr_lock: RunMutex::new(true),
            // 初始化 waits，使用空的 Vec<usize>
            waits: Vec::new(),
        };
    }
    // ready_queue 数组，每个元素都是空的 Vec<VecDeque<CidHandle>>
    // fn init_ready_queue() -> [Vec<VecDeque<CidHandle>>; MAX_THREAD_NUM] {
    //     let mut array: [Vec<VecDeque<CidHandle>>; MAX_THREAD_NUM] = Default::default();
    //     for vec in &mut array {
    //         *vec = vec![VecDeque::new(); PRIO_NUM];
    //     }
    //     array
    // }

    // 初始化 pending_set 数组，每个元素都是空的 BTreeSet<usize>
    fn init_pending_set() -> [BTreeSet<usize>; MAX_THREAD_NUM] {
        Default::default()
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
