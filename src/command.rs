#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    /// starts a new order for this conversation
    StartOrder(String),
    /// ends an order
    EndOrder(String),
    /// adds an item to the currently active order
    AddItem(String, String),
    /// Cancels the currently selected item
    RemoveItem(String),
    /// view the current order
    ViewOrders,
}

type ParseResult = std::result::Result<Command, String>;

pub fn parse_command(message: &str, active_orders: Vec<String>) -> ParseResult {
    use Command::*;
    if !message.starts_with("/") {
        return Err("Invalid command".to_string());
    }

    let normalized_message = message.to_lowercase().replace("@food_ordering_bot", "");
    let tokens: Vec<&str> = normalized_message.split_whitespace().collect();
    let command = tokens[0];
    let args = &tokens[1..];
    match command {
        "/start_order" => {
            if args.len() == 1 {
                Ok(StartOrder(args[0].to_string()))
            } else if args.len() == 0 {
                Err("Specify the name of the order. For example, /start_order waffles".into())
            } else {
                Err(format!(
                    "Order names must not contain spaces. Try /start_order {}",
                    args.join("-")
                ))
            }
        }
        "/end_order" => {
            if active_orders.len() == 0 {
                Err(
                    "There are no active orders. Start one by using /start_order <order name>"
                        .into(),
                )
            } else if let Some(order_name) = infer_order_name(args, &active_orders) {
                Ok(EndOrder(order_name))
            } else if args.len() == 0 {
                Err("Since there are multiple active orders, Specify the name of the order. For example, /end_order waffles".into())
            } else {
                Err(format!("Order {} not found.", args[0]))
            }
        }
        "/order" => {
            if active_orders.len() == 0 {
                Err(
                    "There are no active orders. Start one by using /start_order <order name>"
                        .into(),
                )
            } else if active_orders.len() == 1 {
                if args.len() == 0 {
                    Err("Specify the name of the item you wish to order. For example, /order chocolate".into())
                } else if active_orders.contains(&args[0].to_string()) {
                    let order_name = args[0];
                    let item = args[1..].join(" ");
                    Ok(AddItem(order_name.to_string(), item))
                } else {
                    Ok(AddItem(active_orders[0].clone(), args.join(" ")))
                }
            } else {
                // multiple active orders
                if args.len() < 2 {
                    Err("Specify the order name and item you wish to order. For example, /order waffles chocolate".into())
                } else if active_orders.contains(&args[0].to_string()) {
                    let order_name = args[0];
                    let item = args[1..].join(" ");
                    Ok(AddItem(order_name.to_string(), item))
                } else {
                    Err(format!("Order {} not found. Specify the order name and item you wish to order. For example, /order waffles chocolate", args[0]))
                }
            }
        }
        "/cancel" => {
            if active_orders.len() == 0 {
                Err(
                    "There are no active orders. Start one by using /start_order <order name>"
                        .into(),
                )
            } else if let Some(order_name) = infer_order_name(args, &active_orders) {
                Ok(RemoveItem(order_name))
            } else if args.len() == 0 {
                Err("Since there are multiple active orders, Specify the name of the order. For example, /cancel waffles".into())
            } else {
                Err(format!("Order {} not found.", args[0]))
            }
        }
        "/view_orders" => Ok(ViewOrders),
        _ => Err(format!("Invalid command")),
    }
}

fn infer_order_name(args: &[&str], active_orders: &Vec<String>) -> Option<String> {
    if args.len() == 0 && active_orders.len() == 1 {
        Some(active_orders[0].clone()) // order name not specified, but can be infered
    } else if args.len() == 1 && active_orders.contains(&args[0].to_string()) {
        Some(args[0].to_string()) // the specified order to end exists
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;

    #[test]
    fn parse_unrecognized_command_errors() {
        assert!(parse_command("/invalid_command", vec![]).is_err());
    }

    #[test]
    fn parse_start_order() {
        assert_eq!(
            parse_command("/start_order ", vec![]),
            Err("Specify the name of the order. For example, /start_order waffles".into())
        );
        assert_eq!(
            parse_command("/start_order waffles", vec![]),
            Ok(StartOrder("waffles".into()))
        );
        assert_eq!(
            parse_command("/Start_order WAFFLES ", vec![]),
            parse_command("/start_order waffles", vec![]),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/start_order waffles @food_ordering_bot", vec![]),
            parse_command("/start_order waffles", vec![]),
            "@mentions are ignored"
        );
        assert_eq!(
            parse_command("/start_order mr bean", vec![]),
            Err("Order names must not contain spaces. Try /start_order mr-bean".into())
        );
        assert_eq!(
            parse_command("/start_order mr-bean", vec![]),
            Ok(StartOrder("mr-bean".into()))
        );
    }

    #[test]
    fn parse_end_order() {
        assert_eq!(
            parse_command("/end_order", vec![]),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(parse_command("/end_order", vec!["waffles".to_string(), "pizza".to_string()]), Err("Since there are multiple active orders, Specify the name of the order. For example, /end_order waffles".into()));
        assert_eq!(
            parse_command("/end_order waffles", vec!["waffles".to_string()]),
            Ok(EndOrder("waffles".to_string()))
        );
        assert_eq!(
            parse_command("/end_order", vec!["waffles".to_string()]),
            Ok(EndOrder("waffles".to_string()))
        );
        assert_eq!(
            parse_command(
                "/end_order waffles",
                vec!["pizza".to_string(), "waffles".to_string()]
            ),
            Ok(EndOrder("waffles".to_string()))
        );
        assert_eq!(
            parse_command("/End_order Waffles ", vec!["waffles".to_string()]),
            parse_command("/end_order waffles", vec!["waffles".to_string()]),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/end_order Waffles", vec![]),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(
            parse_command("/end_order Waffles", vec!["pizza".to_string()]),
            Err("Order waffles not found.".into())
        );
    }

    /*
    #[test]
    fn parse_order() {
        assert_eq!(
            parse_command("/order "),
            Err(
                "Specify the name of the item you wish to order. For example, /order chocolate"
                    .into()
            )
        );
        assert_eq!(
            parse_command("/order chocolate"),
            Ok(AddItem("chocolate".into()))
        );
        assert_eq!(
            parse_command("/order Chocolate Pancake"),
            Ok(AddItem("chocolate pancake".into())),
            "capitalization is ignored, and multi-word items are allowed"
        );
    }
    */
}
