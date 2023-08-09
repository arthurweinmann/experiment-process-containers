use std::ffi::{CStr, CString};

use std::str;

use super::config::JailConf;
use super::error::Result;

pub fn extract_u16(mess: &[u8], offset: usize) -> u16 {
    (mess[offset + 1] as u16) | ((mess[offset] as u16) << 8)
}

pub fn extract_i32(mess: &[u8], offset: usize) -> i32 {
    ((mess[offset + 3] as u32)
        | ((mess[offset + 2] as u32) << 8)
        | ((mess[offset + 1] as u32) << 16)
        | ((mess[offset] as u32) << 24)) as i32
}

pub fn extract_u32(mess: &[u8], offset: usize) -> u32 {
    (mess[offset + 3] as u32)
        | ((mess[offset + 2] as u32) << 8)
        | ((mess[offset + 1] as u32) << 16)
        | ((mess[offset] as u32) << 24)
}

pub fn extract_u64(mess: &[u8], offset: usize) -> u64 {
    (mess[offset + 7] as u64)
        | ((mess[offset + 6] as u64) << 8)
        | ((mess[offset + 5] as u64) << 16)
        | ((mess[offset + 4] as u64) << 24)
        | ((mess[offset + 3] as u64) << 32)
        | ((mess[offset + 2] as u64) << 40)
        | ((mess[offset + 1] as u64) << 48)
        | ((mess[offset] as u64) << 56)
}

pub fn put_u32(mess: &mut [u8], offset: usize, v: u32) {
    mess[offset] = (v >> 24) as u8;
    mess[offset + 1] = (v >> 16) as u8;
    mess[offset + 2] = (v >> 8) as u8;
    mess[offset + 3] = v as u8;
}

pub fn put_u32_vec_capacity(mess: &mut Vec<u8>, v: u32) {
    mess.push((v >> 24) as u8);
    mess.push((v >> 16) as u8);
    mess.push((v >> 8) as u8);
    mess.push(v as u8);
}

pub fn put_u16(mess: &mut [u8], offset: usize, v: u16) {
    mess[offset] = (v >> 8) as u8;
    mess[offset + 1] = v as u8;
}

pub fn string_from_bytes<'a>(b: &'a [u8], offset: usize, len: usize) -> &'a str {
    unsafe { str::from_utf8_unchecked(&b[offset..offset + len]) }
}

pub fn parse_uint32_cstr(u1: &CStr) -> u32 {
    (unsafe { std::str::from_utf8_unchecked(u1.to_bytes()) })
        .parse::<u32>()
        .unwrap()
}

/* ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++*/

pub fn parse_create_image_mess(mess: &[u8]) -> Result<(CString, bool, u32)> {
    let mut offset: usize = 6;

    let exe_id = extract_u32(mess, 1);

    let base_image_len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    if offset + base_image_len > mess.len() {
        println!("base_image_len: {}", base_image_len);
        return Err("incomplete message".into());
    }

    Ok((
        unsafe { CString::from_vec_unchecked(mess[offset..offset + base_image_len].to_vec()) },
        mess[5] == 1,
        exe_id,
    ))
}

pub fn parse_kill_mess(mess: &[u8]) -> Result<(u32, u64)> {
    let mut offset: usize = 1;

    let exe_id = extract_u32(mess, offset);
    offset = offset + 4;

    let timeout_sec = extract_u64(mess, offset);
    // offset = offset + 8;

    Ok((exe_id, timeout_sec))
}

pub fn create_pooled_wake_up_mess(jconf: &mut JailConf) -> Vec<u8> {
    let mut mess = Vec::with_capacity(1024);
    let mut offset = 0;

    unsafe { mess.set_len(4) };

    put_u16(&mut mess, 2, jconf.cwd.len() as u16);
    mess.append(unsafe { jconf.cwd.as_mut_vec() }); // after that cwd is an empty string

    if mess.len() + 2 > mess.len() {
        mess.reserve(mess.len() * 2)
    }
    unsafe { mess.set_len(mess.len() + 2) };

    if let Some(exec_file) = jconf.exec_file.take() {
        offset = mess.len() - 2;
        put_u16(&mut mess, offset, exec_file.as_bytes().len() as u16);
        mess.append(&mut exec_file.into_bytes());

        if let Some(mut argv) = jconf.argv.take() {
            if mess.len() + 2 > mess.len() {
                mess.reserve(mess.len() * 2)
            }
            unsafe { mess.set_len(mess.len() + 2) };

            offset = mess.len() - 2;
            put_u16(&mut mess, offset, argv.len() as u16);

            if argv.len() > 0 {
                for a in argv.drain(0..).into_iter() {
                    if mess.len() + 2 > mess.len() {
                        mess.reserve(mess.len() * 2)
                    }
                    unsafe { mess.set_len(mess.len() + 2) };

                    offset = mess.len() - 2;
                    put_u16(&mut mess, offset, a.as_bytes().len() as u16);
                    mess.append(&mut a.into_bytes());
                }
            }
        }
    } else {
        offset = mess.len() - 2;
        put_u16(&mut mess, offset, 0);
    }

    if mess.len() + 2 > mess.len() {
        mess.reserve(mess.len() * 2)
    }
    unsafe { mess.set_len(mess.len() + 2) };

    if let Some(mut env) = jconf.env.take() {
        offset = mess.len() - 2;
        put_u16(&mut mess, offset, env.len() as u16);

        if env.len() > 0 {
            for e in env.drain(0..).into_iter() {
                mess.push(e.as_bytes().len() as u8);
                mess.append(&mut e.into_bytes());
            }
        }
    } else {
        offset = mess.len() - 2;
        put_u16(&mut mess, offset, 0);
    }

    // put total mess len at the beginning
    offset = mess.len() - 2;
    put_u16(&mut mess, 0, offset as u16);

    mess
}

pub fn parse_pooled_wake_up(
    mess: &[u8],
) -> (
    String,
    Option<CString>,
    Option<Vec<CString>>,
    Option<Vec<CString>>,
) {
    let mut offset: usize = 0;

    let mut u16len = extract_u16(&mess, offset) as usize;
    offset = offset + 2;

    let cwd = unsafe { String::from_utf8_unchecked(mess[offset..offset + u16len].to_vec()) };
    offset = offset + u16len;

    u16len = extract_u16(&mess, offset) as usize;
    offset = offset + 2;

    let mut command_name = None;
    let mut command_args = None;
    if u16len > 0 {
        command_name =
            Some(unsafe { CString::from_vec_unchecked(mess[offset..offset + u16len].to_vec()) });
        offset = offset + u16len;

        let u16len = extract_u16(&mess, offset) as usize;
        offset = offset + 2;

        if u16len > 0 {
            let mut command_args_vec = Vec::with_capacity(u16len);

            for _ in 0..u16len {
                let arg_size = extract_u16(&mess, offset) as usize;
                offset = offset + 2;

                command_args_vec.push(unsafe {
                    CString::from_vec_unchecked(mess[offset..offset + arg_size].to_vec())
                });

                offset = offset + arg_size;
            }

            command_args = Some(command_args_vec);
        }
    }

    u16len = extract_u16(&mess, offset) as usize;
    offset = offset + 2;

    let mut env = None;
    if u16len > 0 {
        let mut env_vec = Vec::with_capacity(u16len);

        for _ in 0..u16len {
            let env_size = mess[offset] as usize;
            offset = offset + 1;

            env_vec.push(unsafe {
                CString::from_vec_unchecked(mess[offset..offset + env_size].to_vec())
            });

            offset = offset + env_size;
        }

        env = Some(env_vec);
    }

    (cwd, command_name, command_args, env)
}

/// if no command name, put in pool with exe id
/// in case no command is supplied and this toaster is to be added to a pool
/// if command name provided, first argv must also be the command name
/// mount_slave must be set to true if you intend to mount overlay after a toaster is created and in pool
/// otherwise set it to false even if you mount overlay at execution time
pub fn parse_toaster_command<'a>(
    mess: &'a [u8],
) -> (
    u32,
    u16,
    CString,
    CString,
    &'a [u8],
    Option<&'a [u8]>,
    String,
    Option<&'a str>,
    bool,
    bool,
    bool,
    bool,
    Option<CString>,
    Option<Vec<CString>>,
    Option<Vec<CString>>,
    CString,
) {
    let mut offset: usize = 1;

    let pool = extract_u16(mess, offset);
    offset = offset + 2;

    let exe_id = extract_u32(mess, offset);
    offset = offset + 4;

    let mut u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let uid = unsafe { CString::from_vec_unchecked(mess[offset..offset + u16len].to_vec()) };
    offset = offset + u16len;

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let btrfs_file_system =
        unsafe { CString::from_vec_unchecked(mess[offset..offset + u16len].to_vec()) };
    offset = offset + u16len;

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let overlay_dir = &mess[offset..offset + u16len];
    offset = offset + u16len;

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let mut lower_dirs = None;
    if u16len > 0 {
        lower_dirs = Some(&mess[offset..offset + u16len]);
        offset = offset + u16len;
    }

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let cwd = unsafe { String::from_utf8_unchecked(mess[offset..offset + u16len].to_vec()) };
    offset = offset + u16len;

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let mut log_path = None;
    if u16len > 0 {
        log_path = Some(unsafe { std::str::from_utf8_unchecked(&mess[offset..offset + u16len]) });
        offset = offset + u16len;
    }

    let is_socket_stdin = mess[offset] & 2 > 0;
    let is_log_socket = mess[offset] & 8 > 0;
    let std_only = mess[offset] & 32 > 0;
    let admin = mess[offset] & 128 > 0;
    offset = offset + 1;

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let mut command_name = None;
    let mut command_args = None;
    if u16len > 0 {
        command_name =
            Some(unsafe { CString::from_vec_unchecked(mess[offset..offset + u16len].to_vec()) });
        offset = offset + u16len;

        let u16len = extract_u16(mess, offset) as usize;
        offset = offset + 2;

        if u16len > 0 {
            let mut command_args_vec = Vec::with_capacity(u16len);

            for _ in 0..u16len {
                let arg_size = extract_u16(mess, offset) as usize;
                offset = offset + 2;

                command_args_vec.push(unsafe {
                    CString::from_vec_unchecked(mess[offset..offset + arg_size].to_vec())
                });

                offset = offset + arg_size;
            }

            command_args = Some(command_args_vec);
        }
    }

    u16len = extract_u16(mess, offset) as usize;
    offset = offset + 2;

    let mut env = None;
    if u16len > 0 {
        let mut env_vec = Vec::with_capacity(u16len);

        for _ in 0..u16len {
            let env_size = mess[offset] as usize;
            offset = offset + 1;

            env_vec.push(unsafe {
                CString::from_vec_unchecked(mess[offset..offset + env_size].to_vec())
            });

            offset = offset + env_size;
        }

        env = Some(env_vec);
    }

    let ip = unsafe {
        CString::from_vec_unchecked(mess[offset + 1..offset + 1 + mess[offset] as usize].to_owned())
    };

    (
        exe_id,
        pool,
        uid,
        btrfs_file_system,
        overlay_dir,
        lower_dirs,
        cwd,
        log_path,
        is_log_socket,
        is_socket_stdin,
        std_only,
        admin,
        command_name,
        command_args,
        env,
        ip,
    )
}
