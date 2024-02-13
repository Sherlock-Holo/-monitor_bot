#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    monitor_bot::run().await
}
