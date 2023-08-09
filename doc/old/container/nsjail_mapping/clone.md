# Clone Func in nsjail>subproc.cc

```C++
/*
 * Will be used inside the child process only, so it's safe to have it in BSS.
 * Some CPU archs (e.g. aarch64) must have it aligned. Size: 128 KiB (/2)
 */
static uint8_t cloneStack[128 * 1024] __attribute__((aligned(__BIGGEST_ALIGNMENT__)));
/* Cannot be on the stack, as the child's stack pointer will change after clone() */
static __thread jmp_buf env;

static int cloneFunc(void* arg __attribute__((unused))) {
	longjmp(env, 1);
	return 0;
}

/*
 * Avoid problems with caching of PID/TID in glibc - when using syscall(__NR_clone) glibc doesn't
 * update the internal PID/TID caches, what can lead to invalid values being returned by getpid()
 * or incorrect PID/TIDs used in raise()/abort() functions
 */
pid_t cloneProc(uintptr_t flags) {
	if (flags & CLONE_VM) {
		LOG_E("Cannot use clone(flags & CLONE_VM)");
		return -1;
	}

	if (setjmp(env) == 0) {
		LOG_D("Cloning process with flags:%s", cloneFlagsToStr(flags).c_str());
		/*
		 * Avoid the problem of the stack growing up/down under different CPU architectures,
		 * by using middle of the static stack buffer (which is temporary, and used only
		 * inside of the cloneFunc()
		 */
		void* stack = &cloneStack[sizeof(cloneStack) / 2];
		/* Parent */
		return clone(cloneFunc, stack, flags, NULL, NULL, NULL); /* "real" pid of the child process in parent pid namespace
	}
	/* Child */
	return 0;
}
```

## jmpbuf

The jmp_buf type is an array type suitable for storing information to restore a calling environment. The stored information is sufficient to restore execution at the correct block of the program and invocation of that block. The state of floating-point status flags, or open files, or any other data is not stored in an object of type jmp_buf.

This information is filled by calling macro setjmp and can be restored by calling function longjmp.

### setjmp (http://en.cppreference.com/w/cpp/utility/program/setjmp)

Saves the current execution context into a variable env of type std::jmp_buf. This variable can later be used to restore the current execution context by std::longjmp function. That is, when a call to std::longjmp function is made, the execution continues at the particular call site that constructed the std::jmp_buf variable passed to std::longjmp. In that case setjmp returns the value passed to std::longjmp

Upon return to the scope of setjmp, all accessible objects, floating-point status flags, and other components of the abstract machine have the same values as they had when std::longjmp was executed, except for the non-volatile local variables in setjmp's scope, whose values are indeterminate if they have been changed since the setjmp invocation.

#### Return value

​0​ if the macro was called by the original code and the execution context was saved to env.

Non-zero value if a non-local jump was just performed. The return value is the same as passed to std::longjmp.

### longjmp (https://en.cppreference.com/w/cpp/utility/program/longjmp)

Loads the execution context env saved by a previous call to setjmp. This function does not return. Control is transferred to the call site of the macro setjmp that set up env. That setjmp then returns the value, passed as the status.

If the function that called setjmp has exited, the behavior is undefined (in other words, only long jumps up the call stack are allowed)

No destructors for automatic objects are called. If replacing of std::longjmp with throw and setjmp with catch would execute a non-trivial destructor for any automatic object, the behavior of such std::longjmp is undefined.

### Safe to use ?

```
setjmp()/longjmp() completely subvert stack unwinding and therefore exception handling as well as RAII (destructors in general).

From 18.7/4 "Other runtime support" in the standard:

If any automatic objects would be destroyed by a thrown exception transferring control to another (destination) point in the program, then a call to longjmp(jbuf, val) at the throw point that transfers control to the same (destination) point has undefined behavior.

So the bottom line is that setjmp()/longjmp() do not play well in C++.
```

### RUST does not support setjmp / longjmp

see https://github.com/rust-lang/rfcs/issues/2625.

```
It's not that easy. For example, setjmp/longjmp are incompatible with closure-based APIs that expect "well-bracketed control flow", such as crossbeam::scope: you could use longjmp to skip the part of the code that is otherwise guaranteed to run. This has nothing to do with dropping.

Moreover, mem::forget leaks memory and does not call the destructor. setjmp/longjmp deallocates memory without calling its destructor, and that's unsound. For pinned memory we guarantee that once some memory is pinned, if this memory every gets invalidated (e.g., deallocated), then the destructor will first get called. Any mechanism that pops stack frames without calling destructors breaks this guarantee (well, at least it makes stack-pinning impossible, and that is intended to be allowed).

In other words, the "observational equivalence" argument does not apply in this case, setjmp/longjmp increases the observational power of the context. (This is a well-known result about continuations in PL theory, it is not a particular quirk of how they got implemented in C.) I don't think there is any way setjmp/longjmp can ever be used to "jump across" unknown code. This can only be sound if you can carefully control the stack frames that get popped by the longjmp, and make sure that they contain no types where skipping drop is a problem.
```

## Clone call (http://man7.org/linux/man-pages/man2/clone.2.html)

***We must be sure that libc::clone is glibc clone() wrapper and not the raw system call***

Unlike fork(2), clone() allows the child process to share parts of
its execution context with the calling process, such as the virtual
address space, the table of file descriptors, and the table of signal
handlers.

When the child process is created with clone(), it commences
execution by calling the function pointed to by the argument fn.

## The difference between fork(), vfork(), exec() and clone() ?

- https://stackoverflow.com/questions/4856255/the-difference-between-fork-vfork-exec-and-clone:

vfork() is an obsolete optimization. Before good memory management, fork() made a full copy of the parent's memory, so it was pretty expensive. since in many cases a fork() was followed by exec(), which discards the current memory map and creates a new one, it was a needless expense. Nowadays, fork() doesn't copy the memory; it's simply set as "copy on write", so fork()+exec() is just as efficient as vfork()+exec().

clone() is the syscall used by fork(). with some parameters, it creates a new process, with others, it creates a thread. the difference between them is just which data structures (memory space, processor state, stack, PID, open files, etc) are shared or not.