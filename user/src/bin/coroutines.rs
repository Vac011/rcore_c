#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate lib_so;
#[allow(unused)]
// use lib_so::CoroutineKind::UserNorm;
use user_lib::{coroutine_create, getpid, gettid};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

async fn coroutine_a() {
    // let id = getcid();
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    // let cid = id & 0x3ff;
    println!("The coroutine belongs to pid: {}", pid);
    println!("The coroutine belongs to tid: {}", tid);
    // println!("The coroutine belongs to cid: {}\n", cid);
    let cid = lib_so::demo(tid);
    println!("cid: {}",cid);
    let prio = lib_so::demo1(tid);
    println!("prio: {}",prio);
    let thread = lib_so::demo2(tid);
    println!("thread: {}",thread);
    let bit3 = lib_so::demo3(tid);
    println!("thread: {}",bit3);
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

    let cid1 = lib_so::spawn(create_future(), 3, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid1: {}",cid1);
    let cid2 = lib_so::spawn(create_future(), 4, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid2: {}",cid2);
    let cid3 = lib_so::spawn(create_future(), 2, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid3: {}",cid3);
    let cid4 = lib_so::spawn(create_future(), 5, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid4: {}",cid4);
    let cid5 = lib_so::spawn(create_future(), 2, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid5: {}",cid5);
    coroutine_create(0, 0);
    let cid = lib_so::demo(tid);
    println!("cid: {}",cid);
    lib_so::poll_user_future(pid, tid);
    let cid3 = lib_so::demo_spawn(create_future(), 2, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid3: {}",cid3);
    0
}