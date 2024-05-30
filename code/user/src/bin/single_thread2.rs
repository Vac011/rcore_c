#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate shared;
#[allow(unused)]

use user_lib::{getpid, gettid};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

async fn coroutine_a() {
    println!("----------------EXECUTE------------------");
}

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine_a())
}
#[no_mangle]
pub fn main() -> i32 {
    // 这么写现在是调度不了的
    //coroutine_create(coroutine_a as usize, 0);
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    let cid = shared::spawn(create_future(), 4, pid, tid, shared::CoroutineKind::UserNorm);
    println!("cid1: {}, prio: 4",cid);
    let cid = shared::spawn(create_future(), 1, pid, tid, shared::CoroutineKind::UserNorm);
    println!("cid2: {}, prio: 1",cid);
    let cid = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
    println!("cid3: {}, prio: 3",cid);
    let cid = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    println!("cid4: {}, prio: 2",cid);
    let cid = shared::spawn(create_future(), 5, pid, tid, shared::CoroutineKind::UserNorm);
    println!("cid5: {}, prio: 5",cid);

    shared::poll_future(pid, tid);
    // let cid3 = shared::demo_spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    // println!("cid3: {}",cid3);
    0
}