#![no_std]
#![no_main]
#![allow(unused)]
#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate shared;

use alloc::vec;
use user_lib::{exit, sleep, fork, thread_create, waittid, getpid, gettid};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

async fn coroutine() {
    println!("----------------EXECUTE------------------");
}

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine())
}

#[no_mangle]
pub fn main() -> i32 {
    let pid = fork();
    // 子进程
    if pid == 0 {
        let pid = getpid() as usize;
        let tid = gettid() as usize;
        
        let cid1 = shared::spawn(create_future(), 1, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid1: {}",cid1);
        let cid1 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid2: {}",cid1);
        let cid1 = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid3: {}",cid1);
        sleep(1000);
        shared::poll_future(pid, tid);
    }
    // 父进程
    else {
        let pid = getpid() as usize;
        let tid = gettid() as usize;
        
        let cid1 = shared::spawn(create_future(), 1, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid1: {}",cid1);
        let cid1 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid2: {}",cid1);
        let cid1 = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
        println!("cid3: {}",cid1);
        sleep(1000);
        shared::poll_future(pid, tid);
    }
    0
}