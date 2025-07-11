#![no_std]
#![no_main]
#[allow(unused)]
#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate shared;
use user_lib::get_time;
use user_lib::{getpid, gettid};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

async fn coroutine_a() {
    // println!("----------------EXECUTE------------------");
}

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine_a())
}
#[no_mangle]
pub fn main() -> i32 {
    let current_time = get_time();
    let number = 2048;
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    for _i in 0..number {
        shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);    
    }
    shared::poll_future(pid, tid);

    println!("use {} msecs.", get_time() - current_time);
    0
}