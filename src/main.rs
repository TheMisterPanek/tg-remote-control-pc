use std::env;
use std::fs;
use std::sync::Arc;

use remote_core::bot::callback::dispatch_callback;
use remote_core::bot::command::BotCommand;
use remote_core::bot::dispatch::{dispatch, DispatchResult};
use remote_core::bot::ui::InlineKeyboard;
use remote_core::config::Config;
use remote_core::media::controller::MediaController;
use remote_os::playerctl::PlayerctlController;

use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage,
};

type Err = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "./config.toml".to_string());
    let raw = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!("cannot read {config_path}: {e}");
        std::process::exit(1);
    });
    let config = remote_core::config::Config::from_toml(&raw).unwrap_or_else(|e| {
        eprintln!("invalid config: {e}");
        std::process::exit(1);
    });

    let bot = Bot::new(config.token.clone());
    let config: Arc<Config> = Arc::new(config);
    let controller: Arc<dyn MediaController> = Arc::new(PlayerctlController::new());

    println!("tg-media-remote: bot started, polling for updates…");

    Dispatcher::builder(bot, schema())
        .dependencies(teloxide::dptree::deps![config, controller])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Err> {
    use teloxide::dptree::entry;
    entry()
        .branch(Update::filter_message().endpoint(on_message))
        .branch(Update::filter_callback_query().endpoint(on_callback))
}

fn to_teloxide_keyboard(kb: &InlineKeyboard) -> InlineKeyboardMarkup {
    let row: Vec<InlineKeyboardButton> = kb
        .buttons
        .iter()
        .map(|b| InlineKeyboardButton::callback(b.label.clone(), b.callback_data.clone()))
        .collect();
    InlineKeyboardMarkup::new(vec![row])
}

async fn on_message(
    bot: Bot,
    msg: Message,
    config: Arc<Config>,
    controller: Arc<dyn MediaController>,
) -> Result<(), Err> {
    let Some(text) = msg.text() else { return Ok(()) };
    let Some(rest) = text.strip_prefix('/') else { return Ok(()) };
    let cmd = rest.split_whitespace().next().unwrap_or("");
    let cmd = cmd.split('@').next().unwrap_or("");
    let Some(bot_cmd) = BotCommand::parse(cmd) else { return Ok(()) };

    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    match dispatch(user_id, &bot_cmd, &config, controller.as_ref()) {
        DispatchResult::Reply(reply) => {
            bot.send_message(msg.chat.id, reply).await?;
        }
        DispatchResult::PlayerReply(pr) => {
            bot.send_message(msg.chat.id, pr.text)
                .reply_markup(to_teloxide_keyboard(&pr.keyboard))
                .await?;
        }
        DispatchResult::Ignored => {}
    }
    Ok(())
}

async fn on_callback(
    bot: Bot,
    q: CallbackQuery,
    config: Arc<Config>,
    controller: Arc<dyn MediaController>,
) -> Result<(), Err> {
    bot.answer_callback_query(&q.id).await?;

    let Some(data) = &q.data else { return Ok(()) };
    let user_id = q.from.id.0 as i64;

    let result = dispatch_callback(user_id, data, &config, controller.as_ref());

    let Some(MaybeInaccessibleMessage::Regular(msg)) = &q.message else {
        return Ok(());
    };

    match result {
        DispatchResult::PlayerReply(pr) => {
            bot.edit_message_text(msg.chat.id, msg.id, pr.text)
                .reply_markup(to_teloxide_keyboard(&pr.keyboard))
                .await?;
        }
        DispatchResult::Reply(text) => {
            bot.send_message(msg.chat.id, text).await?;
        }
        DispatchResult::Ignored => {}
    }
    Ok(())
}
