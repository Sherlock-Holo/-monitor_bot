use std::{io, time::Duration};

use clap::Parser;
use monitor::mem::ProcfsMemoryWatch;
use tracing::{level_filters::LevelFilter, subscriber};
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

use crate::{bot::Bot, notify::Notify};

mod bot;
mod monitor;
mod notify;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, action)]
    debug: bool,

    #[clap(long, value_parser = humantime::parse_duration, default_value = "3s")]
    mem_watch_interval: Duration,

    #[clap(long, default_value = "0.7")]
    mem_max_used_ratio: f64,

    #[clap(short = 'b', long)]
    bot_token: String,

    #[clap(long)]
    group_chat_id: i64,
}

pub async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    init_log(args.debug);

    let bot = Bot::new(args.bot_token, args.group_chat_id);

    run_monitor(bot, args.mem_watch_interval, args.mem_max_used_ratio).await
}

async fn run_monitor<N: Notify>(
    notify: N,
    interval: Duration,
    max_mem_used_ratio: f64,
) -> anyhow::Result<()> {
    let mem_watch = ProcfsMemoryWatch::new(notify, interval, max_mem_used_ratio);

    mem_watch.run().await
}

pub fn init_log(debug: bool) {
    let layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_writer(io::stderr);

    let level = if debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let layered = Registry::default().with(layer).with(level);

    subscriber::set_global_default(layered).unwrap();
}
