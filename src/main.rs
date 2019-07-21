extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

mod bot;
mod command;
use command::Command::*;

use std::env;

use futures::Stream;
use telegram_bot::*;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::configure(token).build(core.handle()).unwrap();

    let mut bot = bot::Bot::new();

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {
        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                let res = match command::parse_command(data) {
                    Ok(StartOrder(order_name)) => {
                        bot.start_order(message.chat.clone(), order_name).response
                    }
                    Ok(EndOrder) => bot.end_order(message.chat.clone()).response,
                    Ok(AddItem(item_name)) => {
                        bot.add_item(message.chat.clone(), message.from.clone(), item_name)
                            .response
                    }
                    Ok(RemoveItem) => {
                        bot.remove_item(message.chat.clone(), message.from.clone())
                            .response
                    }
                    Ok(ViewOrder) => bot.view_order(message.chat.clone()).response,
                    Err(error_message) => error_message,
                };
                api.spawn(message.text_reply(res));
            }
        }
        Ok(())
    });

    core.run(future).unwrap();
}
