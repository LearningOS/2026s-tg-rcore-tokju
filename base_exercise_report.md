# 基础练习报告

## 与AI合作的过程和记录

- 每次让AI做一个章节的练习，虽然比较慢，但胜在稳扎稳打，人类有很强的控制能力。如果让AI一下做完全部，人类很难控制，出现问题没有及时发现就会出现严重后果。
- 沙箱环境运行`./test.sh`出现`tee /dev/stderr: No device or address`的问题，有些AI直接忽视，因为全部都PASS了，即使最后输出是测试失败。有些AI会尝试解决，但试了几次就不管了。人为排查问题原来是在沙箱环境运行bash脚本时是无法访问`/dev/stderr`的，故后续给AI的prompt都会带上这个提示
- 有些强的AI会自动运行`./test.sh`来验证，比如Kimi-K2.6, DeepSeek-v4-pro; 弱的AI就只会运行成功后就说停止了，没有运行`./test.sh`来验证输出，比如DeepSeek-v4-flash, 对于弱的AI的必须在prompt里来提醒。
- `./test.sh`FAIL输出比较歧义，`./test.sh`失败时总结输出`Test FAILED: 4/33`会使AI误以为有4个FAIL，但是其实是`33-4`个FAIL，这让AI陷入思考：“FAILED的为什么是4，明明PASSED的个数是4，等等，或者是其他意思...”，我以为思考一次就可以了，没想到，每次都要思考这个有歧义的输出。虽然这并没有什么影响，最后还是通过了，输出`Test PASSED: 33/33`，但这十分消耗token。比较简单的解决方式是在prompt里提醒，最好的办法是改掉总结输出的的文字，或者干脆就不要总结输出，现在AI还是很强的，自己就可以数数，而且AI写代码也用不上这些数字。
- lsp不能触发，发现是`This file is not include any crates`的问题，所以lsp不能触发，lsp不触发，那么就不能及时发现一些问题，解决问题。我发现无论什么样的AI，都会在`no use varible`和`the varible is never change, use const`上出问题，不能一下就写出没有这些问题的代码，这些问本可以用lsp来及时发现，但lsp不触发，导致都是在编译时才发现，十分影响效率。解决方法就是手动把这些目录写进Cargo.toml的workspace里，scripts/crates.txt提供了全部crates的目录，直接复制即可。但是这样又会在运行时warning,说：“warning: profiles for the non root package will be ignored, specify profiles at the workspace root:”，但这影响不大。
- 在ch3练习上，AI完全不改main.rs，只改task.rs，导致AI并没看到main.rs里的TODO，虽然最后还是通过了测试，但代码质量就有点差强人意，main.rs里面还留着TODO。后续许多章节也是如此，但是后续章节比较复杂，测试运行不能一次成功，AI才看到了TODO，才发现自己之前写的方式有问题，并把这个TODO完成。这也只能自己去提醒，注意看代码里的TODO。

## 学习收获

### ch1

- `#![no_std]` 不链接标准库，裸机无操作系统支持，需自行实现所需功能（`tg-rcore-tutorial-sbi`已提供最小支持）；`#![no_main]` 不使用标准main入口，必须自定义入口点`_start`。
- 裸机入口`_start`需手动设栈，因为无标准运行时为其初始化栈指针；控制流为`_start`（设置sp）→ `rust_main`（输出Hello world并shutdown）→ `panic_handler`（异常时shutdown）。
- `tg-rcore-tutorial-sbi`在本章的最小职责：通过`console_putchar`输出字符，通过`shutdown`关机。

### ch2

- `U → S`：用户程序执行`ecall`或发生异常时CPU自动从U mode陷入S mode；`S → U`：内核执行`sret`指令返回U mode。
- `sepc += 4`：`ecall`指令占4字节，若不跳过，`sret`后会再次执行`ecall`导致无限循环。
- syscall参数来源：`a0~a5`传递参数，`a7`传递syscall号；返回值通过`a0`传回。

### ch3

- 抢占式调度：时钟中断强制切换，输出呈现交替（如power_3与power_5交错输出）；协作式调度：依赖程序主动`yield`让出CPU，持续运行一个任务直到其主动退出。
- `TaskControlBlock`封装了上下文（`LocalContext`，保存用户寄存器）、用户栈（独立8KiB栈空间）和完成状态（`finish`），实现多任务独立管理。
- Trap分支中`SupervisorTimer`是时钟中断，由硬件定时触发；`UserEnvCall`是用户程序`ecall`指令，由软件主动触发。

### ch4

- 引入Sv39虚拟内存的原因：用户程序直接使用物理地址存在安全性（可访问内核数据）、隔离性（程序bug可破坏其他程序内存）和灵活性（无法重定位）问题，每个进程需要独立地址空间。
- 内核恒等映射（虚拟地址==物理地址）使得内核可无障碍访问任意物理内存；用户地址空间映射实现进程间隔离；传送门映射（同一物理页映射到内核和所有用户空间的相同虚拟地址）解决跨地址空间切换时`satp`变更导致取指崩溃的问题。
- `translate()`在syscall中：通过进程页表将用户虚拟地址翻译为物理地址，同时检查PTE权限标志（R/W/X + U），权限不足返回`None`。
- `sbrk`扩容时按页映射新物理页，缩容时取消映射物理页，`program_brk`的增减不改变页映射范围直至跨越页边界。

### ch5

- `fork`深拷贝父进程地址空间创建子进程，父进程返回子进程PID，子进程返回0；`exec`清空当前地址空间加载新ELF，PID不变；`waitpid`等待子进程退出并回收资源。
- 父子进程关系通过`PManager`维护，子进程退出进入Zombie态保留PCB和退出码，父进程`waitpid`后删除PCB；若父进程先退出，子进程挂到`initproc`下由其回收。
- `initproc`是内核创建的第一个用户进程，其`fork`出`user_shell`子进程；`initproc`自身循环`wait`回收孤儿进程。

### ch6

- 核心变化：用户程序从内嵌镜像（`APP_ASM`）迁移到磁盘镜像（`fs.img`），实现内核与用户程序解耦。
- VirtIO MMIO映射是文件系统可用前提：内核需在地址空间中映射MMIO地址`0x10001000`，才能通过VirtIO驱动与块设备通信，从而读写磁盘数据。
- `open`→easy-fs打开文件→分配fd插入`fd_table`；`read/write`→查`fd_table`获取`FileHandle`→调用对应方法；`close`→`fd_table[fd]=None`。

### ch7

- 统一`Fd`枚举（`File`/`PipeRead`/`PipeWrite`/`Empty`）使文件、管道、标准I/O共享同一套`read/write`接口，系统调用无需关心fd的具体类型。
- `pipe`创建一对fd（读端+写端），`fork`后父子进程各持两端，各自关闭不需要的一端后通过管道单向通信（写端写入环形缓冲区，读端从环形缓冲区读取）。
- 信号四个关键syscall：`kill`发送信号（加入目标进程的`received`位图），`sigaction`注册/查询信号处理函数，`sigprocmask`更新信号屏蔽字，`sigreturn`从信号处理函数恢复原执行流。
- "syscall返回前检查信号"：若检测到`SIGKILL`等终止信号则进程退出，否则调用注册的信号处理函数，使得信号处理时机与用户态执行流自然衔接。

### ch8

- `Process`是资源容器（地址空间、fd_table、同步原语列表），`Thread`是执行单元（TID、上下文、用户栈），一个Process可对应多个Thread，共享资源。
- `PThreadManager`维护双层关系：`ThreadManager`管理Thread实体和就绪队列（调度单位），`ProcManager`管理Process实体（父子关系、资源回收）。
- `thread_create`在当前进程中创建新线程（分配用户栈、设置入口参数）；`gettid`返回当前线程TID；`waittid`等待指定线程退出并返回退出码。
- 阻塞原语路径：资源不可用时`mutex_lock`/`semaphore_down`/`condvar_wait`返回-1→主循环调用`make_current_blocked()`将线程移出就绪队列→资源释放后`unlock`/`up`/`signal`返回被唤醒的TID→`re_enque`将线程重新加入就绪队列。
