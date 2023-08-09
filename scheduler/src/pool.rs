use jail::config::JailConf;

use disk::overlay_fs::OverlayDir;

/// The uts namespace is fixed for all toasters which do not have perms to modify it, both because of the droped CAP_SYS_ADMIN
/// and the seccomp policy not allowing the syscall sethostname
pub struct NamespacePool<'a> {
    store: Vec<Vec<Item<'a>>>,
}

pub struct Item<'a> {
    pub jconf: JailConf<'a>,
    pub ovdir: OverlayDir,
}

impl<'a> NamespacePool<'a> {
    /// new creates the fixed uts namespace and may panic if it cannot do so
    pub fn new(nb_pool: usize, pool_size: usize) -> NamespacePool<'a> {
        let mut pool = NamespacePool {
            store: Vec::with_capacity(nb_pool),
        };

        for _ in 0..nb_pool {
            pool.store.push(Vec::with_capacity(pool_size));
        }

        pool
    }

    pub fn len(&self, pool_index: usize) -> usize {
        self.store[pool_index].len()
    }

    pub fn push(&mut self, pool_index: usize, item: Item<'a>) {
        self.store[pool_index].push(item);
    }

    pub fn pop(&mut self, pool_index: usize) -> Item<'a> {
        self.store[pool_index].pop().unwrap()
    }
}
