use std::os::unix::net::{UnixListener, UnixStream};

use disk::overlay_fs::OverlayDir;
use jail::config::JailConf;

use super::time_utils::timestamp_second;

// Here's a macro that supports arrays of length 0 through 8, and powers of 2 up to 64:
macro_rules! array {
    (@accum (0, $($_es:expr),*) -> ($($body:tt)*))
        => {array!(@as_expr [$($body)*])};
    (@accum (1, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (2, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)* $($es,)*))};
    (@accum (3, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (2, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (4, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (2, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (5, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (6, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)* $($es,)*))};
    (@accum (7, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)* $($es,)* $($es,)*))};
    (@accum (8, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (16, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (8, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (32, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (16, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (64, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (32, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (128, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (64, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (256, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (128, $($es,)* $($es),*) -> ($($body)*))};

    (@as_expr $e:expr) => {$e};

    [$e:expr; $n:tt] => { array!(@accum ($n, $e) -> ()) };
}

pub struct HashTable<'a> {
    modulo: i32,
    store: [Vec<Item<'a>>; 64],
    by_exe_id: [Vec<[u32; 2]>; 64],
    kill_after: Vec<Option<[u64; 2]>>,
}

pub struct Item<'a> {
    pub pid: i32,
    pub exe_id: u32,
    pub jconf: JailConf<'a>,
    pub ovdir: Option<OverlayDir>,
    pub toaster_listener: Option<UnixListener>,
}

impl<'a> HashTable<'a> {
    pub fn new() -> Self {
        let mut h = HashTable {
            modulo: 64 - 1,                           // array size minus 1
            store: array![Vec::with_capacity(4); 64], // array size mut be a power of two
            by_exe_id: array![Vec::with_capacity(4); 64],
            kill_after: Vec::with_capacity(64),
        };

        for _ in 0..64 {
            h.kill_after.push(None);
        }

        h
    }

    pub fn put_in_kill_after(&mut self, pid: u64, deadline: u64) -> Vec<u32> {
        let now = timestamp_second(0);

        let mut ret = Vec::with_capacity(4);

        let mut done = false;

        let mut s = [pid, deadline];

        'outer: for tab in self.kill_after.iter_mut() {
            if let Some(tab2) = tab {
                if tab2[1] <= now {
                    std::mem::swap(tab2, &mut s);
                    done = true;
                    ret.push(tab2[0] as u32);
                    break 'outer;
                }
            } else {
                tab.replace(s);
                done = true;
                break 'outer;
            }
        }

        if !done {
            self.kill_after.push(Some(s));
        }

        ret
    }

    pub fn extract_kill_after(&mut self) -> Vec<u32> {
        let now = timestamp_second(0);

        let mut ret = Vec::with_capacity(4);

        'outer: for tab in self.kill_after.iter_mut() {
            if let Some(tab2) = tab {
                if tab2[1] <= now {
                    ret.push(tab2[0] as u32);
                    tab.take();
                    break 'outer;
                }
            }
        }

        ret
    }

    pub fn lookup_exe_id(&self, exe_id: u32) -> Option<u32> {
        for tab in self.by_exe_id[(exe_id & self.modulo as u32) as usize].iter() {
            if tab[0] == exe_id as u32 {
                return Some(tab[1]);
            }
        }

        None
    }

    pub fn pop_exe_id(&mut self, exe_id: u32) -> Option<u32> {
        let vector = &mut self.by_exe_id[(exe_id & self.modulo as u32) as usize];
        let mut k: usize = 0;
        let mut found = false;
        for (pos, tab) in vector.iter().enumerate() {
            if tab[0] == exe_id as u32 {
                k = pos;
                found = true;
                break;
            }
        }
        if found {
            return Some(vector.swap_remove(k)[1]);
        }

        None
    }

    pub fn insert(&mut self, item: Item<'a>) {
        self.store[(item.pid & self.modulo) as usize].push(item);
    }

    pub fn insert_exe_id(&mut self, exe_id: i32, pid: u32) {
        self.by_exe_id[(exe_id & self.modulo) as usize].push([exe_id as u32, pid]);
    }

    pub fn lookup(&self, key: i32) -> Option<&Item> {
        for tab in self.store[(key & self.modulo) as usize].iter() {
            if tab.pid == key {
                return Some(tab);
            }
        }

        None
    }

    pub fn borrow(&self, key: i32) -> Option<&Item> {
        for tab in self.store[(key & self.modulo) as usize].iter() {
            if tab.pid == key {
                return Some(tab);
            }
        }

        None
    }

    pub fn pop(&mut self, key: i32) -> Option<Item> {
        let vector = &mut self.store[(key & self.modulo) as usize];
        let mut k: usize = 0;
        let mut found = false;
        for (pos, tab) in vector.iter().enumerate() {
            if tab.pid == key {
                k = pos;
                found = true;
                break;
            }
        }
        if found {
            return Some(vector.swap_remove(k));
        }

        None
    }

    pub fn delete(&mut self, key: i32) {
        let vector = &mut self.store[(key & self.modulo) as usize];
        let mut k: usize = 0;
        let mut found = false;
        for (pos, tab) in vector.iter().enumerate() {
            if tab.pid == key {
                k = pos;
                found = true;
                break;
            }
        }
        if found {
            let l = vector.len() - 1;
            if k < l {
                // https://stackoverflow.com/questions/27904864/what-does-cannot-move-out-of-index-of-mean
                vector[k] = vector.pop().unwrap();
            } else {
                vector.truncate(l);
            }
        }
    }
}
