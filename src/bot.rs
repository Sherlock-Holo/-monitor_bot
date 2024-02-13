use byte_unit::{Byte, UnitType};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
};
use tracing::{debug, instrument};

use crate::notify::Notify;

#[derive(Debug, Clone)]
pub struct Bot {
    bot: teloxide::Bot,
    group_chat_id: i64,
}

impl Bot {
    pub fn new(token: String, group_chat_id: i64) -> Self {
        let bot = teloxide::Bot::new(token);

        Self { bot, group_chat_id }
    }
}

impl Notify for Bot {
    #[instrument(level = "debug", err(Debug))]
    async fn notify_memory(&self, mem_total: usize, mem_used: usize) -> anyhow::Result<()> {
        let mem_total = Byte::from(mem_total);
        let mem_used = Byte::from(mem_used);

        let message = format!(
            r#"<strong>memory total: </strong>{:.2}
<strong>memory usage: </strong>{:.2}"#,
            mem_total.get_appropriate_unit(UnitType::Binary),
            mem_used.get_appropriate_unit(UnitType::Binary)
        );

        self.bot
            .send_message(ChatId(self.group_chat_id), message)
            .parse_mode(ParseMode::Html)
            .await?;

        debug!("notify memory done");

        Ok(())
    }

    #[instrument(level = "debug", skip(err), err(Debug))]
    async fn notify_self_error(&self, err: impl AsRef<str> + Send) -> anyhow::Result<()> {
        let err = err.as_ref();
        self.bot
            .send_message(
                ChatId(self.group_chat_id),
                format!("system monitor self error: {err}"),
            )
            .await?;

        debug!(err, "notify self error done");

        Ok(())
    }
}
