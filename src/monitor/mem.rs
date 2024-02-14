use std::io::ErrorKind;
use std::sync::Arc;
use std::{io, time::Duration};

use async_lock::{RwLock, RwLockUpgradableReadGuard};
use procfs::{Current, Meminfo, ProcResult};
use tokio::{task, time};
use tracing::error;

use crate::notify::Notify;

#[trait_variant::make(MemoryInfo: Send)]
pub trait LocalMemoryInfo {
    async fn get_memory_total(&self) -> io::Result<usize>;

    async fn get_memory_available(&self) -> io::Result<usize>;
}

#[derive(Debug)]
pub struct ProcfsMemoryWatch<N> {
    notify: N,
    interval: Duration,
    max_use_ratio: f64,
    mem_info_cache: Arc<RwLock<Option<Meminfo>>>,
}

#[derive(Debug, Clone)]
pub struct ProcfsMemoryInfo {
    mem_info_cache: Arc<RwLock<Option<Meminfo>>>,
}

impl<N> ProcfsMemoryWatch<N> {
    pub fn new(notify: N, interval: Duration, max_use: f64) -> (Self, ProcfsMemoryInfo) {
        let memory_watch = Self {
            notify,
            interval,
            max_use_ratio: max_use,
            mem_info_cache: Default::default(),
        };
        let memory_info = ProcfsMemoryInfo {
            mem_info_cache: memory_watch.mem_info_cache.clone(),
        };

        (memory_watch, memory_info)
    }
}

async fn get_memory_info() -> ProcResult<Meminfo> {
    task::spawn_blocking(Meminfo::current).await.unwrap()
}

impl MemoryInfo for ProcfsMemoryInfo {
    async fn get_memory_total(&self) -> io::Result<usize> {
        let mem_info_cache = self.mem_info_cache.upgradable_read().await;
        match &*mem_info_cache {
            None => {
                let mem_info = get_memory_info()
                    .await
                    .map_err(|err| io::Error::new(ErrorKind::Other, err))?;
                let total = mem_info.mem_total;

                let mut mem_info_cache = RwLockUpgradableReadGuard::upgrade(mem_info_cache).await;
                *mem_info_cache = Some(mem_info);

                Ok(total as _)
            }

            Some(mem_info) => Ok(mem_info.mem_total as _),
        }
    }

    async fn get_memory_available(&self) -> io::Result<usize> {
        let mem_info_cache = self.mem_info_cache.upgradable_read().await;
        match &*mem_info_cache {
            None => {
                let mem_info = get_memory_info()
                    .await
                    .map_err(|err| io::Error::new(ErrorKind::Other, err))?;
                let available = mem_info.mem_available.expect("mem available is None");

                let mut mem_info_cache = RwLockUpgradableReadGuard::upgrade(mem_info_cache).await;
                *mem_info_cache = Some(mem_info);

                Ok(available as _)
            }

            Some(mem_info) => Ok(mem_info.mem_available.expect("mem available is None") as _),
        }
    }
}

impl<N: Notify> ProcfsMemoryWatch<N> {
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut interval = time::interval(self.interval);

        loop {
            interval.tick().await;

            let mem_info = match get_memory_info().await {
                Ok(mem_info) => mem_info,
                Err(err) => {
                    error!(%err, "get memory info failed");

                    let _ = self
                        .notify
                        .notify_self_error(format!("get memory info failed: {err}"))
                        .await;

                    continue;
                }
            };

            let total = mem_info.mem_total;
            let used = total - mem_info.mem_available.expect("mem available is None");

            *self.mem_info_cache.write().await = Some(mem_info);

            if (used as f64 / total as f64) > self.max_use_ratio {
                if let Err(err) = self.notify.notify_memory(total as _, used as _).await {
                    error!(%err, "notify memory failed");
                }
            }
        }
    }
}
