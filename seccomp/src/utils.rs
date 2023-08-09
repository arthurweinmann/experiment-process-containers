use super::{
    allow_syscall, allow_syscall_if, BpfProgram, Error, SeccompAction, SeccompCmpArgLen as ArgLen,
    SeccompCmpOp::Eq, SeccompCondition as Cond, SeccompFilter, SeccompRule, SyscallRuleSet,
};

/// Shorthand for chaining `SeccompCondition`s with the `and` operator  in a `SeccompRule`.
/// The rule will take the `Allow` action if _all_ the conditions are true.
///
/// [`Allow`]: enum.SeccompAction.html
/// [`SeccompCondition`]: struct.SeccompCondition.html
/// [`SeccompRule`]: struct.SeccompRule.html
///
#[macro_export]
macro_rules! and {
    ($($x:expr,)*) => (SeccompRule::new(vec![$($x),*], SeccompAction::Allow));
    ($($x:expr),*) => (SeccompRule::new(vec![$($x),*], SeccompAction::Allow))
}

/// Shorthand for chaining `SeccompRule`s with the `or` operator in a `SeccompFilter`.
///
/// [`SeccompFilter`]: struct.SeccompFilter.html
/// [`SeccompRule`]: struct.SeccompRule.html
///
#[macro_export]
macro_rules! or {
    ($($x:expr,)*) => (vec![$($x),*]);
    ($($x:expr),*) => (vec![$($x),*])
}

#[cfg(target_arch = "aarch64")]
const SYS_mmap: ::std::os::raw::c_long = 222;

// See include/uapi/asm-generic/fcntl.h in the kernel code.
const FCNTL_FD_CLOEXEC: u64 = 1;
const FCNTL_F_SETFD: u64 = 2;

// See include/uapi/linux/futex.h in the kernel code.
const FUTEX_WAIT: u64 = 0;
const FUTEX_WAKE: u64 = 1;
const FUTEX_REQUEUE: u64 = 3;
#[cfg(target_env = "gnu")]
const FUTEX_CMP_REQUEUE: u64 = 4;
const FUTEX_PRIVATE_FLAG: u64 = 128;
const FUTEX_WAIT_PRIVATE: u64 = FUTEX_WAIT | FUTEX_PRIVATE_FLAG;
const FUTEX_WAKE_PRIVATE: u64 = FUTEX_WAKE | FUTEX_PRIVATE_FLAG;
const FUTEX_REQUEUE_PRIVATE: u64 = FUTEX_REQUEUE | FUTEX_PRIVATE_FLAG;
#[cfg(target_env = "gnu")]
const FUTEX_CMP_REQUEUE_PRIVATE: u64 = FUTEX_CMP_REQUEUE | FUTEX_PRIVATE_FLAG;

// See include/uapi/asm-generic/ioctls.h in the kernel code.
const TCGETS: u64 = 0x5401;
const TCSETS: u64 = 0x5402;
const TIOCGWINSZ: u64 = 0x5413;
const FIOCLEX: u64 = 0x5451;
const FIONBIO: u64 = 0x5421;

// See include/uapi/linux/if_tun.h in the kernel code.
const KVM_GET_API_VERSION: u64 = 0xae00;
const KVM_CREATE_VM: u64 = 0xae01;
const KVM_CHECK_EXTENSION: u64 = 0xae03;
const KVM_GET_VCPU_MMAP_SIZE: u64 = 0xae04;
const KVM_CREATE_VCPU: u64 = 0xae41;
const KVM_GET_DIRTY_LOG: u64 = 0x4010_ae42;
const KVM_SET_TSS_ADDR: u64 = 0xae47;
const KVM_CREATE_IRQCHIP: u64 = 0xae60;
const KVM_RUN: u64 = 0xae80;
const KVM_SET_MSRS: u64 = 0x4008_ae89;
const KVM_SET_CPUID2: u64 = 0x4008_ae90;
const KVM_SET_USER_MEMORY_REGION: u64 = 0x4020_ae46;
const KVM_IRQFD: u64 = 0x4020_ae76;
const KVM_CREATE_PIT2: u64 = 0x4040_ae77;
const KVM_IOEVENTFD: u64 = 0x4040_ae79;
const KVM_SET_REGS: u64 = 0x4090_ae82;
const KVM_SET_SREGS: u64 = 0x4138_ae84;
const KVM_SET_FPU: u64 = 0x41a0_ae8d;
const KVM_SET_LAPIC: u64 = 0x4400_ae8f;
const KVM_GET_SREGS: u64 = 0x8138_ae83;
const KVM_GET_LAPIC: u64 = 0x8400_ae8e;
const KVM_GET_SUPPORTED_CPUID: u64 = 0xc008_ae05;

// See include/uapi/linux/if_tun.h in the kernel code.
const TUNSETIFF: u64 = 0x4004_54ca;
const TUNSETOFFLOAD: u64 = 0x4004_54d0;
const TUNSETVNETHDRSZ: u64 = 0x4004_54d8;

// /// Returns a list of rules that allow syscalls required for running a rust program.
// pub fn rust_required_rules() -> Vec<SyscallRuleSet> {
//     vec![
//         allow_syscall(libc::SYS_sigaltstack),
//         allow_syscall(libc::SYS_munmap),
//         allow_syscall(libc::SYS_exit_group),
//     ]
// }

// /// Returns a list of rules that allow syscalls required for executing another program.
// pub fn jailer_required_rules() -> Vec<SyscallRuleSet> {
//     vec![
//         allow_syscall(libc::SYS_rt_sigprocmask),
//         allow_syscall(libc::SYS_rt_sigaction),
//         allow_syscall(libc::SYS_execve),
//         #[cfg(target_arch = "x86_64")]
//         allow_syscall(libc::SYS_mmap),
//         #[cfg(target_arch = "aarch64")]
//         // See this issue for why we are doing it this way on arch64:
//         // https://github.com/rust-lang/libc/issues/1348.
//         allow_syscall(SYS_mmap),
//         #[cfg(target_arch = "x86_64")]
//         allow_syscall(libc::SYS_arch_prctl),
//         allow_syscall(libc::SYS_set_tid_address),
//         #[cfg(target_arch = "x86_64")]
//         allow_syscall(libc::SYS_readlink),
//         #[cfg(target_arch = "x86_64")]
//         allow_syscall(libc::SYS_open),
//         allow_syscall(libc::SYS_read),
//         allow_syscall(libc::SYS_close),
//         allow_syscall(libc::SYS_brk),
//         allow_syscall(libc::SYS_sched_getaffinity),
//     ]
// }

/// Never allow:
///     - sethostname
pub fn toastate_default_filter() -> BpfProgram {
    SeccompFilter::new(
        vec![
            allow_syscall(libc::SYS_rt_sigprocmask),
            allow_syscall(libc::SYS_rt_sigaction),
            allow_syscall(libc::SYS_execve),
            #[cfg(target_arch = "x86_64")]
            allow_syscall(libc::SYS_mmap),
            #[cfg(target_arch = "aarch64")]
            // See this issue for why we are doing it this way on arch64:
            // https://github.com/rust-lang/libc/issues/1348.
            allow_syscall(SYS_mmap),
            #[cfg(target_arch = "x86_64")]
            allow_syscall(libc::SYS_arch_prctl),
            allow_syscall(libc::SYS_set_tid_address),
            #[cfg(target_arch = "x86_64")]
            allow_syscall(libc::SYS_readlink),
            #[cfg(target_arch = "x86_64")]
            allow_syscall(libc::SYS_open),
            allow_syscall(libc::SYS_read),
            allow_syscall(libc::SYS_close),
            allow_syscall(libc::SYS_brk),
            allow_syscall(libc::SYS_sched_getaffinity),
        ]
        .into_iter()
        .collect(),
        SeccompAction::Trap,
    ).unwrap().try_into().unwrap()
}

// /// The default filter containing the white listed syscall rules required by `Firecracker` to
// /// function.
// ///
// pub fn firecracker_vm_default_filter() -> Result<SeccompFilter, Error> {
//     Ok(SeccompFilter::new(
//         vec![
//             allow_syscall(libc::SYS_accept4),
//             allow_syscall(libc::SYS_brk),
//             allow_syscall(libc::SYS_clock_gettime),
//             allow_syscall(libc::SYS_close),
//             allow_syscall(libc::SYS_connect),
//             allow_syscall(libc::SYS_dup),
//             allow_syscall(libc::SYS_epoll_ctl),
//             allow_syscall(libc::SYS_epoll_pwait),
//             #[cfg(all(target_env = "gnu", target_arch = "x86_64"))]
//             allow_syscall(libc::SYS_epoll_wait),
//             allow_syscall(libc::SYS_exit),
//             allow_syscall(libc::SYS_exit_group),
//             allow_syscall_if(
//                 libc::SYS_fcntl,
//                 or![and![
//                     Cond::new(1, ArgLen::DWORD, Eq, FCNTL_F_SETFD)?,
//                     Cond::new(2, ArgLen::QWORD, Eq, FCNTL_FD_CLOEXEC)?,
//                 ]],
//             ),
//             allow_syscall(libc::SYS_fstat),
//             #[cfg(target_arch = "aarch64")]
//             allow_syscall(libc::SYS_newfstatat),
//             allow_syscall_if(
//                 libc::SYS_futex,
//                 or![
//                     and![Cond::new(1, ArgLen::DWORD, Eq, FUTEX_WAIT_PRIVATE)?],
//                     and![Cond::new(1, ArgLen::DWORD, Eq, FUTEX_WAKE_PRIVATE)?],
//                     and![Cond::new(1, ArgLen::DWORD, Eq, FUTEX_REQUEUE_PRIVATE)?],
//                     #[cfg(target_env = "gnu")]
//                     and![Cond::new(1, ArgLen::DWORD, Eq, FUTEX_CMP_REQUEUE_PRIVATE)?],
//                 ],
//             ),
//             allow_syscall(libc::SYS_getrandom),
//             allow_syscall_if(libc::SYS_ioctl, create_firecracker_ioctl_seccomp_rule()?),
//             allow_syscall(libc::SYS_lseek),
//             #[cfg(target_env = "musl")]
//             allow_syscall_if(
//                 libc::SYS_madvise,
//                 or![and![Cond::new(
//                     2,
//                     ArgLen::DWORD,
//                     Eq,
//                     libc::MADV_DONTNEED as u64
//                 )?],],
//             ),
//             allow_syscall(libc::SYS_mmap),
//             allow_syscall(libc::SYS_munmap),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_open),
//             allow_syscall(libc::SYS_openat),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_pipe),
//             allow_syscall(libc::SYS_read),
//             allow_syscall(libc::SYS_readv),
//             allow_syscall(libc::SYS_recvfrom),
//             // SYS_rt_sigreturn is needed in case a fault does occur, so that the signal handler
//             // can return. Otherwise we get stuck in a fault loop.
//             allow_syscall(libc::SYS_rt_sigreturn),
//             allow_syscall(libc::SYS_sigaltstack),
//             allow_syscall_if(
//                 libc::SYS_socket,
//                 or![and![Cond::new(0, ArgLen::DWORD, Eq, libc::AF_UNIX as u64)?],],
//             ),
//             #[cfg(target_arch = "x86_64")]
//             allow_syscall(libc::SYS_stat),
//             allow_syscall(libc::SYS_timerfd_create),
//             allow_syscall(libc::SYS_timerfd_settime),
//             allow_syscall(libc::SYS_write),
//             allow_syscall(libc::SYS_writev),
//         ]
//         .into_iter()
//         .collect(),
//         SeccompAction::Trap,
//     )?)
// }

// /// Applies the default seccomp filtering of firecracker vm to the current thread.
// ///
// pub fn set_firecracker_seccomp_with_level(seccomp_level: u32) -> Result<(), Error> {
//     // Load seccomp filters before executing guest code.
//     // Execution panics if filters cannot be loaded, use --seccomp-level=0 if skipping filters
//     // altogether is the desired behaviour.
//     match seccomp_level {
//         super::SECCOMP_LEVEL_ADVANCED => firecracker_vm_default_filter()?.apply(),
//         super::SECCOMP_LEVEL_BASIC => firecracker_vm_default_filter()?.allow_all().apply(),
//         super::SECCOMP_LEVEL_NONE => Ok(()),
//         _ => Err(Error::InvalidLevel),
//     }
// }

// pub fn create_firecracker_ioctl_seccomp_rule() -> Result<Vec<SeccompRule>, Error> {
//     Ok(or![
//         and![Cond::new(1, ArgLen::DWORD, Eq, TCSETS)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, TCGETS)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, TIOCGWINSZ)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_CHECK_EXTENSION,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_CREATE_VM)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_API_VERSION,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_SUPPORTED_CPUID,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_VCPU_MMAP_SIZE,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_CREATE_IRQCHIP,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_CREATE_PIT2)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_CREATE_VCPU)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_DIRTY_LOG,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_IOEVENTFD)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_IRQFD)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_TSS_ADDR,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_USER_MEMORY_REGION,)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, FIOCLEX)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, FIONBIO)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, TUNSETIFF)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, TUNSETOFFLOAD)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, TUNSETVNETHDRSZ)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_LAPIC)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_GET_SREGS)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_RUN)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_CPUID2)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_FPU)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_LAPIC)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_MSRS)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_REGS)?],
//         and![Cond::new(1, ArgLen::DWORD, Eq, KVM_SET_SREGS)?],
//     ])
// }

// #[cfg(test)]
// #[cfg(target_env = "musl")]
// mod tests {
//     use super::*;
//     use seccomp::SeccompFilter;
//     use std::thread;

//     const EXTRA_SYSCALLS: [i64; 5] = [
//         libc::SYS_clone,
//         libc::SYS_mprotect,
//         libc::SYS_rt_sigprocmask,
//         libc::SYS_set_tid_address,
//         libc::SYS_sigaltstack,
//     ];

//     fn add_syscalls_install_filter(mut filter: SeccompFilter) {
//         // Test error case: add empty rule array.
//         assert!(filter.add_rules(0, vec![],).is_err());
//         // Add "Allow" rule for each syscall.
//         for syscall in EXTRA_SYSCALLS.iter() {
//             assert!(filter
//                 .add_rules(
//                     *syscall,
//                     vec![SeccompRule::new(vec![], SeccompAction::Allow)],
//                 )
//                 .is_ok());
//         }
//         assert!(filter.apply().is_ok());
//     }

//     #[test]
//     fn test_basic_seccomp() {
//         // Spawn a new thread before running the tests because all tests run
//         // in the same thread. Otherwise other tests will fail because of the
//         // installed seccomp filters.
//         thread::spawn(move || {
//             let filter = default_filter().unwrap().allow_all();
//             add_syscalls_install_filter(filter);
//         })
//         .join()
//         .unwrap();
//     }

//     #[test]
//     fn test_advanced_seccomp() {
//         // Spawn a new thread before running the tests because all tests run
//         // in the same thread. Otherwise other tests will fail because of the
//         // installed seccomp filters.
//         thread::spawn(move || {
//             let filter = default_filter().unwrap();
//             add_syscalls_install_filter(filter);
//         })
//         .join()
//         .unwrap();
//     }
// }
