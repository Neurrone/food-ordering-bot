use std::{collections::HashMap, default::Default, string::String};
use telegram_bot::types::{
    chat::{MessageChat, User},
    InlineKeyboardMarkup,
};

use crate::conversation_orders::ConversationOrders;

/// The result of executing a bot command
pub struct CommandResult {
    /// whether the command succeeded
    pub success: bool,
    /// the response to send to the user
    pub response: String,
    /// an optional reply markup to facilitate easier ordering to attach to some responses
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl CommandResult {
    /// Helper to construct a successful CommandResult
    pub fn success(response: String) -> Self {
        Self {
            success: true,
            response,
            reply_markup: None,
        }
    }
    /// Helper to construct an unsuccessful CommandResult
    pub fn failure(response: String) -> Self {
        Self {
            success: false,
            response,
            reply_markup: None,
        }
    }
}

#[derive(Default)]
/// Food Ordering Bot implementation logic
pub struct Bot {
    active_orders: HashMap<MessageChat, ConversationOrders>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Starts an order
    pub fn start_order(
        &mut self,
        chat: MessageChat,
        creater: User,
        order_name: String,
    ) -> CommandResult {
        match self.active_orders.get_mut(&chat) {
            // there are already orders for this conversation
            Some(conversation_orders) => {
                if conversation_orders.add_order(creater, order_name.clone()) {
                    CommandResult::success(format!("Order started for {}.\nUse /order {} <item> to order, /view_orders to view active orders and /end_order {} when done.", order_name, order_name, order_name))
                } else {
                    CommandResult::failure(format!(
                        "There is already an order for {} in progress.",
                        order_name
                    ))
                }
            }
            None => {
                let mut conversation_orders = ConversationOrders {
                    orders: HashMap::new(),
                };
                conversation_orders.add_order(creater, order_name.clone());
                self.active_orders.insert(chat, conversation_orders);
                CommandResult::success(format!("Order started for {}.\nUse /order <item> to order, /view_orders to view active orders, /end_order when done, or start another order.", order_name))
            }
        }
    }

    /// Terminates an order, if any
    pub fn end_order(
        &mut self,
        chat: &MessageChat,
        user: &User,
        order_name: &str,
    ) -> CommandResult {
        match self.active_orders.get_mut(chat) {
            Some(conversation_orders) => match conversation_orders.remove_order(user, order_name) {
                Ok(completed_order) => {
                    if self.active_orders[chat].orders.is_empty() {
                        self.active_orders.remove(chat);
                    }
                    CommandResult::success(format!("{}", completed_order))
                }
                Err(msg) => CommandResult::failure(msg),
            },
            None => CommandResult::failure(
                "There are no orders in progress. To start an order, use /start_order".into(),
            ),
        }
    }

    /// Adds an item to a running order
    pub fn add_item(
        &mut self,
        chat: &MessageChat,
        user: User,
        order_name: &str,
        item: String,
    ) -> CommandResult {
        match self.active_orders.get_mut(chat) {
            Some(conversation_orders) => match conversation_orders.add_item(order_name, user, item)
            {
                Some(updated_order) => CommandResult {
                    success: true,
                    reply_markup: Some(updated_order.generate_reply_markup()),
                    response: format!(
                        "{}\nUse /order <item> to update your order and /end_order when done.\nYou can also tap on an existing item to update or cancel your order.",
                        updated_order
                    ),
                },
                None => CommandResult::failure(format!("Order {} not found.", order_name)),
            },
            None => CommandResult::failure(
                "There are no orders in progress. To start an order, use /start_order".into(),
            ),
        }
    }

    /// Cancels the currently selected item for an order
    pub fn remove_item(
        &mut self,
        chat: &MessageChat,
        user: &User,
        order_name: &str,
    ) -> CommandResult {
        match self.active_orders.get_mut(chat) {
            Some(conversation_orders) => match conversation_orders.remove_item(order_name, user) {
                Some(updated_order) => CommandResult {
                    success: true,
                    response: format!(
                        "{}\nUse /order <item> to order, and /end_order when done.\nYou can also tap on an existing item to update or cancel your order.",
                        updated_order
                    ),
                    reply_markup: Some(updated_order.generate_reply_markup()),
                },
                None => CommandResult::failure(format!(
                    "You have either not placed any orders for {}, or order {} does not exist.",
                    order_name, order_name
                )),
            },
            None => CommandResult::failure(
                "There are no orders in progress. To start an order, use /start_order".into(),
            ),
        }
    }

    /// Views all active orders for the chat
    pub fn view_orders(&mut self, chat: &MessageChat) -> CommandResult {
        match self.active_orders.get(chat) {
            Some(conversation_orders) => CommandResult {
                success: true,
                response: format!("{}\n\nUse /order <item> to order, /cancel to cancel your order and /end_order when done.\nYou can also tap on an existing item to update or cancel your order.", conversation_orders),
                reply_markup: Some(conversation_orders.generate_reply_markup()),
            },
            None => CommandResult::failure("There are no orders in progress. To start an order, use /start_order".into())
        }
    }

    pub fn handle_callback_query(
        &mut self,
        chat: &MessageChat,
        user: User,
        data: &str,
        is_message_output_of_view_orders: bool,
    ) -> (CommandResult, String) {
        let normalized_query = data.to_lowercase().trim().replace("@food_ordering_bot", "");
        if let Some(sep) = normalized_query.find(' ') {
            let order_name = &normalized_query[..sep];
            let item = &normalized_query[sep + 1..];
            // If only if let chains were properly implemented so this ugly 3-level nesting isn't needed :(
            // if the user clicked on a button that corresponds to their current order, we should cancel it
            // otherwise, the user wants to change their order
            let mut should_cancel_existing_order = false;
            if let Some(conversation_orders) = self.active_orders.get(chat) {
                if let Some(order) = conversation_orders.orders.get(order_name) {
                    if let Some(item_user_ordered) = order.find_user_item(&user) {
                        should_cancel_existing_order = item_user_ordered == item
                    }
                }
            }

            let res = if should_cancel_existing_order {
                self.remove_item(chat, &user, order_name)
            } else {
                // order this item, overriding any previous orders if needed
                self.add_item(chat, user, order_name, item.to_string())
            };

            if res.success {
                let answer = if should_cancel_existing_order {
                    format!("Cancelled order of {} for {}.", item, order_name)
                } else {
                    format!("Updated order for {} to {}.", order_name, item)
                };
                if is_message_output_of_view_orders {
                    // the response in res only contains info about the current order being edited
                    // since the message associated with the callback query contains all orders,
                    // we need to retrieve info about all orders to correctly edit it
                    (self.view_orders(chat), answer)
                } else {
                    (res, answer)
                }
            } else {
                let answer = res.response.clone();
                (res, answer)
            }
        } else {
            (
                CommandResult::failure("Unrecognized callback query".into()),
                "Invalid order or item name".to_string(),
            )
        }
    }

    pub fn get_active_order_names(&self, chat: &MessageChat) -> Vec<&str> {
        match self.active_orders.get(chat) {
            Some(active_orders) => active_orders.orders.keys().map(|k| k.as_ref()).collect(),
            None => vec![],
        }
    }

    pub fn has_active_orders(&self) -> bool {
        !self.active_orders.is_empty()
    }
}
