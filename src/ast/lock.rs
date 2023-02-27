use named_lock::{NamedLock, NamedLockGuard};
use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};
use whoami::username;

const PACKAGE_LOCK_NAME: &str = "move_pkg_lock";
static PACKAGE_THREAD_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static PACKAGE_PROCESS_MUTEX: Lazy<NamedLock> = Lazy::new(|| {
    let user_lock_file = format!("{}_{}", PACKAGE_LOCK_NAME, username());
    NamedLock::create(user_lock_file.as_str()).unwrap()
});

pub(crate) struct PackageLock {
    thread_lock: MutexGuard<'static, ()>,
    process_lock: NamedLockGuard<'static>,
}

impl PackageLock {
    pub(crate) fn lock() -> PackageLock {
        let thread_lock = PACKAGE_THREAD_MUTEX.lock().unwrap();
        let process_lock = PACKAGE_PROCESS_MUTEX.lock().unwrap();
        Self {
            thread_lock,
            process_lock,
        }
    }

    pub(crate) fn unlock(self) {
        let Self {
            thread_lock,
            process_lock,
        } = self;
        drop(process_lock);
        drop(thread_lock);
    }
}
