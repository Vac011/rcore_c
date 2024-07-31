//! 共享调度器模块

#![no_std]
#![no_main]
#![feature(inline_const)]
#[allow(unused)]
extern crate alloc;

use crate::syscall::*;
use crate::println;
use crate::config::*;
use crate::ENTRY;
use spin::Mutex;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicUsize;
use crate::{Runtime, CidHandle, CoroutineKind};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::task::Poll;
use buddy_system_allocator::Heap;
type LockedHeap = Mutex<Heap>;


// /// 各个进程的最高优先级协程，通过共享内存的形式进行通信
// pub static mut PROCESS_PRIO_ARRAY: [AtomicUsize; MAX_PROC_NUM ] = [const { AtomicUsize::new(PRIO_NUM) }; MAX_PROC_NUM ];
// /// 各个线程的最高优先级协程，通过共享内存的形式进行通信
// pub static mut THREAD_PRIO_ARRAY: [AtomicUsize; (MAX_THREAD_NUM ) * (MAX_PROC_NUM )] = [const { AtomicUsize::new(PRIO_NUM) }; (MAX_THREAD_NUM ) * (MAX_PROC_NUM)];

pub struct SharedProcessPrioArray {
    pub data: [AtomicUsize; MAX_PROC_NUM + 2],
}
    
impl SharedProcessPrioArray{
    pub const fn new() -> Self {
        Self {
            data: [const { AtomicUsize::new(PRIO_NUM) }; MAX_PROC_NUM + 2],
            // data: [const { AtomicUsize::new(usize::MAX) }; MAX_THREAD_NUM * MAX_PROC_NUM],
        }
    }
}
    
pub struct SharedThreadPrioArray {
    pub data: [AtomicUsize; MAX_THREAD_NUM * MAX_PROC_NUM],
}
    
impl SharedThreadPrioArray{
    pub const fn new() -> Self {
        Self {
            // data: [const { AtomicUsize::new(usize::MAX) }; MAX_PROC_NUM],
            data: [const { AtomicUsize::new(PRIO_NUM) }; MAX_THREAD_NUM * MAX_PROC_NUM],
        }
    }
}

    /// 进程的 Executor 调用这个函数，通过原子操作更新自己的最高优先级
#[no_mangle]
#[inline(never)]

pub fn update_process_prio(idx: usize, prio: usize) {
    unsafe {
        // PROCESS_PRIO_ARRAY[idx].store(prio, Ordering::Relaxed);
        let ret = (PROCESS_PRIO_BASE as *const usize) as *mut usize as *mut SharedProcessPrioArray;
        (*ret).data[idx].store(prio, Ordering::Relaxed);
    }
}
pub fn update_thread_prio(idx: usize,idy: usize, prio: usize) {
    unsafe {
        // THREAD_PRIO_ARRAY[idx*MAX_THREAD_NUM+idy].store(prio, Ordering::Relaxed);
        let ret = (THREAD_PRIO_BASE as *const usize) as *mut usize as *mut SharedThreadPrioArray;
        (*ret).data[idx*MAX_THREAD_NUM+idy].store(prio, Ordering::Relaxed);
    }
}

/// 内核重新调度进程时，调用这个函数，选出优先级最高的进程，再选出对应的线程
/// 所有进程的优先级相同时，则内核会优先执行协程，这里用 0 来表示内核的优先级
#[no_mangle]
#[inline(never)]
pub fn max_prio_pid() -> usize {
    let mut dataa;
    let mut pid ;
    let mut ret;
    let mut pidarray = ((PROCESS_PRIO_BASE as *const usize) as *mut usize as *mut SharedProcessPrioArray);
    unsafe {
        pid = (*pidarray).data[MAX_PROC_NUM].load(Ordering::Relaxed);
        pid = (pid+1)%MAX_PROC_NUM;

        ret = (PROCESS_PRIO_BASE as *const usize) as *mut usize as *mut SharedProcessPrioArray;
        dataa = (*ret).data[pid].load(Ordering::Relaxed);
        // ret = PROCESS_PRIO_ARRAY[0].load(Ordering::Relaxed);
    }
    for i in 0..MAX_PROC_NUM {
        unsafe {
            // let prio = PROCESS_PRIO_ARRAY[i].load(Ordering::Relaxed);
            let prio = (*ret).data[i].load(Ordering::Relaxed);
            if prio < dataa {
                dataa = prio;
                pid = i;
            }
        }
    }
    unsafe {
        (*pidarray).data[MAX_PROC_NUM].store(pid, Ordering::Relaxed);
    }
    //println!("pid returned is :{}",pid);
    pid
}
#[no_mangle]
#[inline(never)]
pub fn max_prio_tid(pid: usize) -> usize {
    let mut dataa;
    let mut tid ;
    let mut ret;
    let mut tidarray = ((PROCESS_PRIO_BASE as *const usize) as *mut usize as *mut SharedProcessPrioArray);
    unsafe {
        // ret = THREAD_PRIO_ARRAY[pid*MAX_THREAD_NUM].load(Ordering::Relaxed);

        tid = (*tidarray).data[MAX_PROC_NUM + 1].load(Ordering::Relaxed);
        tid = (tid) % MAX_THREAD_NUM;
        ret = (THREAD_PRIO_BASE as *const usize) as *mut usize as *mut SharedThreadPrioArray;
        dataa = (*ret).data[pid*MAX_THREAD_NUM+tid].load(Ordering::Relaxed);
    }
    for i in 0..MAX_THREAD_NUM {
        unsafe {
            // let prio = THREAD_PRIO_ARRAY[pid*MAX_THREAD_NUM+i].load(Ordering::Relaxed);
            let prio = (*ret).data[pid*MAX_THREAD_NUM+i].load(Ordering::Relaxed);
            if prio < dataa {
                dataa = prio;
                tid = i;
            }
        }
    }

    unsafe {
        (*tidarray).data[MAX_PROC_NUM + 1].store(tid, Ordering::Relaxed);
    }
    //println!("tid returned is :{}",tid);
    tid
}

// 添加协程，内核和用户态都可以调用
#[no_mangle]
#[inline(never)]
pub fn spawn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize, tid: usize, kind: CoroutineKind) -> usize {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
        let cid = (*exe).spawn(future, prio, pid, tid, kind);
        // 更新优先级标记 TODO
        let prio = (*exe).max_prio;
        update_process_prio(pid, prio);
        let thread_p = (*exe).thread_prio[tid];
        update_thread_prio(pid, tid, thread_p);
        return cid;
    }
}



/// 用户程序执行协程
#[no_mangle]
#[inline(never)]
pub fn poll_future(pid: usize, tid: usize) {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
        loop {
            if (*exe).is_empty() {
                // println!("ex is empty");如果 Executor 的任务队列为空，则退出循环。
                break;
            }
            // let tid = max_prio_tid(pid);
            let task = (*exe).fetch_coroutine(tid as usize);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    
                    // println!("user task kind {:?}", task.kind);
                    println!("The coroutine belongs to pid: {}", pid);
                    println!("The coroutine belongs to tid: {}", tid);
                    println!("The coroutine belongs to cid: {}", cid.0);
                    
                    match task.execute() {
                        Poll::Pending => {
                            (*exe).pending_coroutine(tid, cid);
                            // println!("pending ");
                        }
                        Poll::Ready(()) => {
                            (*exe).del_coroutine(tid, cid);
                            // println!("ready");
                        }
                    };
                    
                    let _lock = (*exe).wr_lock.lock();
                    let prio: usize = (*exe).max_prio;
                    update_process_prio(pid, prio);
                    let thread_p: usize = (*exe).thread_prio[tid];
                    update_thread_prio(pid, tid, thread_p);
                }//如果获取到了任务，则执行该任务。如果任务处于挂起状态，则将其标记为挂起状态；如果任务已完成，则删除该协程。然后更新进程的最高优先级。
                _ => {
                    // 任务队列不为空，但就绪队列为空，等待任务唤醒.如果没有获取到任务，则让出 CPU 执行权。
                    sys_yield_coroutine();
                }
            }
            // 执行完优先级最高的协程，检查优先级，判断是否让权
            let max_prio_pid = max_prio_pid();
            if pid != max_prio_pid {
                sys_yield_coroutine();
            }

            let max_prio_tid = max_prio_tid(pid);
            if tid != max_prio_tid {
                sys_yield_coroutine();
            }
        }
    }
}

/// 获取当前正在执行的协程 id
#[no_mangle]
#[inline(never)]
pub fn current_cid(tid: usize) -> usize {
    assert!(tid < MAX_THREAD_NUM);
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
        (*exe).currents[tid].as_mut().unwrap().get_val()
    }
}

/// 协程重新入队，手动执行唤醒的过程，内核和用户都会调用这个函数
// #[no_mangle]
// #[inline(never)]
// pub fn re_back(cid: usize, pid: usize, tid: usize) {
//     // println!("[Exec]re back func enter");
//     let mut start = 0;
    
//     unsafe {
//         let heapptr = *(HEAP_BUFFER as *const usize);
//         let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
//         let prio = (*exe).pull_to_ready(tid, CidHandle(cid));
//         // 重新入队之后，需要检查优先级
//         let process_prio = PROCESS_PRIO_ARRAY[pid].load(Ordering::Relaxed);
//         if prio < process_prio {
//             PROCESS_PRIO_ARRAY[pid].store(prio, Ordering::Relaxed);
//         }
//         let thread_prio = THREAD_PRIO_ARRAY[pid*MAX_PROC_NUM+tid].load(Ordering::Relaxed);
//         if prio < thread_prio {
//             THREAD_PRIO_ARRAY[pid*MAX_PROC_NUM+tid].store(prio, Ordering::Relaxed);
//         }
//     }
// }

#[no_mangle]
#[inline(never)]
pub fn get_pending_status(tid: usize, cid: usize) -> bool {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
        return (*exe).is_pending(tid, cid)
    }
}

pub fn check_prio(pid: usize, tid: usize) -> bool {
    unsafe {
        // let heapptr = *(HEAP_BUFFER as *const usize);
        // let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Runtime;
        // let _lock = (*exe).wr_lock.lock();
        // let prio: usize = (*exe).max_prio;

        let max_prio_pid = max_prio_pid();
        if pid != max_prio_pid {
            return true;
        }

        let max_prio_tid = max_prio_tid(pid);
        if tid != max_prio_tid {
            return true;
        }

        return false
    }
}