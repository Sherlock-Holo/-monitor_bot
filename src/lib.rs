use std::{io, time::Duration};

use clap::Parser;
use monitor::mem::ProcfsMemoryWatch;
use tracing::{level_filters::LevelFilter, subscriber};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

use crate::bot::Bot;

mod bot;
mod monitor;
mod notify;

#[derive(Debug, Parser)]
struct Args {
    /// enable debug log
    #[clap(short, long, action)]
    debug: bool,

    /// set memory watch interval
    #[clap(long, value_parser = humantime::parse_duration, default_value = "3s")]
    mem_watch_interval: Duration,

    /// set memory max usage ratio
    #[clap(long, default_value = "0.7")]
    mem_max_usage_ratio: f64,

    /// telegram bot token
    #[clap(short = 'b', long)]
    bot_token: String,

    /// telegram group chat id
    #[clap(long)]
    group_chat_id: i64,
}

pub async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    init_log(args.debug);

    let bot = Bot::new(args.bot_token, args.group_chat_id);

    let (mem_watch, memory_info) = ProcfsMemoryWatch::new(
        bot.clone(),
        args.mem_watch_interval,
        args.mem_max_usage_ratio,
    );

    let watch_task = tokio::spawn(async move { mem_watch.run().await });

    let active_bot_task = tokio::spawn(async move { bot.run_active(memory_info).await });

    let (res1, res2) = futures_util::try_join!(watch_task, active_bot_task)?;
    res1?;
    res2?;

    Err(anyhow::anyhow!("monitor bot stop unexpectedly"))
}

pub fn init_log(debug: bool) {
    let layer = fmt::layer()
        .with_line_number(true)
        .with_target(true)
        .with_writer(io::stderr);

    let level = if debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let targets = Targets::new()
        .with_target("h2", LevelFilter::OFF)
        .with_default(LevelFilter::DEBUG);

    let layered = Registry::default().with(targets).with(layer).with(level);

    subscriber::set_global_default(layered).unwrap();
}
