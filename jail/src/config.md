# uid/gid mapping

To add a custom (new) one, do not forget to set is_newidmap to true to use /usr/bin/newuidmap instead of writing directly to file /proc/pid/gid_map.
You should also check that uidmap is installed, see README.md from the root of Toastainer for installation instructions.

# nice_level

- Priority set in contain.rs::contain_prepare_env

From nsjail doc: "Set jailed process niceness (-20 is highest -priority, 19 is lowest). By default, set to 19"

See contain.md for more explanations


# Nsjail use of personnality

- http://man7.org/linux/man-pages/man2/personality.2.html

```c++
case "persona_addr_compat_layout":
	nsjconf->personality |= ADDR_COMPAT_LAYOUT; // With this flag set, provide legacy virtual address space layout.
	break;
case "persona_mmap_page_zero":
	nsjconf->personality |= MMAP_PAGE_ZERO; // Map page 0 as read-only (to support binaries that depend on this SVr4 behavior).
	break;
case "persona_read_implies_exec":
	nsjconf->personality |= READ_IMPLIES_EXEC; // With this flag set, PROT_READ implies PROT_EXEC for mmap(2).
	break;
case "persona_addr_limit_3gb":
	nsjconf->personality |= ADDR_LIMIT_3GB; // With this flag set, use 0xc0000000 as the offset at which to search a virtual memory chunk on mmap(2); otherwise use 0xffffe000.
	break;
case "persona_addr_no_randomize":
	nsjconf->personality |= ADDR_NO_RANDOMIZE; // With this flag set, disable address-space-layout randomization.
	break;
```

Set in contain.rs::contain_prepare_env called in contain.rs::contain_proc called in subproc.rs::subprocNewproc

# Notes

## Vec

- You can create a Vec for any element that implements the Clone trait