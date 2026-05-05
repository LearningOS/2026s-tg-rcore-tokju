//! 处理器管理模块
//!
//! 定义 `PROCESSOR` 全局变量和 `ProcManager` 进程管理器。
//!
//! ## 设计思路
//!
//! 进程管理分为两部分：
//! - `PROCESSOR`：封装 `PManager`，提供全局访问接口，管理当前运行的进程
//! - `ProcManager`：实现 `Manage` 和 `Schedule` trait，负责进程的存储和调度
//!
//! ## 调度算法
//!
//! 使用 **stride 调度算法**：
//! - 每个进程有 stride 和 priority 字段
//! - pass = BIG_STRIDE / priority
//! - 每次调度选择 stride 最小的进程
//! - 调度后 stride += pass

use crate::process::Process;
use alloc::collections::BTreeMap;
use core::cell::UnsafeCell;
use tg_task_manage::{Manage, PManager, ProcId, Schedule};

/// 处理器全局管理器
///
/// 封装 `PManager<Process, ProcManager>`，通过 `UnsafeCell` 提供内部可变性。
/// 在单核环境下是安全的，因为不会出现并发访问。
pub struct Processor {
    inner: UnsafeCell<PManager<Process, ProcManager>>,
}

unsafe impl Sync for Processor {}

impl Processor {
    /// 创建新的处理器管理器（编译期常量初始化）
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(PManager::new()),
        }
    }

    /// 获取内部 PManager 的可变引用
    #[inline]
    pub fn get_mut(&self) -> &mut PManager<Process, ProcManager> {
        unsafe { &mut (*self.inner.get()) }
    }
}

/// 全局处理器管理器实例
pub static PROCESSOR: Processor = Processor::new();

/// BigStride 常量，用于 stride 调度算法
/// 选择一个较大的值以减少除法误差，同时避免溢出
const BIG_STRIDE: u64 = 1 << 30;

/// 进程管理器
///
/// 负责管理所有进程实体和调度队列：
/// - `tasks`：以 ProcId 为键的进程映射表，存储所有进程实体
/// - `ready_queue`：就绪队列，存储等待执行的进程 PID
///
/// 使用 stride 调度算法。
pub struct ProcManager {
    /// 所有进程实体的映射表
    tasks: BTreeMap<ProcId, Process>,
    /// 就绪队列（stride 调度）
    ready_queue: alloc::vec::Vec<ProcId>,
}

impl ProcManager {
    /// 创建新的进程管理器
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: alloc::vec::Vec::new(),
        }
    }

    /// 计算进程的 pass 值
    fn calc_pass(priority: u64) -> u64 {
        BIG_STRIDE / priority
    }

    /// 找到就绪队列中 stride 最小的进程索引
    fn find_min_stride_idx(&self) -> Option<usize> {
        if self.ready_queue.is_empty() {
            return None;
        }
        let mut min_idx = 0;
        let mut min_stride = u64::MAX;
        for (i, pid) in self.ready_queue.iter().enumerate() {
            if let Some(proc) = self.tasks.get(pid) {
                if proc.stride < min_stride {
                    min_stride = proc.stride;
                    min_idx = i;
                }
            }
        }
        Some(min_idx)
    }
}

/// 实现 Manage trait：进程实体的增删查
impl Manage<Process, ProcId> for ProcManager {
    /// 插入新进程到进程表
    #[inline]
    fn insert(&mut self, id: ProcId, task: Process) {
        self.tasks.insert(id, task);
    }

    /// 根据 PID 获取进程的可变引用
    #[inline]
    fn get_mut(&mut self, id: ProcId) -> Option<&mut Process> {
        self.tasks.get_mut(&id)
    }

    /// 从进程表中删除进程（回收资源）
    #[inline]
    fn delete(&mut self, id: ProcId) {
        self.tasks.remove(&id);
    }
}

/// 实现 Schedule trait：进程调度（stride 调度算法）
impl Schedule<ProcId> for ProcManager {
    /// 将进程加入就绪队列
    fn add(&mut self, id: ProcId) {
        self.ready_queue.push(id);
    }

    /// 取出下一个要执行的进程（stride 最小的）
    fn fetch(&mut self) -> Option<ProcId> {
        let idx = self.find_min_stride_idx()?;
        let pid = self.ready_queue.remove(idx);

        // 更新该进程的 stride
        if let Some(proc) = self.tasks.get_mut(&pid) {
            let pass = Self::calc_pass(proc.priority);
            proc.stride = proc.stride.wrapping_add(pass);
        }

        Some(pid)
    }
}
