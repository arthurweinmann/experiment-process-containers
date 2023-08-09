// use jail::config::{JailConf, MountT};
// use jail::error::Result;
// use jail::subproc::run_monitor_child;
// use sys_util::NixPath;

// use std::ffi::{CStr, CString};

// // ty: alpine | ubuntu
// pub fn jconf_for_rootfs<'a>(cmd: &'a str, arg: &'a str, root_dir: &str) -> JailConf<'a> {
//     let mut jconf = JailConf::new();

//     jconf
//         .keep_caps()
//         .with_chroot(root_dir)
//         .chroot_is_rw()
//         .with_hostname("toastate")
//         .with_cwd("/root")
//         // we did not yet test that passing env to execv works
//         .with_env(vec![CStr::from_bytes_with_nul_unchecked(
//             b"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\0",
//         )]) // for alpine, see for ubuntu
//         .clone_newuser()
//         .clone_newns()
//         .clone_newuts()
//         .clone_newpid() // needed for mounting /proc and to be sure all process are terminated
//         .execute()
//         .prepare_env_in_child()
//         .handle_fds_in_child();

//     let mut v = vec![CStr::from_bytes_with_nul_unchecked(b"/bin/sh\0"), CStr::from_bytes_with_nul(cmd.as_bytes()).unwrap()];
//     if !arg.is_empty() {
//         v.push(CStr::from_bytes_with_nul(arg.as_bytes()).unwrap());
//     }

//     jconf.with_exevc(CString::new("/bin/sh").unwrap(), v);

//     jconf.with_default_mounts();

//     jconf.instantiate();

//     jconf
// }

// pub fn create_image<P2: ?Sized + NixPath, P3: ?Sized + NixPath>(
//     sh_install_script: Option<&P2>,
//     image_folder: &P3,
//     interactive: bool,
// ) -> Result<()> {
//     let res: Result<()> = image_folder.with_nix_path_str(|chroot| {
//         let mut jconf = JailConf::new();
//         jconf.use_jailconf_only = true;
//         jconf
//             .keep_caps()
//             .with_chroot(chroot)
//             .chroot_is_rw()
//             .with_hostname("toastate")
//             .with_cwd("/root")
//             // we did not yet test that passing env to execv works
//             .with_env(vec![CStr::from_bytes_with_nul_unchecked(
//                 b"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\0",
//             )]) // for alpine, see for ubuntu
//             .clone_newuser()
//             .clone_newns()
//             .clone_newuts()
//             .clone_newpid() // needed for mounting /proc and to be sure all process are terminated
//             .execute()
//             .prepare_env_in_child()
//             .handle_fds_in_child();

//         if !interactive {
//             if let Some(script) = sh_install_script {
//                 script.with_nix_path_str(|script_str| {
//                     jconf.with_mnt(MountT {
//                         src: Some(CString::new(script_str.as_bytes()).unwrap()),
//                         dst: MountT::transform_dst(&jconf, "root/install.sh"),
//                         dst_in_pivot: CString::new("/root/install.sh".as_bytes()).unwrap(),
//                         fs_type: None,
//                         options: None,
//                         flags: libc::MS_BIND | libc::MS_REC | libc::MS_PRIVATE,
//                         is_dir: false,
//                         is_symlink: false,
//                         is_mandatory: true,
//                         mounted: false,
//                     });
//                 })?;

//                 jconf.with_exevc(
//                     CString::new("/bin/sh").unwrap(),
//                     vec![
//                         CStr::from_bytes_with_nul_unchecked(b"/bin/sh\0"),
//                         CStr::from_bytes_with_nul_unchecked(b"./install.sh\0"),
//                     ],
//                 );
//             } else {
//                 return Err(
//                     "You must provide a path to a script when not in interactive mode".into(),
//                 );
//             }
//         } else {
//             jconf.with_exevc(
//                 CString::new("/bin/sh").unwrap(),
//                 vec![
//                     CStr::from_bytes_with_nul_unchecked(b"/bin/sh\0"),
//                     CStr::from_bytes_with_nul_unchecked(b"-i\0"),
//                 ],
//             );
//         }

//         jconf.with_default_mounts();

//         jconf.instantiate();

//         run_monitor_child(&mut jconf, vec![], vec![], None).unwrap();

//         Ok(())
//     })?;

//     res
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs::{create_dir, remove_dir_all};
//     use std::path::Path;

//     // export RUST_BACKTRACE=full; cargo test -- --nocapture --test-threads=1 create_image
//     // #[test]
//     // fn test_create_image() {
//     //     let chroot_dir = "/home/arthurbuntu/rust/toastainer/rootfs/alpine";
//     //     if Path::new(chroot_dir).exists() {
//     //         remove_dir_all(chroot_dir).unwrap();
//     //     }
//     //     create_dir(chroot_dir).unwrap();
//     //     create_image("http://dl-cdn.alpinelinux.org/alpine/v3.10/releases/x86_64/alpine-minirootfs-3.10.3-x86_64.tar.gz", Some("/home/arthurbuntu/rust/toastainer/rootfs/src/test.sh"), chroot_dir, true, true).unwrap();
//     // }
// }
