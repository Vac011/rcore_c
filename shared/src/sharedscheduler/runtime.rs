///import concerning package
use alloc::collections::{BTreeMap, VecDeque, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use super::{
    coroutine::{Coroutine, CidHandle, CoroutineKind},
    BitMap,
};
use alloc::boxed::Box;
use core::ops::Add;
use core::pin::Pin;
use core::future::Future;
use crate::config::{MAX_THREAD_NUM, PRIO_NUM};
use crate::println;
// use crate::MAX_COR_NUM;
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
    // pub ready_queue: [BTreeSet<CidHandle>; MAX_THREAD_NUM * PRIO_NUM],
    pub ready_queue: [BTreeSet<CidHandle>; MAX_THREAD_NUM * PRIO_NUM],
    // pub ready_queue: Vec<Vec<VecDeque<CidHandle>>>,
    /// 阻塞协程集合
    pub pending_set: [BTreeSet<usize>; MAX_THREAD_NUM],
    /// 协程优先级位图
    pub bitmap: BitMap,
    pub threadmap: [BitMap; MAX_THREAD_NUM],
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
    pub const fn new(busy_wait: bool) -> Self {
        // const READY_QUEUE_VALUE: Vec<VecDeque<CidHandle>> = Vec::new();
        const PENDING_SET_VALUE: BTreeSet<usize> = BTreeSet::new();
        const READY_SET_VALUE: BTreeSet<CidHandle> = BTreeSet::new();
        Self {
            currents: [None; MAX_THREAD_NUM],
            tasks: BTreeMap::new(),
            // ready_queue: [READY_QUEUE_VALUE; MAX_THREAD_NUM],
            ready_queue: [READY_SET_VALUE; MAX_THREAD_NUM * PRIO_NUM],
            // ready_queue: Vec::new(),
            pending_set: [PENDING_SET_VALUE; MAX_THREAD_NUM],
            bitmap: BitMap(0),
            threadmap: [BitMap(0); MAX_THREAD_NUM],
            max_prio: PRIO_NUM,
            thread_prio: [PRIO_NUM; MAX_THREAD_NUM],
            wr_lock: RunMutex::new(busy_wait),
            waits: Vec::new(),
        }
    }

    ///new a coroutine and add it to the queue
    /// par
    /// return
    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid:usize, tid: usize, kind: CoroutineKind) -> usize {
        let task = Coroutine::new(future, prio, pid, tid, kind);
        let cid = task.cid;
        let lock = self.wr_lock.lock();
        // self.ready_queue[tid][prio].push_back(cid);
        self.ready_queue[tid * PRIO_NUM + prio -1].insert(cid);
        self.tasks.insert(cid, task);
        self.bitmap.update(prio, true);
        self.threadmap[tid].update(prio, true);
    
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
    pub fn del_coroutine(&mut self, tid: usize, cid: CidHandle){
        let lock = self.wr_lock.lock();
        let task = self.tasks.get(&cid).unwrap();
        let p = task.inner.lock().prio;
        self.ready_queue[tid * PRIO_NUM + p - 1].remove(&cid);
        if self.ready_queue[tid * PRIO_NUM + p - 1].is_empty() {
            //刷新位图
            self.threadmap[tid].update(p, false);
        }
        let mut empty = true;
        for i in 0..MAX_THREAD_NUM {
            empty = empty && self.ready_queue[i * PRIO_NUM + p -1].is_empty();
        }
        if empty {
            self.bitmap.update(p, false);
        }
        self.max_prio = self.bitmap.get_priority();
        self.thread_prio[tid] = self.threadmap[tid].get_priority();
        self.tasks.remove(&cid);
        drop(lock);
    }

    ///to add the coroutine to the pending queue
    /// par
    /// return
    pub fn pending_coroutine(&mut self, tid: usize, cid: CidHandle){
        let _lock = self.wr_lock.lock();
        let task = self.tasks.get(&cid).unwrap();
        let p = task.inner.lock().prio;
        self.ready_queue[tid * PRIO_NUM + p -1].remove(&cid);
        if self.ready_queue[tid * PRIO_NUM + p].is_empty() {
            //刷新位图
            self.threadmap[tid].update(p, false);
        }
        let mut empty = true;
        for i in 0..MAX_THREAD_NUM {
            empty = empty && self.ready_queue[i * PRIO_NUM + p -1].is_empty();
        }
        if empty {
            self.bitmap.update(p, false);
        }
        self.max_prio = self.bitmap.get_priority();
        self.thread_prio[tid] = self.threadmap[tid].get_priority();
        // self.tasks.remove(&cid);
        self.pending_set[tid].insert(cid.0);
        // self.tasks.remove(&CidHandle(cid));
    }

    ///pull the pending coroutine to the ready queue
    /// par
    /// ret
    pub fn pull_to_ready(&mut self, tid: usize, cid: CidHandle) -> usize{
        let lock = self.wr_lock.lock();
        let prio = self.tasks.get(&cid).unwrap().inner.lock().prio;
        // let tid = gettid() as usize;
        // self.ready_queue[tid][prio].push_back(cid);
        self.ready_queue[tid * PRIO_NUM + prio -1].insert(cid);
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
            
            let mut cid = CidHandle(1);
            if let Some(btree) = self.ready_queue.get(tid * PRIO_NUM + thread_p -1) {
                if let Some(first_cid) = btree.iter().next() {
                    cid = *first_cid;
                }
            }
            
            // let cid = *self.ready_queue.get(tid * PRIO_NUM + thread_p);
            // let task = (*self.tasks.get(&cid).unwrap()).clone();
            let mut task: Option<Arc<Coroutine>> = None;
            if let Some(ttask) = self.tasks.get(&cid) {
                task = Some(Arc::clone(ttask));
                // 在这里使用 task_clone
            }
            drop(_lock);
            self.currents[tid] = Some(cid);
            task
        }
    }
}
