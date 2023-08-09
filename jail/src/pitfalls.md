# You must waitpid processes you create through clone or fork
- If you do not waitpid a process created with clone or fork, when this child process terminates, it goes in "zombie" mode and is marked \<defunct\>.

- Processes marked \<defunct\> are dead processes (so-called 'zombies') that remain because their parent has not destroyed them properly. 
These processes will be destroyed by init(8) if the parent process exits
The parade is to waitpid: (https://stackoverflow.com/questions/13331570/force-the-parent-to-reap-a-child-process-defunct)

- A child that terminates, but has not been waited for becomes a "zombie". The kernel maintains a minimal set of information about the 
zombie process (PID, termination status, resource usage information) in order to allow the parent to later perform a wait to obtain 
information about the child. As long as a zombie is not removed from the system via a wait, it will consume a slot in the kernel process table,
 and if this table fills, it will not be possible to create further processes.

