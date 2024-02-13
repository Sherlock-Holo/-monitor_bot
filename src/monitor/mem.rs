use std::{io, time::Duration};

use procfs::{Current, Meminfo};
use tokio::{task, time};
use tracing::error;

use crate::notify::Notify;

#[trait_variant::make(MemoryInfo: Send)]
pub trait LocalMemoryInfo {
    async fn get_memory_total(&mut self) -> io::Result<usize>;

    async fn get_memory_available(&mut self) -> io::Result<usize>;
}

#[derive(Debug)]
pub struct ProcfsMemoryWatch<N> {
    notify: N,
    interval: Duration,
    max_use_ratio: f64,
}

impl<N> ProcfsMemoryWatch<N> {
    pub fn new(notify: N, interval: Duration, max_use: f64) -> Self {
        Self {
            notify,
            interval,
            max_use_ratio: max_use,
        }
    }
}

impl<N: Notify> ProcfsMemoryWatch<N> {
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut interval = time::interval(self.interval);

        loop {
            interval.tick().await;

            let mem_info = match task::spawn_blocking(Meminfo::current).await.unwrap() {
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
            let used = total - mem_info.mem_available.expect("mem availabe is None");

            if (used as f64 / total as f64) > self.max_use_ratio {
                if let Err(err) = self.notify.notify_memory(total as _, used as _).await {
                    error!(%err, "notify memory failed");
                }
            }
        }
    }
}
