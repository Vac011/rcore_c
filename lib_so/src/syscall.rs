// 调试用
const SYSCALL_YIELD: usize = 124;
const SYSCALL_COROUTINE_CREATE: usize = 4000;
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_coroutine_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_COROUTINE_CREATE, [entry, arg, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}