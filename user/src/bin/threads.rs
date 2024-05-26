#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
extern crate lib_so;

use alloc::vec;
use user_lib::{exit, getcid, gettid, getpid, thread_create, waittid};

use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

pub fn thread_a() {
    // for _ in 0..1000 {
    //     print!("a");
    // }
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
    lib_so::poll_user_future(pid, tid);
    let cid = lib_so::demo(tid);
    println!("cid: {}",cid);
    // lib_so::poll_user_future(pid, tid);
    // exit(1)
}

pub fn thread_b() {
    // for _ in 0..1000 {
    //     print!("b");
    // }
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    let cid1 = lib_so::spawn(create_future(), 1, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid1: {}",cid1);
    let cid2 = lib_so::spawn(create_future(), 4, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid2: {}",cid2);
    let cid3 = lib_so::spawn(create_future(), 2, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid3: {}",cid3);
    let cid4 = lib_so::spawn(create_future(), 5, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid4: {}",cid4);
    let cid5 = lib_so::spawn(create_future(), 2, pid, tid, lib_so::CoroutineKind::UserNorm);
    println!("cid5: {}",cid5);
    lib_so::poll_user_future(pid, tid);
    let cid = lib_so::demo(tid);
    println!("cid: {}",cid);
    // lib_so::poll_user_future(pid, tid);
    // exit(2)
}

pub fn thread_c() {
    // for _ in 0..1000 {
    //     print!("c");
    // }
    let id = getcid();
    let pid = id >> 20;
    let tid = (id >> 10) & 0x3ff;
    let cid = id & 0x3ff;
    println!("\nThe coroutine belongs to pid: {}\n", pid);
    println!("The coroutine belongs to tid: {}\n", tid);
    println!("The coroutine belongs to cid: {}\n", cid);
    
    // exit(3)
}

async fn coroutine_a() {
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
    let pid = getpid() as usize;
    let tid: usize = gettid() as usize;
    println!("mpid: {}",pid);
    println!("mtid: {}",tid);
    0
}
