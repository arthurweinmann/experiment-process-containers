use std::ffi::{CString};

extern {
    fn customls(input: *const libc::c_char);
}

pub fn custom_ls(folder: &str) {
    let fold = CString::new(folder.as_bytes()).unwrap();
    unsafe {customls(fold.as_ptr())};
}