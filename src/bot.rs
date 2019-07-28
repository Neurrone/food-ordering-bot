use std::{
    collections::{HashMap, HashSet},
    default::Default,
    fmt,
    string::String,
};
use telegram_bot::types::chat::{MessageChat, User};

/// The result of executing a bot command
pub struct CommandResult {
    /// whether the command succeeded
    pub success: bool,
    /// the response to send to the user
    pub response: String,
}

impl CommandResult {
    /// Helper to construct a successful CommandResult
    pub fn success(response: String) -> Self {
        Self {
            success: true,
            response,
        }
    }
    /// Helper to construct an unsuccessful CommandResult
    pub fn failure(response: String) -> Self {
        Self {
            success: false,
            response,
        }
    }
}

/// Represents an active order
#[derive(Clone)]
pub struct Order {
    /// The name of the order, e.g "waffles"
    name: String,
    /// map of the item name to the users who ordered them
    items: HashMap<String, HashSet<User>>,
    /// the creater of the order
    owner: User,
}

impl Order {
    /// Adds an item to the current order
    /// Returns whether the addition overrides the user's previous order
    pub fn add_item(&mut self, user: User, item: String) -> bool {
        // Remove any existing items this user has ordered
        let overrides_existing_order = self.remove_item(&user).is_some();
        match self.items.get_mut(&item) {
            Some(users) => {
                users.insert(user);
                overrides_existing_order
            }
            None => {
                let mut users = HashSet::new();
                users.insert(user);
                self.items.insert(item, users);
                overrides_existing_order
            }
        }
    }

    /// Removes a user's order, returning the item that was removed, if any
    pub fn remove_item(&mut self, user: &User) -> Option<String> {
        let mut existing_item: Option<String> = None;
        for (item, users) in self.items.iter_mut() {
            if users.remove(user) {
                existing_item = Some(item.clone());
                break;
            }
        }
        match existing_item {
            Some(item) => {
                if self.items[&item].is_empty() {
                    self.items.remove(&item);
                }
                Some(item)
            }
            None => None,
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_orders: Vec<String> = self
            .items
            .iter()
            .map(|(item, users)| {
                let mut sorted_users: Vec<String> =
                    users.iter().map(|user| user.first_name.clone()).collect();
                sorted_users.sort();
                format!("{} {}: {}", users.len(), item, sorted_users.join(", "))
            })
            .collect();
        sorted_orders.sort();
        write!(
            f,
            "Orders for {}:\n\n{}",
            self.name,
            sorted_orders.join("\n")
        )
    }
}

/// Active orders for a conversation
pub struct ConversationOrders {
    /// active orders for this conversation
    pub orders: HashMap<String, Order>,
}

impl ConversationOrders {
    /// Adds an order for this conversation, returning whether the addition was successful
    pub fn add_order(&mut self, creater: User, order_name: String) -> bool {
        if let Some(_) = self.orders.get(&order_name) {
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
            Some(conversation_orders) => {
                // there are already orders for this conversation
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
            Some(conversation_orders) => {
                match conversation_orders.add_item(order_name, user, item) {
                    Some(updated_order) => CommandResult::success(format!(
                        "{}\nUse /order <item> to update your order and /end_order when done.",
                        updated_order
                    )),
                    None => CommandResult::failure(format!("Order {} not found.", order_name)),
                }
            }
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
                Some(updated_order) => CommandResult::success(format!(
                    "{}\nUse /order <item> to order, and /end_order when done.",
                    updated_order
                )),
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
            Some(conversation_orders) => CommandResult::success(format!("{}\n\nUse /order <item> to order, /cancel to cancel your order and /end_order when done.", conversation_orders)),
            None => CommandResult::failure("There are no orders in progress. To start an order, use /start_order".into())
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
