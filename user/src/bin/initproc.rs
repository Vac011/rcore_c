#![no_std]
#![no_main]

extern crate user_lib;
extern crate alloc;
extern crate shared;
use user_lib::{exec, fork, wait, yield_, getpid, gettid};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;

async fn main_coroutine() {
    panic!("into the main coroutine");
}

fn create_future() -> Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> {
    Box::pin(main_coroutine())
}

#[no_mangle]
fn main() -> i32 {
    let pid = getpid() as usize;
    let tid = gettid() as usize;
    shared::spawn(create_future(), shared::PRIO_NUM, pid, tid, shared::CoroutineKind::UserNorm);
    if fork() == 0 {
        exec("user_shell\0", &[core::ptr::null::<u8>()]);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            /*
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid,
                exit_code,
            );
            */
        }
    }
    0
}
