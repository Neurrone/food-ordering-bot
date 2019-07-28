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
    Help,
    Start,
}

type ParseResult = std::result::Result<Command, String>;

pub fn parse_command(message: &str, active_orders: &[&str]) -> ParseResult {
    use Command::*;
    if !message.starts_with("/") {
        return Err("Use /help for supported commands.".to_string());
    }

    let normalized_message = message.to_lowercase().replace("@food_ordering_bot", "");
    let tokens: Vec<&str> = normalized_message.split_whitespace().collect();
    let command = tokens[0];
    let args = &tokens[1..];
    match command {
        "/help" => Ok(Help),
        "/start" => Ok(Start),
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
                } else if active_orders.contains(&args[0]) {
                    let order_name = args[0];
                    let item = args[1..].join(" ");
                    Ok(AddItem(order_name.to_string(), item))
                } else {
                    Ok(AddItem(active_orders[0].to_string(), args.join(" ")))
                }
            } else {
                // multiple active orders
                if args.len() < 2 {
                    Err("Specify the order name and item you wish to order. For example, /order waffles chocolate".into())
                } else if active_orders.contains(&args[0]) {
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
                Err("As there are multiple active orders, Specify the name of the order. For example, /cancel waffles".into())
            } else {
                Err(format!("Order {} not found.", args[0]))
            }
        }
        "/view_orders" => Ok(ViewOrders),
        _ => Err(format!("Use /help for a list of recognized commands.")),
    }
}

fn infer_order_name(args: &[&str], active_orders: &[&str]) -> Option<String> {
    if args.len() == 0 && active_orders.len() == 1 {
        Some(active_orders[0].to_string()) // order name not specified, but can be infered
    } else if args.len() == 1 && active_orders.contains(&args[0]) {
        Some(args[0].to_string()) // the specified order to end exists
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;

    static NO_ORDERS: &[&str] = &[];
    static WAFFLES: &[&str] = &["waffles"];
    static PIZZA: &[&str] = &["pizza"];
    static WAFFLES_AND_PIZZA: &[&str] = &["waffles", "pizza"];

    #[test]
    fn parse_unrecognized_command_errors() {
        assert!(parse_command("/invalid_command", NO_ORDERS).is_err());
        assert!(parse_command("hi", NO_ORDERS).is_err());
        assert!(parse_command("hi", WAFFLES).is_err());
    }

    #[test]
    fn parse_start_order() {
        assert_eq!(
            parse_command("/start_order ", NO_ORDERS),
            Err("Specify the name of the order. For example, /start_order waffles".into())
        );
        assert_eq!(
            parse_command("/start_order waffles", NO_ORDERS),
            Ok(StartOrder("waffles".into()))
        );
        assert_eq!(
            parse_command("/Start_order WAFFLES ", NO_ORDERS),
            parse_command("/start_order waffles", NO_ORDERS),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/start_order waffles @food_ordering_bot", NO_ORDERS),
            parse_command("/start_order waffles", NO_ORDERS),
            "@mentions are ignored"
        );
        assert_eq!(
            parse_command("/start_order ice cream", NO_ORDERS),
            Err("Order names must not contain spaces. Try /start_order ice-cream".into())
        );
        assert_eq!(
            parse_command("/start_order ice-cream", NO_ORDERS),
            Ok(StartOrder("ice-cream".into())),
            "order names may contain -"
        );
    }

    #[test]
    fn parse_end_order() {
        assert_eq!(
            parse_command("/end_order", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );

        assert_eq!(
            parse_command("/end_order waffles", WAFFLES),
            Ok(EndOrder("waffles".into()))
        );
        assert_eq!(
            parse_command("/end_order", WAFFLES),
            Ok(EndOrder("waffles".into())),
            "order name may be omitted if there is only 1 active order"
        );
        assert_eq!(
            parse_command("/end_order ice-cream", WAFFLES),
            Err("Order ice-cream not found.".into())
        );

        // multiple active orders
        assert_eq!(parse_command("/end_order", WAFFLES_AND_PIZZA), Err("Since there are multiple active orders, Specify the name of the order. For example, /end_order waffles".into()));
        assert_eq!(
            parse_command("/end_order waffles", WAFFLES_AND_PIZZA),
            Ok(EndOrder("waffles".into()))
        );
        assert_eq!(
            parse_command("/end_order pizza", WAFFLES_AND_PIZZA),
            Ok(EndOrder("pizza".into()))
        );

        assert_eq!(
            parse_command("/End_order Waffles ", WAFFLES),
            parse_command("/end_order waffles", WAFFLES),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/end_order Waffles", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(
            parse_command("/end_order Waffles", PIZZA),
            Err("Order waffles not found.".into())
        );
    }

    #[test]
    fn parse_order() {
        // no active orders
        assert_eq!(
            parse_command("/order", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(
            parse_command("/order chocolate", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(
            parse_command("/order waffles chocolate", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );

        // one active order
        assert_eq!(
            parse_command("/order", WAFFLES),
            Err(
                "Specify the name of the item you wish to order. For example, /order chocolate"
                    .into()
            ),
        );
        assert_eq!(
            parse_command("/order chocolate", WAFFLES),
            Ok(AddItem("waffles".into(), "chocolate".into())),
            "Order name may be omitted if there is only 1 active order"
        );
        assert_eq!(
            parse_command("/order Large Chocolate ", WAFFLES),
            Ok(AddItem("waffles".into(), "large chocolate".into())),
            "capitalization is ignored, and multi-word items are allowed"
        );
        assert_eq!(
            parse_command("/order waffles chocolate", WAFFLES),
            Ok(AddItem("waffles".into(), "chocolate".into())),
            "Order name may be specified even when there is only 1 active order"
        );
        assert_eq!(
            parse_command("/order waffles Large Chocolate", WAFFLES),
            Ok(AddItem("waffles".into(), "large chocolate".into())),
            "capitalization is ignored, and multi-word items are allowed"
        );

        // 2 active orders
        assert_eq!(
            parse_command("/order", WAFFLES_AND_PIZZA),
            Err("Specify the order name and item you wish to order. For example, /order waffles chocolate".into()),
        );
        assert_eq!(
            parse_command("/order chocolate", WAFFLES_AND_PIZZA),
            Err("Specify the order name and item you wish to order. For example, /order waffles chocolate".into()),
        );
        assert_eq!(
            parse_command("/order waffles", WAFFLES_AND_PIZZA),
            Err("Specify the order name and item you wish to order. For example, /order waffles chocolate".into()),
        );
        assert_eq!(
            parse_command("/order waffles chocolate", WAFFLES_AND_PIZZA),
            Ok(AddItem("waffles".into(), "chocolate".into())),
        );
        assert_eq!(
            parse_command("/order  waffles LARGE  CHOCOLATE ", WAFFLES_AND_PIZZA),
            Ok(AddItem("waffles".into(), "large chocolate".into())),
        );
        assert_eq!(
            parse_command("/order pizza Barbecue chicken", WAFFLES_AND_PIZZA),
            Ok(AddItem("pizza".into(), "barbecue chicken".into())),
        );
        assert_eq!(
            parse_command("/order ice-cream chocolate cone", WAFFLES_AND_PIZZA),
            Err("Order ice-cream not found. Specify the order name and item you wish to order. For example, /order waffles chocolate".into()),
        );
    }

    #[test]
    fn parse_cancel() {
        assert_eq!(
            parse_command("/cancel", NO_ORDERS),
            Err("There are no active orders. Start one by using /start_order <order name>".into())
        );
        assert_eq!(
            parse_command("/cancel", NO_ORDERS),
            parse_command("/cancel waffles", NO_ORDERS)
        );

        // 1 active order
        assert_eq!(
            parse_command("/cancel", WAFFLES),
            Ok(RemoveItem("waffles".into()))
        );
        assert_eq!(
            parse_command("/cancel Waffles", WAFFLES),
            Ok(RemoveItem("waffles".into()))
        );
        assert_eq!(
            parse_command("/cancel ice-cream", WAFFLES),
            Err("Order ice-cream not found.".into())
        );

        // 2 active orders
        assert_eq!(
            parse_command("/cancel", WAFFLES_AND_PIZZA),
            Err("As there are multiple active orders, Specify the name of the order. For example, /cancel waffles".into())
        );
        assert_eq!(
            parse_command("/cancel PIZZA ", WAFFLES_AND_PIZZA),
            Ok(RemoveItem("pizza".into()))
        );
        assert_eq!(
            parse_command("/cancel ice-cream", WAFFLES_AND_PIZZA),
            Err("Order ice-cream not found.".into())
        );
    }
}
