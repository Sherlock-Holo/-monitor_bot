#[trait_variant::make(Notify: Send)]
pub trait LocalNotify {
    async fn notify_memory(&self, mem_total: usize, mem_used: usize) -> anyhow::Result<()>;

    async fn notify_self_error(&self, err: impl AsRef<str> + Send) -> anyhow::Result<()>;
}
