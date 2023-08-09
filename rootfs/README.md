# Pitfalls

## Internet access from inside

- In order to gain internet access from inside your container, you need to do: `echo "nameserver 8.8.8.8" > /etc/resolv.conf`.

# Ubuntu rootfs debug

APT uses user uid 105 and gid: 8, 50, 43, 42

When tar -xf ubuntu base rootfs from inside user namespace with inside 0 and outside 0:

```BASH
# tar -xf ubuntu-base-18.04-base-amd64.tar.gz
tar: dev/urandom: Cannot mknod: Operation not permitted
tar: dev/random: Cannot mknod: Operation not permitted
tar: dev/full: Cannot mknod: Operation not permitted
tar: dev/zero: Cannot mknod: Operation not permitted
tar: dev/tty: Cannot mknod: Operation not permitted
tar: dev/ptmx: Cannot mknod: Operation not permitted
tar: dev/null: Cannot mknod: Operation not permitted
tar: etc/gshadow: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: etc/shadow: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: run/utmp: Cannot change ownership to uid 0, gid 43: Invalid argument
tar: sbin/pam_extrausers_chkpwd: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: sbin/unix_chkpwd: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: usr/bin/expiry: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: usr/bin/chage: Cannot change ownership to uid 0, gid 42: Invalid argument
tar: usr/bin/wall: Cannot change ownership to uid 0, gid 5: Invalid argument
tar: var/log/lastlog: Cannot change ownership to uid 0, gid 43: Invalid argument
tar: var/log/wtmp: Cannot change ownership to uid 0, gid 43: Invalid argument
tar: var/log/btmp: Cannot change ownership to uid 0, gid 43: Invalid argument
tar: var/cache/apt/archives/partial: Cannot change ownership to uid 105, gid 0: Invalid argument
tar: var/lib/apt/lists/auxfiles: Cannot change ownership to uid 105, gid 0: Invalid argument
tar: var/lib/apt/lists/partial: Cannot change ownership to uid 105, gid 0: Invalid argument
tar: var/local: Cannot change ownership to uid 0, gid 50: Invalid argument
tar: var/mail: Cannot change ownership to uid 0, gid 8: Invalid argument
tar: Exiting with failure status due to previous errors
```

if we do this:

```RUST
jconf.uids[0].inside_id = 0;
jconf.uids[0].outside_id = 0;
jconf.uids[0].count = 150;
jconf.uids[0].is_newidmap = false;
jconf.gids[0].inside_id = 0;
jconf.gids[0].outside_id = 0;
jconf.gids[0].count = 150;
jconf.gids[0].is_newidmap = false;
```

then we only got for error:

```BASH
# tar -xf ubuntu-base-18.04-base-amd64.tar.gz
tar: dev/urandom: Cannot mknod: Operation not permitted
tar: dev/random: Cannot mknod: Operation not permitted
tar: dev/full: Cannot mknod: Operation not permitted
tar: dev/zero: Cannot mknod: Operation not permitted
tar: dev/tty: Cannot mknod: Operation not permitted
tar: dev/ptmx: Cannot mknod: Operation not permitted
tar: dev/null: Cannot mknod: Operation not permitted
tar: Exiting with failure status due to previous errors

# ls dev
fd  pts  shm  stderr  stdin  stdout
```

and for apt update:


```BASH
# apt update
E: setgroups 65534 failed - setgroups (22: Invalid argument)
E: setegid 65534 failed - setegid (22: Invalid argument)
Reading package lists... Done
E: setgroups 65534 failed - setgroups (22: Invalid argument)
E: setegid 65534 failed - setegid (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (22: Invalid argument)
E: Method http has died unexpectedly!
E: Sub-process http returned an error code (112)
```

if we do this:

```RUST
jconf.uids[0].inside_id = 0;
jconf.uids[0].outside_id = 0;
jconf.uids[0].count = 70000;
jconf.uids[0].is_newidmap = false;
jconf.gids[0].inside_id = 0;
jconf.gids[0].outside_id = 0;
jconf.gids[0].count = 70000;
jconf.gids[0].is_newidmap = false;
```

Then `apt update` works