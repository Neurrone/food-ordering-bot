#![warn(clippy::all)]
extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

mod bot;
mod command;
mod conversation_orders;
mod order;
use bot::CommandResult;
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
    let future = api.stream()
    .inspect_err(|err| eprintln!("{:?}", err))
    // map Err to Ok variant
    // this is terrible, but I'm resorting to this to prevent errors from crashing the bot
    // TODO: figure out if there's a better way of preventing errors from panicking
    .then(|r| {
        match r {
            Ok(_) => r,
            // it doesn't matter if we use an id of 1, since nothing uses it. This is terrible though :(
            Err(e) => Ok(Update { id: 1, kind: UpdateKind::Error(e.description().to_string())})
        }
    })
    .for_each(|update| {
        // If the received update contains a new message...
        match update.kind {
            UpdateKind::Message(message) => {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let had_active_orders_before = bot.has_active_orders();
                    let res = match command::parse_command(
                        data,
                        &bot.get_active_order_names(&message.chat),
                    ) {
                        Ok(Start) => CommandResult::success("Use /start_order <order name> to start a new order, or /help for a full list of commands.".into()),
                        Ok(Help) => CommandResult::success("/start_order <order name> - starts an order. For example, /start_order waffles.
    /view_orders - shows active orders.

    The following commands will ask for the order name, if there are multiple active orders.

    /order [order name] <item> - adds an item to an order, or replaces the previously chosen one.
    /cancel [order-name] - removes your previously selected item from an order.
    /end_order [order-name] - stops an order.

    For feature requests, bug reports and source: https://github.com/Neurrone/food-ordering-bot".to_string()),
                        Ok(StartOrder(order_name)) => {
                            bot.start_order(message.chat.clone(), message.from.clone(), order_name)
                        }
                        Ok(EndOrder(order_name)) => {
                            bot.end_order(&message.chat, &message.from, &order_name)
                        }
                        Ok(AddItem(order_name, item_name)) => {
                            bot.add_item(
                                &message.chat,
                                message.from.clone(),
                                &order_name,
                                item_name,
                            )
                        }
                        Ok(RemoveItem(order_name)) => {
                            bot.remove_item(&message.chat, &message.from, &order_name)
                        }
                        Ok(ViewOrders) => bot.view_orders(&message.chat),
                        Err(error_message) => CommandResult::failure(error_message),
                    };
                    match res.reply_markup {
                        Some(markup) => api.spawn(
                            message.text_reply(res.response).reply_markup(markup)),
                        None => api.spawn(message.text_reply(res.response))
                    }
                    let had_active_orders_now = bot.has_active_orders();
                    if had_active_orders_before != had_active_orders_now {
                        let status = if had_active_orders_now {
                            "There are now active orders."
                        } else {
                            "No active orders."
                        };
                        println!("{}", status);
                    }
                }
            },
            UpdateKind::CallbackQuery(query) => {
                let is_original_command_output_of_view_orders = match query.message.clone().reply_to_message {
                    Some(m) => if let MessageOrChannelPost::Message(message) = *m {
                        if let MessageKind::Text { ref data, .. } = message.kind {
                            data.to_lowercase().trim() == "/view_orders"
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                    None => false
                };
                let (res, answer) = bot.handle_callback_query(&query.message.chat, query.from.clone(), &query.data, is_original_command_output_of_view_orders);
                api.spawn(query.answer(answer));
                if res.success {
                    api.spawn(query.message.edit_text(res.response));
                    api.spawn(query.message.edit_reply_markup(res.reply_markup));
                }
            },
            _ => ()
        }
        Ok(())
    });

    // this should never panick, since we prevent the stream's future from returning an error
    core.run(future).unwrap();
}
