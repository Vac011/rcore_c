///import concerning package
use alloc::collections::{BTreeMap, VecDeque, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
// use syscall::yield_;
use super::{
    coroutine::{Coroutine, CidHandle, CoroutineKind},
    BitMap,
};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use crate::config::{MAX_THREAD_NUM, PRIO_NUM};
use core::task::Poll;

///rCore-N中实现了锁  TODO
pub struct RunMutex {
    mutex: Mutex<()>,
    busy_wait: bool,
}

impl RunMutex {
    pub const fn new(busy_wait: bool) -> Self {
        RunMutex { mutex: Mutex::new(()), busy_wait, }
    }

    pub fn lock(&mut self) -> spin::MutexGuard<'_, ()> {
        if self.busy_wait {
            self.mutex.lock()
        } else {
            loop {
                let mut op_lock = self.mutex.try_lock();
                if op_lock.is_some() {
                    return op_lock.unwrap();
                }
                // yield_();
            }
            
        }
    }
}

///Runtime data struct, to distinguish from rCore-N, rename it as Runtime
/// 里面很多数据结构看不太懂，得仔细看看
pub struct Runtime{
    /// 当前正在运行的协程 Id
    pub currents: [Option<CidHandle>; MAX_THREAD_NUM],
    /// 协程 map
    pub tasks: BTreeMap<CidHandle, Arc<Coroutine>>,
    /// 就绪协程队列
    pub ready_queue: [Vec<VecDeque<CidHandle>>; MAX_THREAD_NUM],
    /// 阻塞协程集合
    pub pending_set: [BTreeSet<usize>; MAX_THREAD_NUM],
    /// 协程优先级位图
    pub bitmap: BitMap,
    /// 进程最高优先级协程代表的优先级，内核可以直接访问物理地址来读取
    pub max_prio: usize,
    ///线程
    pub thread_prio: [usize; MAX_THREAD_NUM],
    /// 整个 Executor 的读写锁，内核读取 priority 时，可以不获取这个锁，在唤醒协程时，需要获取锁
    pub wr_lock: RunMutex,
    /// 执行器线程id
    pub waits: Vec<usize>,
    
} 



impl Runtime{
    ///to construct the runtime......like executor
    /// par
    /// return
    pub const fn new(busy_wait: bool) -> Self {
        const READY_QUEUE_VALUE: Vec<VecDeque<CidHandle>> = Vec::new();
        const PENDING_SET_VALUE: BTreeSet<usize> = BTreeSet::new();
        Self {
            currents: [None; MAX_THREAD_NUM],
            tasks: BTreeMap::new(),
            ready_queue: [READY_QUEUE_VALUE; MAX_THREAD_NUM],
            pending_set: [PENDING_SET_VALUE; MAX_THREAD_NUM],
            bitmap: BitMap(0),
            max_prio: PRIO_NUM,
            thread_prio: [PRIO_NUM; MAX_THREAD_NUM],
            wr_lock: RunMutex::new(busy_wait),
            waits: Vec::new(),
        }
    }

    ///new a coroutine and add it to the queue
    /// par
    /// return
    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, pid: usize, tid:usize, prio: usize, kind: CoroutineKind) -> usize {
        let task = Coroutine::new(future, pid, tid, prio, kind);
        let cid = task.cid;
        // let tid = gettid();
        let lock = self.wr_lock.lock();
        self.ready_queue[tid][prio].push_back(cid);
        self.tasks.insert(cid, task);
        self.bitmap.update(prio, true);
        if prio < self.max_prio {
            self.max_prio = prio;
        }
        if prio < self.thread_prio[tid] {
            self.thread_prio[tid] = prio;
        }
        drop(lock);
        return cid.0;
    }

    ///delete the coroutine and drop it from the queue
    /// par
    /// return
    pub fn del_coroutine(&mut self, cid: CidHandle){
        let lock = self.wr_lock.lock();
        self.tasks.remove(&cid);
        drop(lock);
    }

    ///to add the coroutine to the pending queue
    /// par
    /// return
    pub fn pending_coroutine(&mut self, tid:usize, cid: usize){
        let _lock = self.wr_lock.lock();
        self.pending_set[tid].insert(cid);
    }

    ///pull the pending coroutine to the ready queue
    /// par
    /// ret
    pub fn pull_to_ready(&mut self, tid: usize, cid: CidHandle) -> usize{
        let lock = self.wr_lock.lock();
        let prio = self.tasks.get(&cid).unwrap().inner.lock().prio;
        // let tid = gettid() as usize;
        self.ready_queue[tid][prio].push_back(cid);
        self.bitmap.update(prio, true);
        if prio < self.thread_prio[tid] {
            self.thread_prio[tid] = prio;
        }
        if prio < self.max_prio {
            self.max_prio = prio;
        }
        
        self.pending_set[tid].remove(&cid.0);
        drop(lock);
        self.thread_prio[tid]
    }

    ///to tell weather the coroutine is in the pending queue
    /// par
    /// return
    pub fn is_pending(&mut self, tid: usize, cid: usize) -> bool {
        let _lock = self.wr_lock.lock();
        self.pending_set[tid].contains(&cid)
    }

    ///to tell weather the coroutine queue is empty
    /// par
    /// return
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// get the coroutine to run according to its prio
    /// par
    /// ret
    pub fn fetch_coroutine(&mut self, tid: usize) -> Option<Arc<Coroutine>> {
        assert!(tid < MAX_THREAD_NUM);
        let _lock = self.wr_lock.lock();
        let prio = self.max_prio;
        if prio == PRIO_NUM {
            self.currents[tid] = None;//防止取到主协程
            None
        } else {
            let thread_p = self.thread_prio[tid];
            let cid = self.ready_queue[tid][thread_p].pop_front().unwrap();
            let task = (*self.tasks.get(&cid).unwrap()).clone();
            if self.ready_queue[tid][thread_p].is_empty() {
                self.bitmap.update(prio, false);
                self.max_prio = self.bitmap.get_priority();
            }//TODO bitmap的更新还没想好
            drop(_lock);
            self.currents[tid] = Some(cid);
            Some(task)
        }
    }

    /// coroutine schedule in an single process
    /// par
    /// ret
    pub fn coroutine_run(&mut self, tid: usize){
        let task = self.fetch_coroutine(tid as usize);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    // println!("user task kind {:?}", task.kind);
                    match task.execute() {
                        Poll::Pending => {
                            self.pending_coroutine(tid, cid.0);
                        }
                        Poll::Ready(()) => {
                            self.del_coroutine(cid);
                        }
                    };
                }//如果获取到了任务，则执行该任务。如果任务处于挂起状态，则将其标记为挂起状态；如果任务已完成，则删除该协程。然后更新进程的最高优先级。
                _ => {
                    // 任务队列不为空，但就绪队列为空，等待任务唤醒.如果没有获取到任务，则让出 CPU 执行权。
                    // yield_();
                }
            }
    }
}


// impl Runtime {
//     ///coroutine switch   TODO???   this is a schedule maybe
//     /// https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter8/1thread.html
//     /// here is an easy example
//     /// par
//     /// return
//     fn t_yield(&mut self) -> bool {
//         let mut pos = self.current;
//         while self.tasks[pos].state != State::Ready {
//             pos += 1;
//             if pos == self.tasks.len() {
//                 pos = 0;
//             }
//             if pos == self.current {
//                 return false;
//             }
//         }

//         if self.tasks[self.current].state != State::Available {
//             self.tasks[self.current].state = State::Ready;
//         }

//         self.tasks[pos].state = State::Running;
//         let old_pos = self.current;
//         self.current = pos;

//         unsafe {
//             switch(&mut self.tasks[old_pos].ctx, &self.tasks[pos].ctx);
//         }
//         self.tasks.len() > 0
//     }
// }
