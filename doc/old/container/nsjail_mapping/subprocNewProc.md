# first process in the child after clone

The first process created inside a PID namespace gets a process ID of 1 within the namespace. This process has a similar role to the init process on traditional Linux systems. In particular, the init process can perform initializations required for the PID namespace as whole (e.g., perhaps starting other processes that should be a standard part of the namespace) and becomes the parent for processes in the namespace that become orphaned.

See https://lwn.net/Articles/532748/

PID namespace is mandatory for containers, see https://unix.stackexchange.com/questions/456620/how-to-perform-chroot-with-linux-namespaces:

```
This illustrates one advantage of combining a mount namespace with a "PID namespace". Being inside a PID namespace would prevent you from entering the mount namespace of an unconfined process. It also prevents you entering the root of an unconfined process (/proc/$PID/root). And of course a PID namespace also prevents you from killing any process which is outside it :-).
```

# subprocNewProc

```C++
static void subprocNewProc(nsjconf_t* nsjconf, int fd_in, int fd_out, int fd_err, int pipefd) {
	if (!contain::setupFD(nsjconf, fd_in, fd_out, fd_err)) {
		return;
	}
	if (!resetEnv()) {
		return;
	}

	if (pipefd == -1) { /* in this case we never fall in that and init ns is done in calling process, we would have to set nsjconf-> mode to MODE_STANDALONE_EXECVE */
		if (!user::initNsFromParent(nsjconf, getpid())) {
			LOG_E("Couldn't initialize net user namespace");
			return;
		}
		if (nsjconf->use_cgroupv2) {
			if (!cgroup2::initNsFromParent(nsjconf, getpid())) {
				LOG_E("Couldn't initialize net user namespace");
				return;
			}
		} else if (!cgroup::initNsFromParent(nsjconf, getpid())) {
			LOG_E("Couldn't initialize net user namespace");
			return;
		}
	} else {
		char doneChar;
		if (util::readFromFd(pipefd, &doneChar, sizeof(doneChar)) != sizeof(doneChar)) { /* kSubprocDoneChar is written to pipefd after calling process has sucessfully init NS */
			return;
		}
		if (doneChar != kSubprocDoneChar) {
			return;
		}
	}
	if (!contain::containProc(nsjconf)) { /* mounts are done here, from within the child */
		return;
	}
	if (!nsjconf->keep_env) {
		clearenv(); /* we fall into this case with the mapping's nsjconf */
	}
	for (const auto& env : nsjconf->envs) {
		putenv(const_cast<char*>(env.c_str()));
	}

	auto connstr = net::connToText(fd_in, /* remote= */ true, NULL);
	LOG_I("Executing '%s' for '%s'", nsjconf->exec_file.c_str(), connstr.c_str());

	std::vector<const char*> argv;
	for (const auto& s : nsjconf->argv) {
		argv.push_back(s.c_str());
		LOG_D(" Arg: '%s'", s.c_str());
	}
	argv.push_back(nullptr);

	/* Should be the last one in the sequence */
	if (!sandbox::applyPolicy(nsjconf)) { /* is it here that mount perms are taken back ? mount syscall is effectivement forbidden by the kafel policy */
		return;
	}

	if (nsjconf->use_execveat) { /* we do not use exec caveat in this sample */
#if defined(__NR_execveat)
		util::syscall(__NR_execveat, nsjconf->exec_fd, (uintptr_t) "",
		    (uintptr_t)argv.data(), (uintptr_t)environ, AT_EMPTY_PATH);
#else  /* defined(__NR_execveat) */
		LOG_E("Your system doesn't support execveat() syscall");
		return;
#endif /* defined(__NR_execveat) */
	} else {
		execv(nsjconf->exec_file.c_str(), (char* const*)argv.data());
	}

	PLOG_E("execve('%s') failed", nsjconf->exec_file.c_str());
}
```

# contain::setupFD(nsjconf, fd_in, fd_out, fd_err)

# resetEnv()

# contain::containProc

Do the following: 

## containUserNs

### user::initNsFromChild:

This function does nothing if `(!nsjconf->clone_newuser && nsjconf->orig_euid != 0)`.

it makes sure all capabilities are retained after the subsequent setuid/setgid, as they will be needed for privileged operations: mounts, uts change etc, with syscall `prctl(PR_SET_SECUREBITS, SECBIT_KEEP_CAPS | SECBIT_NO_SETUID_FIXUP, 0UL, 0UL, 0UL)`.

setResGid from nsjconf->gids[0].inside_id

Then setgroups syscall from groups<gid_t> = [nsjconf->gids[i].inside_id]. Best effort because of /proc/self/setgroups. We deny setgroups(2) calls only if user namespaces are in use.

Then `setResUid(nsjconf->uids[0].inside_id)`

Disable securebits again to avoid spawned programs unexpectedly retaining capabilities after a UID/GID change with `prctl(PR_SET_SECUREBITS, 0UL, 0UL, 0UL, 0UL)`

## containInitPidNs

### pid::initNs

```C++
if (nsjconf->mode != MODE_STANDALONE_EXECVE) { /* this mapping is for mode  MODE_STANDALONE_ONCE which is default one*/
		return true;
}
```

## containInitMountNs

### mnt::initNs

`if (nsjconf->mode != MODE_STANDALONE_EXECVE)` which is the case, then we call function `initNsInternal` then return.

- initNsInternal: (only thing done when not in MODE_STANDALONE_EXECVE)

	- `if (nsjconf->clone_newns)` which is the case, then initCloneNs:

		`chdir("/")`

		destdir can have these values, the first one that succeeds to be created and accessed is choosen : `destdir = "/run/user/{nsjconf->orig_uid}/nsjail/root" || "/run/user/nsjail.{nsjconf->orig_uid}.root" || "/tmp/nsjail.{nsjconf->orig_uid}.root" || "{getenv("TMPDIR")}/nsjail.{nsjconf->orig_uid}.root" || "/dev/shm/nsjail.{nsjconf->orig_uid}.root" || "/tmp/nsjail.{nsjconf->orig_uid}.root.{rand integer}"`

		Make changes to / (recursively) private, to avoid changing the global mount ns: `mount("/", "/", NULL, MS_REC | MS_PRIVATE, NULL)`

		`mount(NULL, destdir->c_str(), "tmpfs", 0, "size=16777216")`

		tmpdir is choosen as destdir, but instead of "root", we have "tmp"

		`mount(NULL, tmpdir->c_str(), "tmpfs", 0, "size=16777216")`

		TO BE COMPLETE, MANY MOUNT, UMOUNT2 AND PIVOT ROOT AFTER THAT


	- `chdir(nsjconf->cwd.c_str())`

## containInitNetNs

TODO: complete

## containInitUtsNs

### uts::initNs

`sethostname(nsjconf->hostname.data(), nsjconf->hostname.length())`

## containInitCgroupNs

### cgroup::initNs

It is done in parent

```C++
bool initNs(void) {
	return true;
}
```

## containDropPrivs

```C++
#ifndef PR_SET_NO_NEW_PRIVS
#define PR_SET_NO_NEW_PRIVS 38
#endif

if (!nsjconf->disable_no_new_privs) { /* which is the case in this mapping */
	prctl(PR_SET_NO_NEW_PRIVS, 1UL, 0UL, 0UL, 0UL)
}

caps::initNs(nsjconf)
```

- caps::initNs(nsjconf):

	Seems to get the current capabilities of the process with `util::syscall(__NR_capget, (uintptr_t)&cap_hdr, (uintptr_t)&cap_data)`, than clearInheritable from the returned caps, to avoid mistakes by starting with an empty inheritable set. it then removes all capabilities from the ambient set first with `prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL, 0UL, 0UL, 0UL)`. It works with newer kernel versions only, so the call don't panic() if it fails. Our current nsjconf does not contain any allowed cap, so this function makes sure all other caps (those which were not explicitly requested) are removed from the bounding set. We need to have CAP_SETPCAP to do that now. It calls `prctl(PR_CAPBSET_DROP, (unsigned long)i.val, 0UL, 0UL, 0UL)`. Finally, it makes sure inheritable set is preserved across execve via the modified ambient set by calling `prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_RAISE, (unsigned long)cap, 0UL, 0UL)`.


## Next calls are made as non-root thanks to containDropPrivs


## containCPU


### cpu::initCpu

```C++
cpu_set_t* mask = CPU_ALLOC(nsjconf->num_cpus);

size_t mask_size = CPU_ALLOC_SIZE(nsjconf->num_cpus);
CPU_ZERO_S(mask_size, mask);

for (size_t i = 0; i < nsjconf->max_cpus; i++) {
	setRandomCpu(mask, mask_size, nsjconf->num_cpus); /* Select random core to assign */
}

sched_setaffinity(0, mask_size, mask) /* set cpu affinity for a task */

CPU_FREE(mask);
```

## containSetLimits

TODO: complete

Handles RLIMIT_XXX like for example RLIMIT_CPU or RLIMIT_NPROC

## containPrepareEnv

TODO: complete

## containMakeFdsCOE

TODO: complete

Apparently, marks relevant file-descriptors as close-on-exec

# execv