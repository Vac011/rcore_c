use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::{sync::Arc, task::Wake};
use core::task::{Waker, Poll, Context};
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;

/// 协程 Id
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd, Default)]
///Cid data struct according to the define of process Id
pub struct CidHandle(pub usize);
impl CidHandle {
    /// 生成新的协程 Id
    pub fn generate() -> CidHandle {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        CidHandle(id)
    }
    /// 根据 usize 生成协程 Id
    pub fn from_val(v: usize) -> Self {
        Self(v)
    }
    /// 获取协程 Id 的 usize
    pub fn get_val(&self) -> usize {
        self.0
    } 
}

// ///Cid allocator according to the allocator of process
// ///https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter8/1thread-kernel.html
// pub struct CidAllocator{
//     ///the max cid
//     current : usize,
//     ///the dealloc cid that can be allocated again
//     recycled : Vec<usize>,
// }

// impl CidAllocator {
//     pub fn new() -> Self {
//         CidAllocator {
//             current: 0,
//             recycled: Vec::new(),
//         }
//     }
//     pub fn alloc(&mut self) -> usize {
//         if let Some(id) = self.recycled.pop() {
//             id
//         } else {
//             self.current += 1;
//             self.current - 1
//         }
//     }
//     pub fn dealloc(&mut self, id: usize) {
//         assert!(id < self.current);
//         assert!(
//             !self.recycled.iter().any(|i| *i == id),
//             "id {} has been deallocated!",
//             id
//         );
//         self.recycled.push(id);
//     }
// }

// ///here is a global initilization but each thread needs one
// /// TODO
// lazy_static! {
//     static ref CidAllocator : UPSafeCell<CidAllocator> = unsafe {
//         UPSafeCell::new(CidAllocator::new())
//     };
// }
// ///interface
// pub fn cid_alloc() -> CidHandle {
//     CidAllocator.exclusive_access().alloc()
// }

// impl Drop for CidHandle {
//     fn drop(&mut self) {
//         CidAllocator.exclusive_access().dealloc(self.0);
//     }
// }

///Coroutine Waker TODO
/// 
struct CoroutineWaker(CidHandle);

impl CoroutineWaker {
    /// 新建协程 waker
    pub fn new(cid: CidHandle) -> Waker {
        Waker::from(Arc::new(Self(cid)))
    }
}

impl Wake for CoroutineWaker {
    fn wake(self: Arc<Self>) { }
    fn wake_by_ref(self: &Arc<Self>) { }
}


pub struct Coroutine{
    ///concerning ID
    pub pid : usize,
    pub tid : usize,
    pub cid : CidHandle,
    ///kind of coroutine to be done
    pub kind : CoroutineKind,
    ///coroutine state
    // state : State,

    ///inner data
    pub inner : Mutex<CoroutineInner>,
}

pub enum CoroutineKind {
    /// 内核调度协程
    KernSche,
    /// 内核系统调用协程
    KernSyscall,
    /// 用户协程
    UserNorm,
}

///an example
// enum State {
//     Available, // 初始态：线程空闲，可被分配一个任务去执行
//     Running,   // 运行态：线程正在执行
//     Ready,     // 就绪态：线程已准备好，可恢复执行
// }


pub struct CoroutineInner{
    ///future
    pub future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, 
    //priority
    pub prio : usize,
    //waker
    pub waker : Arc<Waker>,
}


impl Coroutine{
    /// to construct a coroutine object
    /// par
    /// return
    pub fn new(future: Pin<Box<dyn Future<Output=()> + Send + Sync>>, prio: usize, pid:usize, tid: usize, kind: CoroutineKind) -> Arc<Self> {
        let cid = CidHandle::generate();
        Arc::new(
            Coroutine {
                pid,
                tid,
                cid,
                kind,
                // state,
                inner: Mutex::new(CoroutineInner {
                    future,
                    prio,
                    waker: Arc::new(CoroutineWaker::new(cid)),
                })
                
            }
        )
    }

    /// get cid or tid or pid
    /// par
    /// return
    // pub fn get_cid(){}   这个在cidhandle里可以实现

    /// execute
    /// par
    /// return ready or pending
    pub fn execute(self: Arc<Self>) -> Poll<()> {
        let mut inner = self.inner.lock();
        let waker = inner.waker.clone();
        let mut context = Context::from_waker(&*waker);
        inner.future.as_mut().poll(&mut context)
    }

}
