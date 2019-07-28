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
                let res = match command::parse_command(
                    data,
                    &bot.get_active_order_names(message.chat.clone()),
                ) {
                    Ok(Start) => "Use /start_order <order name> to start a new order, or /help for a full list of commands.".to_string(),
                    Ok(Help) => "/start_order <order name> - starts an order. For example, /start_order waffles.
/view_orders - shows active orders.

The following commands will ask you to specify the order name, if multiple orders are active simultaneously.

/order [order name] <item> - adds an item to an order, or replaces the previously chosen one.
/cancel [order-name] - removes your previously selected item from an order.
/end_order [order-name] - stops an order.".to_string(),
                    Ok(StartOrder(order_name)) => {
                        bot.start_order(message.chat.clone(), order_name).response
                    }
                    Ok(EndOrder(order_name)) => {
                        bot.end_order(message.chat.clone(), &order_name).response
                    }
                    Ok(AddItem(order_name, item_name)) => {
                        bot.add_item(
                            message.chat.clone(),
                            message.from.clone(),
                            &order_name,
                            item_name,
                        )
                        .response
                    }
                    Ok(RemoveItem(order_name)) => {
                        bot.remove_item(message.chat.clone(), message.from.clone(), &order_name)
                            .response
                    }
                    Ok(ViewOrders) => bot.view_orders(message.chat.clone()).response,
                    Err(error_message) => error_message,
                };
                api.spawn(message.text_reply(res));
            }
        }
        Ok(())
    });

    core.run(future).unwrap();
}
