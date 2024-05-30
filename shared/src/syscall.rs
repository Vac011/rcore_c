// 调试用
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;

const SYSCALL_YIELD: usize = 124;
const SYSCALL_COROUTINE_CREATE: usize = 4000;
const SYSCALL_YIELD_COROUTINE :usize = 4010;


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

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_coroutine_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_COROUTINE_CREATE, [entry, arg, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_yield_coroutine()->isize{
    syscall(SYSCALL_YIELD_COROUTINE, [0,0,0])
}