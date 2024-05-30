use super::{ProcessControlBlock, TaskControlBlock, TaskStatus};
use crate::sync::UPIntrFreeCell;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::*;
use shared;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
/// 可以通过更改fetch将其变为含优先级调度
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        //println!("we did add a threads");
        self.ready_queue.push_back(task);
    }

    // pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
    //     self.ready_queue.pop_front()
    // }

    //fetch coroutines
    pub fn fetch_coroutine(&mut self)->Option<Arc<TaskControlBlock>>{
        let pid = shared::max_prio_pid();
        let tid = shared::max_prio_tid(pid);
        let num = self.ready_queue.len();
        if num == 0 {return None;}
        let mut task;
        let mut cnt = 0;
        // println!("//--------------------------//");
        // println!("num of threads:{}",num);
        // println!("pid to switch:{}",pid);
        // println!("tid to switch:{}",tid);


        loop{
            task = self.ready_queue.pop_front().unwrap();
            let pid_p = task.process.upgrade().unwrap().getpid();
            let tid_p = task.inner_exclusive_access().res.as_ref().unwrap().tid as usize;

            //println!("cnt:{}",cnt);
            // println!("pid_p:{}",pid_p);
            // println!("tid_p:{}",tid_p);

            if pid ==pid_p && tid == tid_p{
                // println!("pid returned_here half way is:{}",pid);
                // println!("tid returned_here half way is:{}",tid);
                
                return Some(task);
            }
            self.ready_queue.push_back(task);
            cnt += 1;
            if cnt >= num {break;}
        }

        task = self.ready_queue.pop_front().unwrap();
        Some(task)
        //self.ready_queue.pop_front()
    }

    #[allow(unused)]
    pub fn prioritize(&mut self, pid: usize) {
        let q = &mut self.ready_queue;
        if q.is_empty() || q.len() == 1 {
            return;
        }
        let front_pid = q.front().unwrap().process.upgrade().unwrap().pid.0;
        // if front_pid == pid {
        //     debug!("[Taskmgr] Task {} already at front", pid);

        //     return;
        // }
        q.rotate_left(1);
        while {
            let f_pid = q.front().unwrap().process.upgrade().unwrap().pid.0;
            f_pid != pid && f_pid != front_pid
        } {
            q.rotate_left(1);
        }
        // if q.front().unwrap().process.upgrade().unwrap().pid.0 == pid {
        //     debug!("[Taskmgr] Prioritized task {}", pid);
        // }
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPIntrFreeCell<TaskManager> =
        unsafe { UPIntrFreeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPIntrFreeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPIntrFreeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
}

// pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
//     TASK_MANAGER.exclusive_access().fetch()
// }

pub fn fetch_task_coroutine() ->Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch_coroutine()
}


pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.exclusive_access().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
