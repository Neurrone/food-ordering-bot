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

#[derive(Default)]
/// Food Ordering Bot implementation logic
pub struct Bot {
    active_orders: HashMap<MessageChat, Order>,
}

/// Represents an active order
/// Each conversation may only have one active order at a time.
pub struct Order {
    /// The name of the order, e.g "waffles"
    name: String,
    /// map of the item name to the users who ordered them
    items: HashMap<String, HashSet<User>>,
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

impl Bot {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Starts an order
    /// Only one order may be active at a time per conversation
    pub fn start_order(&mut self, chat: MessageChat, order_name: String) -> CommandResult {
        match self.active_orders.get(&chat) {
            Some(existing_order) => CommandResult::failure(format!("There is already an order for {} in progress. Use /order <item> to add an item to the existing order.", existing_order.name)),
            None => {
                self.active_orders.insert(chat, Order { name: order_name.clone(), items: HashMap::new()});
                CommandResult::success(format!("Order started for {}.\nUse /order <item> to order, /view_order to view the current order and /end_order when done.", order_name))
            }
        }
    }

    /// Terminates an order, if any
    pub fn end_order(&mut self, chat: MessageChat) -> CommandResult {
        match self.active_orders.remove(&chat) {
            Some(completed_order) => CommandResult::success(format!("{}", completed_order)),
            None => CommandResult::failure(
                "There are no orders in progress. To start an order, use /start_order".into(),
            ),
        }
    }

    /// Adds an item to a running order
    pub fn add_item(&mut self, chat: MessageChat, user: User, item: String) -> CommandResult {
        match self.active_orders.get_mut(&chat) {
            Some(active_order) => {
                let _overrode_previous_order = active_order.add_item(user, item.clone());
                CommandResult::success(format!("{}\nUse /order <item> to update your order and /end_order when done.", active_order))
            }
            None => CommandResult::failure(
                "There are no orders in progress. To start an order, use /start_order".into(),
            ),
        }
    }

    /// Cancels the currently selected item for the current order
    pub fn remove_item(&mut self, chat: MessageChat, user: User) -> CommandResult {
        match self.active_orders.get_mut(&chat) {
            Some(active_order) => {
                match active_order.remove_item(&user) {
                    Some(item_removed) => CommandResult::success(format!("Cancelled existing order for {}.\nUse /order <item> to order, /view_order to view the current order and /end_order when done.", item_removed)),
                    None => CommandResult::failure("You have not placed any orders. Use /order <item> to do so.".into()),
                }
            },
            None => CommandResult::failure("There are no orders in progress. To start an order, use /start_order".into())
        }
    }

    /// Views the current order
    pub fn view_order(&mut self, chat: MessageChat) -> CommandResult {
        match self.active_orders.get(&chat) {
            Some(order) => CommandResult::success(format!("{}\n\nUse /order <item> to order, /cancel to cancel your order and /end_order when done.", order)),
            None => CommandResult::failure("There are no orders in progress. To start an order, use /start_order".into())
        }
    }
}
