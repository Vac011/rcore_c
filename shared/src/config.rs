/// 页面大小
pub const PAGE_SIZE: usize = 0x1000;
/// 跳板页虚拟地址
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
/// 用户态中断虚拟地址
// pub const USER_TRAP_BUFFER: usize = TRAMPOLINE - PAGE_SIZE;
/// 共享调度器使用的数据所在的虚拟地址，在这个位置记录了用户程序堆的虚拟地址
/// 在共享代码中操作不同进程的堆和 Executor 主要是读取这个虚拟地址中保存的用户程序堆 heap 的虚拟地址
/// 再来进行分配
// pub const HEAP_BUFFER: usize = USER_TRAP_BUFFER - PAGE_SIZE;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub const BASE_ADDRESS: usize = 0x80200000;
pub const HEAP_BUFFER: usize = BASE_ADDRESS - PAGE_SIZE;


pub const PROCESS_PRIO_BASE: usize = HEAP_BUFFER - PAGE_SIZE*2; // 示例地址，根据需要调整
pub const THREAD_PRIO_BASE: usize = PROCESS_PRIO_BASE - PAGE_SIZE*2; // 示例地址，根据需要调整
/// 用户程序入口
pub const ENTRY: usize = 0x1000;
/// CPU数量 + 用户态中断处理线程
pub const MAX_THREAD_NUM: usize = 15;

/// 协程支持的优先级数目
pub const PRIO_NUM: usize = 8;
/// 支持的最大进程数量
pub const MAX_PROC_NUM: usize = 30;



