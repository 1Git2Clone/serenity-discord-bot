use std::sync::{Mutex, MutexGuard, PoisonError};

pub fn process_mutex<'src, T, F, Ret>(
    mutex: &'src Mutex<T>,
    f: F,
) -> Result<Ret, PoisonError<MutexGuard<'src, T>>>
where
    F: FnOnce(MutexGuard<'src, T>) -> Ret,
{
    let mutex_guard = mutex.lock()?;
    Ok(f(mutex_guard))
}
