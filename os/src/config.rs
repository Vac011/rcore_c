#[allow(unused)]

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const HEAP_BUFFER: usize = TRAMPOLINE - PAGE_SIZE;
pub const TRAP_CONTEXT_BASE: usize = HEAP_BUFFER - PAGE_SIZE;

pub const PROCESS_PRIO_BASE: usize = TRAP_CONTEXT_BASE - PAGE_SIZE*2; // 示例地址，根据需要调整
pub const THREAD_PRIO_BASE: usize = PROCESS_PRIO_BASE - PAGE_SIZE*2; // 示例地址，根据需要调整

pub const BASE_ADDRESS: usize = 0x80200000;

pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};
