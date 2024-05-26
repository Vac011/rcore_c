#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec;
use user_lib::{exit, thread_create, waittid, getcid, sleep};

pub fn thread_a() -> ! {
    for _ in 0..1000 {
        print!("a");
    }
    exit(1)
}

pub fn thread_b() -> ! {
    for _ in 0..1000 {
        print!("b");
    }
    exit(2)
}

pub fn thread_c() -> ! {
    for _ in 0..1000 {
        print!("c");
    }
    let id = getcid();
    let pid = id >> 20;
    let tid = (id >> 10) & 0x3ff;
    let cid = id & 0x3ff;
    println!("\nThe coroutine belongs to pid: {}\n", pid);
    println!("The coroutine belongs to tid: {}\n", tid);
    println!("The coroutine belongs to cid: {}\n", cid);
    exit(3)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    // for tid in v.iter() {
    //     let exit_code = waittid(*tid as usize);
    //     println!("thread#{} exited with code {}", tid, exit_code);
    // }
    sleep(10000);
    println!("main thread exited.");
    0
}
