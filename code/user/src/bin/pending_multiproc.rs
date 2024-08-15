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

async fn coroutine_a() {
    println!("----------------EXECUTE------------------");
    for i in 1..1000{
        print!("a")
    }
}

fn create_futurea() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine_a())
}

async fn coroutine_b() {
    println!("----------------EXECUTE------------------");
    for i in 1..1000{
        print!("b")
    }
}

fn create_futureb() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine_b())
}

#[no_mangle]
pub fn main() -> i32 {
    let pid = fork();
    // 子进程
    if pid == 0 {
        let pid = getpid() as usize;
        let tid = gettid() as usize;
        
        let cid1 = shared::spawn(create_futurea(), 3, pid, tid, shared::CoroutineKind::UserNorm);
        println!("child_cid1: {}, prio: 3",cid1);
        let cid1 = shared::spawn(create_futurea(), 4, pid, tid, shared::CoroutineKind::UserNorm);
        println!("child_cid2: {}, prio: 4",cid1);
        let cid1 = shared::spawn(create_futurea(), 3, pid, tid, shared::CoroutineKind::UserNorm);
        println!("child_cid3: {}, prio: 3",cid1);
        // sleep(1000);
        shared::poll_future(pid, tid);
    }
    // 父进程
    else {
        let pid = getpid() as usize;
        let tid = gettid() as usize;
        
        let cid1 = shared::spawn(create_futureb(), 1, pid, tid, shared::CoroutineKind::UserNorm);
        println!("parent_cid1: {}, prio: 1",cid1);
        let cid1 = shared::spawn(create_futureb(), 4, pid, tid, shared::CoroutineKind::UserNorm);
        println!("parent_cid2: {}, prio: 4",cid1);
        let cid1 = shared::spawn(create_futureb(), 3, pid, tid, shared::CoroutineKind::UserNorm);
        println!("parent_cid3: {}, prio: 3",cid1);
        // sleep(1000);
        sleep(3);
        shared::poll_future(pid, tid);
        println!("\n main process exited");
    }
    0
}