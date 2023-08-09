use super::errno::Errno;
use super::NixPath;

// works, but no NULL pointer
pub fn mount<
    P1: ?Sized + NixPath,
    P2: ?Sized + NixPath,
    P3: ?Sized + NixPath,
    P4: ?Sized + NixPath,
>(
    source: Option<&P1>,
    target: &P2,
    fstype: Option<&P3>,
    flags: libc::c_ulong,
    data: Option<&P4>,
) -> Result<(), Errno> {
    let res = source.with_nix_path(|source| {
        target.with_nix_path(|target| {
            fstype.with_nix_path(|fstype| {
                data.with_nix_path(|data| {
                    println!("{:?} {:?} {:?} {:?}", source, target, fstype, data);
                    unsafe {
                        libc::mount(
                            source.as_ptr(),
                            target.as_ptr(),
                            fstype.as_ptr(),
                            flags,
                            data.as_ptr() as *const libc::c_void,
                        )
                    }
                })
            })
        })
    })????;

    Errno::result(res).map(drop)
}

// does not work, why ?
pub fn mount_with_null<
    P1: ?Sized + NixPath,
    P2: ?Sized + NixPath,
    P3: ?Sized + NixPath,
    P4: ?Sized + NixPath,
>(
    source: Option<&P1>,
    target: &P2,
    fstype: Option<&P3>,
    flags: libc::c_ulong,
    data: Option<&P4>,
) -> Result<(), Errno> {
    let src: *const libc::c_char;
    if let Some(k) = source {
        println!("src: {:?}", k.to_toast_path()?);
        src = k.to_toast_path()?.as_ptr();
    } else {
        src = std::ptr::null();
    }

    let fs: *const libc::c_char;
    if let Some(k) = fstype {
        println!("fs: {:?}", k.to_toast_path()?);
        fs = k.to_toast_path()?.as_ptr();
    } else {
        fs = std::ptr::null();
    }

    let dat: *const libc::c_void;
    if let Some(k) = data {
        println!("dat: {:?}", k.to_toast_path()?);
        dat = k.to_toast_path()?.as_ptr() as *const libc::c_void;
    } else {
        dat = std::ptr::null();
    }

    println!("{:?} {:?} {:?} {:?}", src, target.to_toast_path()?, fs, dat);

    let res = unsafe { libc::mount(src, target.to_toast_path()?.as_ptr(), fs, flags, dat) };

    Errno::result(res).map(drop)
}

pub fn umount<P: ?Sized + NixPath>(target: &P) -> Result<(), Errno> {
    let res = target.with_nix_path(|cstr| unsafe { libc::umount(cstr.as_ptr()) })?;

    Errno::result(res).map(drop)
}

pub fn umount2<P: ?Sized + NixPath>(target: &P, flags: libc::c_int) -> Result<(), Errno> {
    let res = target.with_nix_path(|cstr| unsafe { libc::umount2(cstr.as_ptr(), flags) })?;

    Errno::result(res).map(drop)
}
