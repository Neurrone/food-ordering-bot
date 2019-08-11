use std::{
    collections::{HashMap, HashSet},
    fmt,
    string::String,
};
use telegram_bot::types::chat::User;

/// Represents an active order
#[derive(Clone)]
pub struct Order {
    /// The name of the order, e.g "waffles"
    pub name: String,
    /// map of the item name to the users who ordered them
    pub items: HashMap<String, HashSet<User>>,
    /// the creater of the order
    pub owner: User,
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
        if self.items.is_empty() {
            return write!(f, "Orders for {}:\n\nNone", self.name);
        }

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
