# execve

- see http://man7.org/linux/man-pages/man2/execve.2.html

execve() executes the program referred to by pathname.  This causes
the program that is currently being run by the calling process to be
replaced with a new program, with newly initialized stack, heap, and
(initialized and uninitialized) data segments.

execve() does not return on success, and the text, initialized data,
uninitialized data (bss), and stack of the calling process are over‐
written according to the contents of the newly loaded program.

# Exec Caveat

- See https://manpages.debian.org/testing/manpages-dev/execveat.2.en.html

In addition to the reasons explained in openat(2), the execveat() system call is also needed to allow fexecve(3) to be implemented on systems that do not have the /proc filesystem mounted. (-> nodejs solution ?)
For the same reasons described in fexecve(3), the natural idiom when using execveat() is to set the close-on-exec flag on dirfd. (But see BUGS.)

## BUGS

The ENOENT error described above means that it is not possible to set the close-on-exec flag on the file descriptor given to a call of the form:
execveat(fd, "", argv, envp, AT_EMPTY_PATH);

However, the inability to set the close-on-exec flag means that a file descriptor referring to the script leaks through to the script itself. As well as wasting a file descriptor, this leakage can lead to file-descriptor exhaustion in scenarios where scripts recursively employ execveat().
