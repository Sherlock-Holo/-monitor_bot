use byte_unit::{Byte, UnitType};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me, ParseMode};
use teloxide::utils::command::BotCommands as _;
use tracing::{debug, error, instrument};

use crate::monitor::mem::MemoryInfo;
use crate::notify::Notify;

const MEMORY_CALLBACK_DATA: &str = "memory";

#[derive(Debug, Clone)]
pub struct Bot {
    bot: teloxide::Bot,
    group_chat_id: ChatId,
}

impl Bot {
    pub fn new(token: String, group_chat_id: i64) -> Self {
        let bot = teloxide::Bot::new(token);

        Self {
            bot,
            group_chat_id: ChatId(group_chat_id),
        }
    }

    fn create_inline_buttons() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default().append_row([InlineKeyboardButton::callback(
            "memory usage",
            MEMORY_CALLBACK_DATA,
        )])
    }

    pub async fn run_active<M: MemoryInfo + Sync + Clone + 'static>(
        &self,
        memory_info: M,
    ) -> anyhow::Result<()> {
        let handler = dptree::entry()
            .branch(Update::filter_message().endpoint({
                let bot = self.clone();

                move |_: teloxide::Bot, msg: Message, me: Me| {
                    let bot = bot.clone();

                    async move { bot.handle_command(msg, me).await }
                }
            }))
            .branch(Update::filter_callback_query().endpoint({
                let bot = self.clone();

                move |_: teloxide::Bot, callback_query: CallbackQuery| {
                    let bot = bot.clone();
                    let memory_info = memory_info.clone();

                    async move { bot.handle_callback(callback_query, memory_info).await }
                }
            }));

        Dispatcher::builder(self.bot.clone(), handler)
            .build()
            .dispatch()
            .await;

        Err(anyhow::anyhow!("bot is stopped"))
    }

    #[instrument(level = "debug", err(Debug))]
    async fn handle_command(&self, msg: Message, me: Me) -> anyhow::Result<()> {
        if msg.chat.id != self.group_chat_id {
            error!("ignore unknown chat message");

            return Ok(());
        }

        if let Some(text) = msg.text() {
            if let Ok(command) = Command::parse(text, me.username()) {
                match command {
                    Command::Show => {
                        self.bot
                            .send_message(msg.chat.id, "<strong>choose type</strong>")
                            .parse_mode(ParseMode::Html)
                            .reply_to_message_id(msg.id)
                            .reply_markup(Self::create_inline_buttons())
                            .await?;

                        debug!("send button done");
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(level = "debug", skip(memory_info), err(Debug))]
    async fn handle_callback<M: MemoryInfo>(
        &self,
        callback_query: CallbackQuery,
        memory_info: M,
    ) -> anyhow::Result<()> {
        if let Some(chat_id) = callback_query.chat_id() {
            if chat_id != self.group_chat_id {
                error!("ignore unknown chat message");

                return Ok(());
            }

            if let Some(data) = callback_query.data {
                match data.as_str() {
                    MEMORY_CALLBACK_DATA => {
                        self.bot.answer_callback_query(callback_query.id).await?;

                        let total = memory_info.get_memory_total().await?;
                        let available = memory_info.get_memory_available().await?;
                        let message = Self::get_memory_info_message(total, total - available);

                        self.bot
                            .send_message(chat_id, message)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }

                    _ => {
                        debug!("ignore unknown callback data");
                    }
                }
            }
        }

        Ok(())
    }

    fn get_memory_info_message(mem_total: usize, mem_used: usize) -> String {
        let mem_total = Byte::from(mem_total);
        let mem_used = Byte::from(mem_used);

        let message = format!(
            r#"<strong>memory total: </strong>{:.2}
<strong>memory usage: </strong>{:.2}"#,
            mem_total.get_appropriate_unit(UnitType::Binary),
            mem_used.get_appropriate_unit(UnitType::Binary)
        );
        message
    }
}

impl Notify for Bot {
    #[instrument(level = "debug", err(Debug))]
    async fn notify_memory(&self, mem_total: usize, mem_used: usize) -> anyhow::Result<()> {
        let message = Self::get_memory_info_message(mem_total, mem_used);

        self.bot
            .send_message(self.group_chat_id, message)
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
                self.group_chat_id,
                format!("system monitor self error: {err}"),
            )
            .await?;

        debug!(err, "notify self error done");

        Ok(())
    }
}

#[derive(Debug, BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Show,
}
