// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Small system utility modules for usage by other modules.
//!

extern crate libc;
use libc::PATH_MAX;
use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::str::from_utf8;

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod ioctl;

#[macro_use]
pub mod macro_lib;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod epoll;

pub mod errno;
mod eventfd;
pub mod execv;
pub mod fcntl;
pub mod libnl;
pub mod mount;
pub mod num_cpu;
pub mod sched;
pub mod signal;
pub mod socket;
pub mod stat;
pub mod statfs;
pub mod statvfs;
mod struct_util;
mod terminal;
pub mod time;
pub mod uio;
pub mod unistd;
pub mod uts;

pub mod bindings;

pub use eventfd::*;
pub use ioctl::*;
pub use signal::*;
pub use struct_util::{read_struct, read_struct_slice};
pub use terminal::*;

mod syscall;
pub use syscall::*;

// *********** Path Traits

pub trait NixPath {
    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T;

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T;

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno>;
}

impl NixPath for str {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(OsStr::new(self))
    }

    fn len(&self) -> usize {
        NixPath::len(OsStr::new(self))
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        OsStr::new(self).with_nix_path(f)
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(self))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        OsStr::new(self).to_toast_path()
    }
}

impl NixPath for String {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(OsStr::new(self))
    }

    fn len(&self) -> usize {
        NixPath::len(OsStr::new(self))
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        OsStr::new(self).with_nix_path(f)
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(self))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        OsStr::new(self).to_toast_path()
    }
}

impl NixPath for OsStr {
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_bytes().with_nix_path(f)
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        if let Some(v) = self.to_str() {
            Ok(f(v))
        } else {
            Err(errno::Errno::from_i32(libc::EINVAL))
        }
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        self.as_bytes().to_toast_path()
    }
}

impl NixPath for CStr {
    fn is_empty(&self) -> bool {
        self.to_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.to_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        // Equivalence with the [u8] impl.
        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        Ok(f(self))
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(match self.to_str() {
            Ok(v) => v,
            Err(_) => return Err(errno::Errno::from_i32(libc::EINVAL)),
        }))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        // Equivalence with the [u8] impl.
        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        Ok(self)
    }
}

impl NixPath for [u8] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const libc::c_char)))
                }
            }
        }
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(match from_utf8(self) {
            Ok(v) => v,
            Err(_) => return Err(errno::Errno::from_i32(libc::EINVAL)),
        }))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(CStr::from_ptr(buf.as_ptr() as *const libc::c_char))
                }
            }
        }
    }
}

impl NixPath for Vec<u8> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const libc::c_char)))
                }
            }
        }
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(match from_utf8(self) {
            Ok(v) => v,
            Err(_) => return Err(errno::Errno::from_i32(libc::EINVAL)),
        }))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(CStr::from_ptr(buf.as_ptr() as *const libc::c_char))
                }
            }
        }
    }
}

impl NixPath for &[u8] {
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const libc::c_char)))
                }
            }
        }
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        Ok(f(match from_utf8(self) {
            Ok(v) => v,
            Err(_) => return Err(errno::Errno::from_i32(libc::EINVAL)),
        }))
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(errno::Errno::from_i32(libc::EINVAL));
        }

        match self.iter().position(|b| *b == 0) {
            /* check that there is no null bytes inside the path since it would be a problem in C */
            Some(_) => Err(errno::Errno::from_i32(libc::EINVAL)),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(CStr::from_ptr(buf.as_ptr() as *const libc::c_char))
                }
            }
        }
    }
}

impl NixPath for Path {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_os_str().with_nix_path(f)
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        if let Some(v) = self.to_str() {
            Ok(f(v))
        } else {
            Err(errno::Errno::from_i32(libc::EINVAL))
        }
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        self.as_os_str().to_toast_path()
    }
}

impl NixPath for PathBuf {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_os_str().with_nix_path(f)
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        if let Some(v) = self.to_str() {
            Ok(f(v))
        } else {
            Err(errno::Errno::from_i32(libc::EINVAL))
        }
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        self.as_os_str().to_toast_path()
    }
}

/// Treats `None` as an empty string.
impl<'a, NP: ?Sized + NixPath> NixPath for Option<&'a NP> {
    fn is_empty(&self) -> bool {
        self.map_or(true, NixPath::is_empty)
    }

    fn len(&self) -> usize {
        self.map_or(0, NixPath::len)
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&CStr) -> T,
    {
        if let Some(nix_path) = *self {
            nix_path.with_nix_path(f)
        } else {
            unsafe { CStr::from_ptr("\0".as_ptr() as *const _).with_nix_path(f) }
        }
    }

    fn with_nix_path_str<T, F>(&self, f: F) -> Result<T, errno::Errno>
    where
        F: FnOnce(&str) -> T,
    {
        if let Some(nix_path) = *self {
            nix_path.with_nix_path_str(f)
        } else {
            "".with_nix_path_str(f)
        }
    }

    fn to_toast_path(&self) -> Result<&CStr, errno::Errno> {
        if let Some(toast_path) = *self {
            toast_path.to_toast_path()
        } else {
            unsafe { CStr::from_ptr("\0".as_ptr() as *const _).to_toast_path() }
        }
    }
}
