use std::path::{Path, PathBuf};

pub fn pathbuf_to_bytes<'a>(path: &'a PathBuf) -> &'a[u8] {
    path.to_str().unwrap().as_bytes()
}

pub fn path_to_bytes<'a>(path: &'a Path) -> &'a[u8] {
    path.to_str().unwrap().as_bytes()
}