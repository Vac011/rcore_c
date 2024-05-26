use lib_so::{Runtime, RunMutex};
use lib_so::{MAX_THREAD_NUM, PRIO_NUM, CidHandle};
use lib_so::*;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
// use alloc::{vec, collections::VecDeque};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
// use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use spin::Mutex;
use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::Heap;

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

// pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Runtime = Runtime::new(true);

#[no_mangle]
#[link_section = ".bss.memory"]
static mut MEMORY: [u8; KERNEL_HEAP_SIZE] = [0u8; KERNEL_HEAP_SIZE];


/// 初始化全局分配器和内核堆分配器。
pub fn init_heap() {

    unsafe {
        HEAP.lock().init(
            MEMORY.as_ptr() as usize,
            KERNEL_HEAP_SIZE,
        );
        // HEAP.lock().transfer(NonNull::new_unchecked(MEMORY.as_mut_ptr()), MEMORY.len());
        
    }
    // error!("heap {:#x}", unsafe{ &mut HEAP as *mut Mutex<MutAllocator<32>> as usize });
    // error!("heap {:#x}", core::mem::size_of::<Mutex<MutAllocator<32>>>());
    // error!("EXECUTOR ptr {:#x}", unsafe{ &mut EXECUTOR as *mut Executor as usize });
    // error!("memory {:#x}", unsafe{ &mut MEMORY as *mut u8 as usize });
    unsafe {
        EXECUTOR = Runtime {
            // 初始化 currents 数组，每个元素都是 None
            currents: [None; MAX_THREAD_NUM],
            // 初始化 tasks，使用空的 BTreeMap
            tasks: BTreeMap::new(),
            // 初始化 ready_queue 数组，每个元素都是空的 Vec<VecDeque<CidHandle>>
            // ready_queue: {
            //     const R_QUEUE_VALUE: Vec<VecDeque<CidHandle>> = Vec::new();
            //     [R_QUEUE_VALUE; MAX_THREAD_NUM]
            // },
            ready_queue: {
                const R_SET_VALUE: BTreeSet<CidHandle> = BTreeSet::new();
                [R_SET_VALUE; MAX_THREAD_NUM * PRIO_NUM]
            },
            // 初始化 pending_set 数组，每个元素都是空的 BTreeSet<usize>
            pending_set: {
                const P_SET_VALUE: BTreeSet<usize> = BTreeSet::new();
                [P_SET_VALUE; MAX_THREAD_NUM]
            },
            // 初始化 bitmap，使用空的 BitMap
            bitmap: BitMap::new(),
            threadmap: [BitMap::new(); MAX_THREAD_NUM],
            // 初始化 max_prio，默认为 0
            max_prio: PRIO_NUM+1,
            // 初始化 thread_prio 数组，每个元素都为 0
            thread_prio: [PRIO_NUM+1; MAX_THREAD_NUM],
            // 初始化 wr_lock，使用新建的 RunMutex
            wr_lock: RunMutex::new(true),
            // 初始化 waits，使用空的 Vec<usize>
            waits: Vec::new(),
            temp: 0,
        };
    }
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP.lock().alloc(layout).ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}


