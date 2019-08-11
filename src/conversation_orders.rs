use std::{collections::HashMap, fmt, string::String};
use telegram_bot::{
    types::{chat::User, InlineKeyboardMarkup},
    InlineKeyboardButton,
};

use crate::order::Order;

/// Active orders for a conversation
pub struct ConversationOrders {
    /// active orders for this conversation
    pub orders: HashMap<String, Order>,
}

impl ConversationOrders {
    /// Adds an order for this conversation, returning whether the addition was successful
    pub fn add_order(&mut self, creater: User, order_name: String) -> bool {
        if self.orders.get(&order_name).is_some() {
            false // the order already exists
        } else {
            self.orders.insert(
                order_name.clone(),
                Order {
                    name: order_name,
                    items: HashMap::new(),
                    owner: creater,
                },
            );
            true
        }
    }

    /// Removes or ends an order for this conversation, returning the removed order on success
    /// Only the creater of the order may remove it
    pub fn remove_order(&mut self, user: &User, order_name: &str) -> Result<Order, String> {
        match self.orders.get(order_name) {
            Some(order) => {
                if order.owner.id == user.id {
                    Ok(self.orders.remove(order_name).unwrap())
                } else {
                    Err(format!(
                        "Only {} may end their order for {}.",
                        order.owner.first_name, order_name
                    ))
                }
            }
            None => Err(format!("Order {} not found.", order_name)),
        }
    }

    /// Adds an item to the specified order, returning the Order that was just updated
    pub fn add_item(&mut self, order_name: &str, user: User, item: String) -> Option<Order> {
        match self.orders.get_mut(order_name) {
            Some(order) => {
                let _overrode_previous_order = order.add_item(user, item.clone());
                Some(order.clone())
            }
            None => None, // the order we're trying to add an item to does not exist
        }
    }

    /// Removes a user's item from the order, returning the item that was just removed
    pub fn remove_item(&mut self, order_name: &str, user: &User) -> Option<Order> {
        match self.orders.get_mut(order_name) {
            Some(order) => {
                if let Some(_item_removed) = order.remove_item(user) {
                    Some(self.orders[order_name].clone())
                } else {
                    None // the user did not order this
                }
            }
            None => None, // the order we're trying to remove this user's item from doesn't exist
        }
    }

    /// Returns inline keyboard buttons which users can click to order an existing item
    pub fn generate_reply_markup(&self) -> InlineKeyboardMarkup {
        let buttons: Vec<InlineKeyboardButton> = self
            .orders
            .values()
            .map(|order| order.generate_inline_buttons())
            .flatten()
            .collect();
        let mut keyboard_markup = InlineKeyboardMarkup::new();
        for row in buttons.chunks(2) {
            keyboard_markup.add_row(row.to_vec());
        }
        keyboard_markup
    }
}

impl fmt::Display for ConversationOrders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.orders.is_empty() {
            write!(f, "There are no active orders.")
        } else {
            let orders_to_display: Vec<String> = self
                .orders
                .values()
                .map(|order| format!("{}", order))
                .collect();
            let header = if orders_to_display.len() > 1 {
                format!("There are {} orders.\n", orders_to_display.len())
            } else {
                "".to_string()
            };
            write!(f, "{}{}", header, orders_to_display.join("\n\n"))
        }
    }
}
