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
use alloc::sync::Arc;

async fn coroutine_a() {
    // let id = getcid();
    let pid = getpid();
    let tid = gettid();
    // let cid = id & 0x3ff;
    println!("The coroutine belongs to pid: {}\n", pid);
    println!("The coroutine belongs to tid: {}\n", tid);
    // println!("The coroutine belongs to cid: {}\n", cid);
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
    println!("pid: {}",pid);
    println!("tid: {}",tid);
    let demo1 = lib_so::max_prio_pid();
    println!("{}", demo1);
    let demo2 = lib_so::max_prio_tid(pid);
    println!("{}", demo2);
    let demo3 = lib_so::demo();
    println!("{}", demo3);
    // let prio = lib_so::max_prio_pid();
    // println!("prio: {}",prio);
    let cid = lib_so::spawn(create_future(), 3, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid: {}",cid);
    0
}