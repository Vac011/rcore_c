#![no_std]
#![no_main]
#![allow(unused)]
#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate shared;

use alloc::vec;
use user_lib::{exit, gettid, getpid, thread_create, waittid, sleep};

use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;


async fn coroutine_a() {
    println!("----------------EXECUTE------------------");
}

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(coroutine_a())
}

pub fn thread_a() {
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    println!("a");
    let cid1 = shared::spawn(create_future(), 1, pid, tid, shared::CoroutineKind::UserNorm);
    println!("acid1: {}",cid1);
    let cid2 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    println!("acid2: {}",cid2);
    let cid3 = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
    println!("acid3: {}",cid3);
    let cid4 = shared::spawn(create_future(), 4, pid, tid, shared::CoroutineKind::UserNorm);
    println!("acid4: {}",cid4);
    let cid5 = shared::spawn(create_future(), 5, pid, tid, shared::CoroutineKind::UserNorm);
    println!("acid5: {}",cid5);
    shared::poll_future(pid, tid);
    println!("a finish");
    exit(1)
}

pub fn thread_b() {
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    println!("b");
    let cid1 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    println!("bcid1: {}",cid1);
    let cid2 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    println!("bcid2: {}",cid2);
    let cid3 = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
    println!("bcid3: {}",cid3);
    let cid4 = shared::spawn(create_future(), 4, pid, tid, shared::CoroutineKind::UserNorm);
    println!("bcid4: {}",cid4);
    let cid5 = shared::spawn(create_future(), 5, pid, tid, shared::CoroutineKind::UserNorm);
    println!("bcid5: {}",cid5);
    shared::poll_future(pid, tid);
    println!("b finish");
    exit(2)
}

pub fn thread_c() {
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    println!("c");
    let cid1 = shared::spawn(create_future(), 1, pid, tid, shared::CoroutineKind::UserNorm);
    println!("ccid1: {}",cid1);
    let cid2 = shared::spawn(create_future(), 2, pid, tid, shared::CoroutineKind::UserNorm);
    println!("ccid2: {}",cid2);
    let cid3 = shared::spawn(create_future(), 3, pid, tid, shared::CoroutineKind::UserNorm);
    println!("ccid3: {}",cid3);
    let cid4 = shared::spawn(create_future(), 4, pid, tid, shared::CoroutineKind::UserNorm);
    println!("ccid4: {}",cid4);
    let cid5 = shared::spawn(create_future(), 5, pid, tid, shared::CoroutineKind::UserNorm);
    println!("ccid5: {}",cid5);
    shared::poll_future(pid, tid);
    println!("c finish");
    exit(2)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("main thread exited.");
    // let pid = getpid() as usize;
    // let tid: usize = gettid() as usize;
    // println!("mpid: {}",pid);
    // println!("mtid: {}",tid);
    0
}