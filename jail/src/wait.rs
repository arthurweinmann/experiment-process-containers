/**
 * Wait for all children, so it is not for use in the global rust programs that handles multiple container
 * You may use it for tests with only one container
 * This function blocks until all children are done
*/
pub fn reap_proc(print_seccomp_violation: bool) -> i32 {
    cpp_bindings::reap_proc(print_seccomp_violation)
}
