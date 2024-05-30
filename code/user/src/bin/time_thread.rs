#![no_std]
#![no_main]
#![allow(unused)]
#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use user_lib::{exit, gettid, getpid, thread_create, waittid, sleep, get_time};

pub fn thread() {
    // println!("----------------EXECUTE------------------");
    exit(1);
}
#[no_mangle]
pub fn main() -> i32 {
    let current_time = get_time();
    const NUM_THREADS: usize = 2048; // 定义要创建的线程数量
    let mut v = Vec::with_capacity(NUM_THREADS); // 创建一个预先分配好容量的向量
    for i in 0..NUM_THREADS {
        v.push(thread_create(thread as usize, 0));
    }

    for tid in v.iter() {
        // waittid(*tid as usize);
        waittid(*tid as usize);
        // println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("use {} msecs.", get_time() - current_time);
    // println!("main thread exited.");
    // let pid = getpid() as usize;
    // let tid: usize = gettid() as usize;
    // println!("mpid: {}",pid);
    // println!("mtid: {}",tid);
    0
}