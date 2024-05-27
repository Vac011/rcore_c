#![allow(unused)]
use crate::{
    task::{add_task, current_task, TaskControlBlock},
    console::*,
};

pub fn sys_coroutine_create(entry: usize, arg: usize) -> isize {
    //create a new coroutine

    // add new task to scheduler

    // add new coroutine to current thread
    println!("\n\ntry\n\n");
    0
    //todo!("Don't Worry!")
}


pub fn sys_getcid() -> isize {
    let mut id: [isize; 3] = [0, 0, 0];
    id[0] = current_task()
            .unwrap()
            .process
            .upgrade()
            .unwrap()
            .getpid() as isize;
    id[1] = current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid as isize;
    id[1] = 0;
    (id[0] << 20) | (id[1] << 10) | id[2]
}
