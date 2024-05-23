use lib_so::{Runtime, RunMutex};
use lib_so::{MAX_THREAD_NUM, PRIO_NUM, CidHandle};
use lib_so::*;
// use syscall::yield_;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{vec, collections::VecDeque};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use spin::Mutex;
use buddy_system_allocator::Heap;

#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Runtime = Runtime::new(true);

// 托管空间 16 KiB
const MEMORY_SIZE: usize = 1 << 21;
#[no_mangle]
#[link_section = ".data.memory"]
static mut MEMORY: [u8; MEMORY_SIZE] = [0u8; MEMORY_SIZE];


/// 初始化全局分配器和内核堆分配器。
pub fn init() {

    unsafe {
        HEAP.lock().init(
            MEMORY.as_ptr() as usize,
            MEMORY_SIZE,
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
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        while true {
            let op_heap = HEAP.try_lock();
            if op_heap.is_some() {
                return op_heap.unwrap().alloc(layout).ok()
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr());
            }
            // yield_();
        }
        return 0 as *mut u8;
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        while true {
            let op_heap = HEAP.try_lock();
            if op_heap.is_some() {
                op_heap.unwrap().dealloc(NonNull::new_unchecked(ptr), layout);
                return;
            }
            // yield_();
        }
    }
}

