use super::config::JailConf;

// TODO: see how to implement all calls from nsjail:cpu.cc which is not the case right now, (for example libc lacks CPU_ALLOC and CPU_ALLOC_SIZE)
// For now it is implemented through cpp_bindings package
// see https://linux.die.net/man/3/cpu_alloc
pub fn init_cpu(jconf: &JailConf) -> bool {
    cpp_bindings::init_cpu(jconf.num_cpus as libc::c_int, jconf.max_cpus as libc::c_int)
}