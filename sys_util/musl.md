# MUSL C standard library

- See https://en.wikipedia.org/wiki/Musl:

```
Some Linux distributions that can use musl as the standard C library include Alpine Linux, Dragora 3, Gentoo Linux, OpenWRT, Sabotage[6] Morpheus Linux[7] and Void Linux. For binaries that have been linked against glibc, gcompat[8] can be used to execute them on Musl-based distros.
```

- See also https://www.musl-libc.org/

- it is used by seccomp and sys_util package as `#[cfg(target_env = "musl")]`.

# Benchmarks

It seems musl makes Rust slow due to a bug. Track this bug: https://andygrove.io/2020/05/why-musl-extremely-slow/ 