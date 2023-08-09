#include <unistd.h>
#include <sys/syscall.h>
#include <netlink/route/link.h>
#include <arpa/inet.h>
#include <net/if.h>
#include <net/route.h>
#include <sys/ioctl.h>
#include <fcntl.h>
#include <string>
#include <signal.h>
#include <iostream>
#include <sys/wait.h>
#include <limits.h>
#include <inttypes.h>
#include <linux/capability.h>
#include <sys/prctl.h>

// #include <linux/btrfs.h>

// #include <sys/capability.h>

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     SYSCALLS UTILS
 * 
 * 
 * ***********************************************************************************************************************************************************/

long syscall(long sysno, uintptr_t a0, uintptr_t a1, uintptr_t a2, uintptr_t a3, uintptr_t a4,
			 uintptr_t a5)
{
	return ::syscall(sysno, a0, a1, a2, a3, a4, a5);
}

extern "C"

	// why not use static int and how to transcribe it into rust ??? https://stackoverflow.com/questions/15235526/the-static-keyword-and-its-various-uses-in-c
	long
	getResGIDSyscallNb()
{

#if defined(__NR_setresgid32)
	return __NR_setresgid32;
#endif /* defined(__NR_setresuid32) */

	return __NR_setresgid;
}

extern "C"

	long
	setResUIDSyscallNb()
{

#if defined(__NR_setresuid32)
	return __NR_setresuid32;
#else  /* defined(__NR_setresuid32) */
	return __NR_setresuid;
#endif /* defined(__NR_setresuid32) */
}

extern "C"

	int
	libnlRtmSetLink()
{
	return RTM_SETLINK;
}

extern "C"

	int
	libnlNlmFCreate()
{
	return NLM_F_CREATE;
}

/* NSJail func was defined as follow in user.cc

// did not find another way in rust to do ```#if defined(__NR_setresgid32)``` to test whether a syscall is defined or not
// if we find one in the future we won't need this anymore

static bool setResGid(gid_t gid) {
	LOG_D("setresgid(%d)", gid);
#if defined(__NR_setresgid32)
	if (util::syscall(__NR_setresgid32, (uintptr_t)gid, (uintptr_t)gid, (uintptr_t)gid) == -1) {
		PLOG_W("setresgid32(%d)", (int)gid);
		return false;
	}
#else  // defined(__NR_setresgid32)
	if (util::syscall(__NR_setresgid, (uintptr_t)gid, (uintptr_t)gid, (uintptr_t)gid) == -1) {
		PLOG_W("setresgid(%d)", gid);
		return false;
	}
#endif // defined(__NR_setresuid32)
	return true;
}

static bool setResUid(uid_t uid) {
	LOG_D("setresuid(%d)", uid);
#if defined(__NR_setresuid32)
	if (util::syscall(__NR_setresuid32, (uintptr_t)uid, (uintptr_t)uid, (uintptr_t)uid) == -1) {
		PLOG_W("setresuid32(%d)", (int)uid);
		return false;
	}
#else  //defined(__NR_setresuid32)
	if (util::syscall(__NR_setresuid, (uintptr_t)uid, (uintptr_t)uid, (uintptr_t)uid) == -1) {
		PLOG_W("setresuid(%d)", uid);
		return false;
	}
#endif //defined(__NR_setresuid32)
	return true;
}

*/

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     NET.CC
 * 
 * 
 * ***********************************************************************************************************************************************************/

extern "C"

	int
	ifaceUp(const char *ifacename)
{
	int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_IP);
	if (sock == -1)
	{
		printf("ifaceUp->socket(AF_INET, SOCK_STREAM, IPPROTO_IP)");
		return 0;
	}

	struct ifreq ifr;
	memset(&ifr, '\0', sizeof(ifr));
	snprintf(ifr.ifr_name, IF_NAMESIZE, "%s", ifacename);

	if (ioctl(sock, SIOCGIFFLAGS, &ifr) == -1)
	{
		close(sock);
		printf("ifaceUp->ioctl(sock, SIOCGIFFLAGS, &ifr)");
		return 0;
	}

	ifr.ifr_flags |= (IFF_UP | IFF_RUNNING);

	if (ioctl(sock, SIOCSIFFLAGS, &ifr) == -1)
	{
		close(sock);
		printf("ifaceUp->ioctl(sock, SIOCSIFFLAGS, &ifr)");
		return 0;
	}

	close(sock);
	return 1;
}

// extern "C"

// 	bool
// 	testCCharToStdString(const char *iface)
// {
// 	printf("bob is: %s\n", iface);

// 	/* This is how std::string s is assigned
//        though a  C string ‘a’ */
// 	// string s(a);
// 	// cf https://www.geeksforgeeks.org/how-to-convert-c-style-strings-to-stdstring-and-vice-versa/
// 	std::string s(iface);

// 	printf("bob2 is: %s\n", s.c_str()); // https://www.maizure.org/projects/printf/index.html

// 	std::cout << s;

// 	// printf("iface: %s; ip: %s ; mask: %s ; gw: %s");
// 	// return true; returns false by default, intriguing :thinking:, -> investigate

// 	// also std::cout << s; seems to print to stdout after rust test prints its finish lines, why ? is it kind of async or is the cpp binding executed in another thread ?

// 	return true;
// }

extern "C"

	// bool
	// ifaceConfig   (const std::string &iface, const std::string &ip, const std::string &mask,
	// 			const std::string &gw)
	// nsjail uses of course std::string which manages its own memory and has lots of advantage, unfortunately RUST
	// does not quite understand it. So we need to create std::string from const char*, test if no memory does leak

	bool
	ifaceConfig(const char *ifacec, const char *ipc, const char *maskc,
				const char *gwc)
{
	// conversion to std::string, see test function testCCharToStdString above
	// we could may have used the raw const char * instead of the std::string.c_str() below
	// which transform std::string into const char * anyway, but since std::string
	// are supposed to manage and free their own memory, we use caution by still using
	// them. Nevertheless, investigate !
	std::string iface(ifacec);
	std::string ip(ipc);
	std::string mask(maskc);
	std::string gw(gwc);

	int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_IP);
	if (sock == -1)
	{
		printf("ifaceConfig->socket(AF_INET, SOCK_STREAM, IPPROTO_IP)");
		return false;
	}

	struct in_addr addr;
	if (inet_pton(AF_INET, ip.c_str(), &addr) != 1)
	{
		close(sock);
		printf("ifaceConfig->inet_pton(AF_INET, ip.c_str(), &addr)");
		return false;
	}
	if (addr.s_addr == INADDR_ANY)
	{
		close(sock);
		printf("ifaceConfig->addr.s_addr == INADDR_ANY");
		return true;
	}

	struct ifreq ifr;
	memset(&ifr, '\0', sizeof(ifr));
	snprintf(ifr.ifr_name, IF_NAMESIZE, "%s", iface.c_str());
	struct sockaddr_in *sa = (struct sockaddr_in *)(&ifr.ifr_addr);
	sa->sin_family = AF_INET;
	sa->sin_addr = addr;
	if (ioctl(sock, SIOCSIFADDR, &ifr) == -1)
	{
		close(sock);
		printf("ifaceConfig->ioctl(sock, SIOCSIFADDR, &ifr)");
		return false;
	}

	if (inet_pton(AF_INET, mask.c_str(), &addr) != 1)
	{
		close(sock);
		printf("ifaceConfig->inet_pton(AF_INET, mask.c_str(), &addr)");
		return false;
	}
	sa->sin_family = AF_INET;
	sa->sin_addr = addr;
	if (ioctl(sock, SIOCSIFNETMASK, &ifr) == -1)
	{
		close(sock);
		printf("ifaceConfig->ioctl(sock, SIOCSIFNETMASK, &ifr)");
		return false;
	}

	if (!ifaceUp(iface.c_str()))
	{
		close(sock);
		printf("ifaceConfig->!ifaceUp(iface.c_str())");
		return false;
	}

	if (inet_pton(AF_INET, gw.c_str(), &addr) != 1)
	{
		close(sock);
		printf("ifaceConfig->inet_pton(AF_INET, gw.c_str(), &addr)");
		return false;
	}
	if (addr.s_addr == INADDR_ANY)
	{
		close(sock);
		return true;
	}

	struct rtentry rt;
	memset(&rt, '\0', sizeof(rt));
	struct sockaddr_in *sdest = (struct sockaddr_in *)(&rt.rt_dst);
	struct sockaddr_in *smask = (struct sockaddr_in *)(&rt.rt_genmask);
	struct sockaddr_in *sgate = (struct sockaddr_in *)(&rt.rt_gateway);
	sdest->sin_family = AF_INET;
	sdest->sin_addr.s_addr = INADDR_ANY;
	smask->sin_family = AF_INET;
	smask->sin_addr.s_addr = INADDR_ANY;
	sgate->sin_family = AF_INET;
	sgate->sin_addr = addr;

	rt.rt_flags = RTF_UP | RTF_GATEWAY;
	char rt_dev[IF_NAMESIZE];
	snprintf(rt_dev, sizeof(rt_dev), "%s", iface.c_str());
	rt.rt_dev = rt_dev;

	if (ioctl(sock, SIOCADDRT, &rt) == -1)
	{
		close(sock);
		printf("ifaceConfig->ioctl(sock, SIOCADDRT, &rt)");
		return false;
	}

	close(sock);
	return true;
}

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     UTILS.CC
 * 
 * 
 * ***********************************************************************************************************************************************************/

ssize_t readFromFd(int fd, void *buf, size_t len)
{
	uint8_t *charbuf = (uint8_t *)buf;

	size_t readSz = 0;
	while (readSz < len)
	{
		ssize_t sz = TEMP_FAILURE_RETRY(read(fd, &charbuf[readSz], len - readSz));
		if (sz <= 0)
		{
			break;
		}
		readSz += sz;
	}
	return readSz;
}

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     CPU.CC
 * 
 * 
 * ***********************************************************************************************************************************************************/

static __thread pthread_once_t rndThreadOnce = PTHREAD_ONCE_INIT;
static __thread uint64_t rndX;

/* MMIX LCG PRNG */
static const uint64_t a = 6364136223846793005ULL;
static const uint64_t c = 1442695040888963407ULL;

static void rndInitThread(void)
{
#if defined(__NR_getrandom)
	if (syscall(__NR_getrandom, (uintptr_t)&rndX, sizeof(rndX), 0) == sizeof(rndX))
	{
		return;
	}
#endif /* defined(__NR_getrandom) */
	int fd = TEMP_FAILURE_RETRY(open("/dev/urandom", O_RDONLY | O_CLOEXEC));
	if (fd == -1)
	{
		printf(
			"Couldn't open /dev/urandom for reading. Using gettimeofday "
			"fall-back");
		struct timeval tv;
		gettimeofday(&tv, NULL);
		rndX = tv.tv_usec + ((uint64_t)tv.tv_sec << 32);
		return;
	}
	if (readFromFd(fd, (uint8_t *)&rndX, sizeof(rndX)) != sizeof(rndX))
	{
		printf("Couldn't read '%zu' bytes from /dev/urandom", sizeof(rndX));
		close(fd);
	}
	close(fd);
}

uint64_t rnd64(void)
{
	pthread_once(&rndThreadOnce, rndInitThread);
	rndX = a * rndX + c;
	return rndX;
}

void setRandomCpu(cpu_set_t *mask, size_t mask_size, size_t cpu_num)
{
	if ((size_t)CPU_COUNT_S(mask_size, mask) >= cpu_num)
	{
		printf("Number of CPUs in the mask '%d' is bigger than number of available CPUs '%zu'",
			   CPU_COUNT(mask), cpu_num);
	}

	for (;;)
	{
		uint64_t n = rnd64() % cpu_num;
		if (!CPU_ISSET_S(n, mask_size, mask))
		{
			printf("Setting allowed CPU#:%" PRIu64 " of [0-%zu]", n, cpu_num - 1);
			CPU_SET_S(n, mask_size, mask);
			break;
		}
	}
}

extern "C"

	int
	initCpu(int num_cpus, int max_cpus)
{
	if (num_cpus < 0)
	{
		printf("sysconf(_SC_NPROCESSORS_ONLN) returned %d", num_cpus);
		return 0;
	}
	if (max_cpus > num_cpus)
	{
		printf("Requested number of CPUs:%d is bigger than CPUs online:%d",
			   max_cpus, num_cpus);
		return 1;
	}
	if (max_cpus == num_cpus)
	{
		printf("All CPUs requested (%d of %d)", max_cpus, num_cpus);
		return 1;
	}
	if (max_cpus == 0)
	{
		printf("No max_cpus limit set");
		return 1;
	}

	cpu_set_t *mask = CPU_ALLOC(num_cpus);
	if (mask == NULL)
	{
		printf("Failure allocating cpu_set_t for %d CPUs", num_cpus);
		return 0;
	}

	size_t mask_size = CPU_ALLOC_SIZE(num_cpus);
	CPU_ZERO_S(mask_size, mask);

	for (int i = 0; i < max_cpus; i++)
	{
		setRandomCpu(mask, mask_size, num_cpus);
	}

	if (sched_setaffinity(0, mask_size, mask) == -1)
	{
		printf("sched_setaffinity(max_cpus=%d) failed", max_cpus);
		CPU_FREE(mask);
		return 0;
	}
	CPU_FREE(mask);

	return 1;
}

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     WAIT PID
 * 
 * 
 * ***********************************************************************************************************************************************************/

static void seccompViolation(siginfo_t *si)
{
	printf("pid=%d commited a syscall/seccomp violation and exited with SIGSYS", si->si_pid);

	char fname[PATH_MAX];
	snprintf(fname, sizeof(fname), "/proc/%d/syscall", (int)si->si_pid);
	int pid_syscall_fd = TEMP_FAILURE_RETRY(open(fname, O_RDONLY | O_CLOEXEC));

	char buf[4096];
	ssize_t rdsize = readFromFd(pid_syscall_fd, buf, sizeof(buf) - 1);
	close(pid_syscall_fd);
	if (rdsize < 1)
	{
		printf("pid=%d, SiSyscall: %d, SiCode: %d, SiErrno: %d, SiSigno: %d",
			   (int)si->si_pid, si->si_syscall, si->si_code, si->si_errno, si->si_signo);
		return;
	}
	buf[rdsize - 1] = '\0';

	uintptr_t arg1, arg2, arg3, arg4, arg5, arg6, sp, pc;
	ptrdiff_t sc;
	int ret = sscanf(buf, "%td %tx %tx %tx %tx %tx %tx %tx %tx", &sc, &arg1, &arg2, &arg3,
					 &arg4, &arg5, &arg6, &sp, &pc);
	if (ret == 9)
	{
		printf(
			"pid=%d, Syscall number: %td, Arguments: %#tx, %#tx, %#tx, %#tx, %#tx, %#tx, "
			"SP: %#tx, PC: %#tx, si_syscall: %d, si_errno: %#x",
			(int)si->si_pid, sc, arg1, arg2, arg3, arg4, arg5, arg6, sp, pc, si->si_syscall,
			si->si_errno);
	}
	else if (ret == 3)
	{
		printf(
			"pid=%d, SiSyscall: %d, SiCode: %d, SiErrno: %d, SiSigno: %d, SP: %#tx, PC: "
			"%#tx",
			(int)si->si_pid, si->si_syscall, si->si_code, si->si_errno, si->si_signo, arg1,
			arg2);
	}
	else
	{
		printf("pid=%d, SiSyscall: %d, SiCode: %d, SiErrno: %d, Syscall string '%s'",
			   (int)si->si_pid, si->si_syscall, si->si_code, si->si_errno, buf);
	}
}

static int reapProc_wait4(pid_t pid)
{
	int status;

	if (wait4(pid, &status, 0, NULL) == pid) //  -> epoll pidfd: ajouter a epoll
	{										 // hang until something happens

		if (WIFEXITED(status))
		{
			return WEXITSTATUS(status);
		}
		if (WIFSIGNALED(status))
		{
			return 128 + WTERMSIG(status);
		}
	}
	return 0;
}

extern "C"

	int
	reapProc(int printSeccompViolation)
{
	int rv = 0;
	siginfo_t si;

	for (;;)
	{
		si.si_pid = 0;
		if (waitid(P_ALL, 0, &si, WNOHANG | WNOWAIT | WEXITED) == -1) // epoll pidfd -> executer ca dans le top parent, il choppe les pid de tout le monde
		{
			break;
		}
		if (si.si_pid == 0)
		{
			break;
		} // ajouter les pid trouver a epoll
		if (si.si_code == CLD_KILLED && si.si_status == SIGSYS)
		{
			if (printSeccompViolation)
			{
				seccompViolation(&si);
			}
		}
		rv = reapProc_wait4(si.si_pid);
	}

	return rv;
}

/**************************************************************************************************************************************************************
 * 
 * 
 * 
 *                                                     CAPS.CC
 * 
 * 
 * ***********************************************************************************************************************************************************/

#define NS_VALSTR_STRUCT(x) \
	{                       \
		x, #x               \
	}

// TODO: use sys_utils bindings generated for target linux kernel to be sure to always have all caps
struct {
	const int val;
	const char* const name; // previously const char *const name ?
} static const capNames[] = {
    NS_VALSTR_STRUCT(CAP_CHOWN),
    NS_VALSTR_STRUCT(CAP_DAC_OVERRIDE),
    NS_VALSTR_STRUCT(CAP_DAC_READ_SEARCH),
    NS_VALSTR_STRUCT(CAP_FOWNER),
    NS_VALSTR_STRUCT(CAP_FSETID),
    NS_VALSTR_STRUCT(CAP_KILL),
    NS_VALSTR_STRUCT(CAP_SETGID),
    NS_VALSTR_STRUCT(CAP_SETUID),
    NS_VALSTR_STRUCT(CAP_SETPCAP),
    NS_VALSTR_STRUCT(CAP_LINUX_IMMUTABLE),
    NS_VALSTR_STRUCT(CAP_NET_BIND_SERVICE),
    NS_VALSTR_STRUCT(CAP_NET_BROADCAST),
    NS_VALSTR_STRUCT(CAP_NET_ADMIN),
    NS_VALSTR_STRUCT(CAP_NET_RAW),
    NS_VALSTR_STRUCT(CAP_IPC_LOCK),
    NS_VALSTR_STRUCT(CAP_IPC_OWNER),
    NS_VALSTR_STRUCT(CAP_SYS_MODULE),
    NS_VALSTR_STRUCT(CAP_SYS_RAWIO),
    NS_VALSTR_STRUCT(CAP_SYS_CHROOT),
    NS_VALSTR_STRUCT(CAP_SYS_PTRACE),
    NS_VALSTR_STRUCT(CAP_SYS_PACCT),
    NS_VALSTR_STRUCT(CAP_SYS_ADMIN),
    NS_VALSTR_STRUCT(CAP_SYS_BOOT),
    NS_VALSTR_STRUCT(CAP_SYS_NICE),
    NS_VALSTR_STRUCT(CAP_SYS_RESOURCE),
    NS_VALSTR_STRUCT(CAP_SYS_TIME),
    NS_VALSTR_STRUCT(CAP_SYS_TTY_CONFIG),
    NS_VALSTR_STRUCT(CAP_MKNOD),
    NS_VALSTR_STRUCT(CAP_LEASE),
    NS_VALSTR_STRUCT(CAP_AUDIT_WRITE),
    NS_VALSTR_STRUCT(CAP_AUDIT_CONTROL),
    NS_VALSTR_STRUCT(CAP_SETFCAP),
    NS_VALSTR_STRUCT(CAP_MAC_OVERRIDE),
    NS_VALSTR_STRUCT(CAP_MAC_ADMIN),
    NS_VALSTR_STRUCT(CAP_SYSLOG),
    NS_VALSTR_STRUCT(CAP_WAKE_ALARM),
    NS_VALSTR_STRUCT(CAP_BLOCK_SUSPEND),
#if defined(CAP_AUDIT_READ)
    NS_VALSTR_STRUCT(CAP_AUDIT_READ),
#endif /* defined(CAP_AUDIT_READ) */
#if defined(CAP_BPF)
    NS_VALSTR_STRUCT(CAP_BPF),
#endif /* defined(CAP_BPF) */
#if defined(CAP_PERFMON)
    NS_VALSTR_STRUCT(CAP_PERFMON),
#endif /* defined(CAP_PERFMON) */
#if defined(CAP_CHECKPOINT_RESTORE)
    NS_VALSTR_STRUCT(CAP_CHECKPOINT_RESTORE),
#endif /* defined(CAP_CHECKPOINT_RESTORE) */
};

static cap_user_data_t getCaps()
{
	static __thread struct __user_cap_data_struct cap_data[_LINUX_CAPABILITY_U32S_3];
	const struct __user_cap_header_struct cap_hdr = {
		.version = _LINUX_CAPABILITY_VERSION_3,
		.pid = 0,
	};
	if (syscall(__NR_capget, (uintptr_t)&cap_hdr, (uintptr_t)&cap_data) == -1)
	{
		printf("capget() failed");
		return NULL;
	}
	return cap_data;
}

static void clearInheritable(cap_user_data_t cap_data)
{
	for (size_t i = 0; i < _LINUX_CAPABILITY_U32S_3; i++)
	{
		cap_data[i].inheritable = 0U;
	}
}

static bool setCaps(const cap_user_data_t cap_data)
{
	const struct __user_cap_header_struct cap_hdr = {
		.version = _LINUX_CAPABILITY_VERSION_3,
		.pid = 0,
	};
	if (syscall(__NR_capset, (uintptr_t)&cap_hdr, (uintptr_t)cap_data) == -1)
	{
		printf("capset() failed");
		return false;
	}
	return true;
}

static void setInheritable(cap_user_data_t cap_data, unsigned int cap)
{
	size_t off_byte = CAP_TO_INDEX(cap);
	unsigned mask = CAP_TO_MASK(cap);
	cap_data[off_byte].inheritable |= mask;
}

static bool getEffective(cap_user_data_t cap_data, unsigned int cap)
{
	size_t off_byte = CAP_TO_INDEX(cap);
	unsigned mask = CAP_TO_MASK(cap);
	return cap_data[off_byte].effective & mask;
}

static bool getPermitted(cap_user_data_t cap_data, unsigned int cap)
{
	size_t off_byte = CAP_TO_INDEX(cap);
	unsigned mask = CAP_TO_MASK(cap);
	return cap_data[off_byte].permitted & mask;
}

static bool getInheritable(cap_user_data_t cap_data, unsigned int cap)
{
	size_t off_byte = CAP_TO_INDEX(cap);
	unsigned mask = CAP_TO_MASK(cap);
	return cap_data[off_byte].inheritable & mask;
}

#if !defined(PR_CAP_AMBIENT)
#define PR_CAP_AMBIENT 47
#define PR_CAP_AMBIENT_RAISE 2
#define PR_CAP_AMBIENT_CLEAR_ALL 4
#endif /* !defined(PR_CAP_AMBIENT) */
static bool initNsKeepCaps(cap_user_data_t cap_data)
{
	/* Copy all permitted caps to the inheritable set */
	for (const auto &i : capNames)
	{
		if (getPermitted(cap_data, i.val))
		{
			setInheritable(cap_data, i.val);
		}
	}
	// LOG_D("Adding the following capabilities to the inheritable set:%s", dbgmsg1.c_str());

	if (!setCaps(cap_data))
	{
		return false;
	}

	/* Make sure the inheritable set is preserved across execve via the ambient set */
	for (const auto &i : capNames)
	{
		if (!getPermitted(cap_data, i.val))
		{
			continue;
		}
		if (prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_RAISE, (unsigned long)i.val, 0UL, 0UL) ==
			-1)
		{
			printf("Error: prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_RAISE, %s)", i.name);
		}
	}
	// LOG_D("Added the following capabilities to the ambient set:%s", dbgmsg2.c_str());

	return true;
}

extern "C"

	int
	keepCaps()
{
	cap_user_data_t cap_data = getCaps();
	if (cap_data == NULL)
	{
		return 0;
	}

	/* Let's start with an empty inheritable set to avoid any mistakes */
	clearInheritable(cap_data);

	/*
	 * Remove all capabilities from the ambient set first. It works with newer kernel versions
	 * only, so don't panic() if it fails
	 */
	if (prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL, 0UL, 0UL, 0UL) == -1)
	{
		printf("prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL)");
	}

	if (initNsKeepCaps(cap_data))
	{
		return 1;
	}

	return 0;
}

extern "C"

	int
	handleCaps(int keepCaps)
{
	// cap_t caps;

	// printf("eUID = %ld;  eGID = %ld;  ",
	// 	   (long)geteuid(), (long)getegid());

	// caps = cap_get_proc();
	// printf("capabilities: %s\n", cap_to_text(caps, NULL));

	// ********************************************************

	cap_user_data_t cap_data = getCaps();
	if (cap_data == NULL)
	{
		return 0;
	}

	/* Let's start with an empty inheritable set to avoid any mistakes */
	clearInheritable(cap_data);

	/*
	 * Remove all capabilities from the ambient set first. It works with newer kernel versions
	 * only, so don't panic() if it fails
	 */
	if (prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL, 0UL, 0UL, 0UL) == -1)
	{
		printf("prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL)");
	}

	if (keepCaps)
	{
		return initNsKeepCaps(cap_data);
	}

	if (!setCaps(cap_data))
	{
		return 0;
	}

	/*
	 * Make sure all other caps (those which were not explicitly requested) are removed from the
	 * bounding set. We need to have CAP_SETPCAP to do that now
	 */
	if (getEffective(cap_data, CAP_SETPCAP))
	{
		for (const auto &i : capNames)
		{
			if (getInheritable(cap_data, i.val))
			{
				continue;
			}
			if (prctl(PR_CAPBSET_DROP, (unsigned long)i.val, 0UL, 0UL, 0UL) == -1)
			{
				printf("prctl(PR_CAPBSET_DROP, %s)", i.name);
				return 0;
			}
		}
	}

	// printf("eUID = %ld;  eGID = %ld;  ",
	// 	   (long)geteuid(), (long)getegid());

	// caps = cap_get_proc();
	// printf("capabilities: %s\n", cap_to_text(caps, NULL));

	return 1;
}