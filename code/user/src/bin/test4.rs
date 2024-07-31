//! 对比rCore-Tutorial-v3任务切换性能测试
#![no_std]
#![no_main]
#[allow(unused)]
#[macro_use]
extern crate alloc;
extern crate shared;

use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;
use user_lib::{gettid, getpid, get_time};
use shared::println;
const MAX_TASK: usize = 1000;

async fn a() {}
fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(a())
}
// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    let current_time = get_time();
    let number = MAX_TASK;
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    for i in 0..number {
        shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);    
    }
    shared::poll_future(pid, tid);
    println!("use {} msecs.", get_time() - current_time);
    0
}
