use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::{sync::Arc, task::Wake};
use core::task::{Waker, Poll, Context};
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use crate::println;
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

///Coroutine Waker TODO
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
    pub fn execute(self: Arc<Self>) -> Poll<()> {
        let mut inner = self.inner.lock();
        let waker = inner.waker.clone();
        let mut context = Context::from_waker(&*waker);
        if self.should_block() {
            return Poll::Pending;
        }
        inner.future.as_mut().poll(&mut context)
    }

    fn should_block(&self) -> bool {
        // 在此处添加阻塞条件
        // self.cid.get_val() == 1
        false
    }

}
