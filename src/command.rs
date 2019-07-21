#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    /// starts a new order for this conversation
    StartOrder(String),
    /// ends the currently active order for the conversation
    EndOrder,
    /// adds an item to the currently active order
    AddItem(String),
    /// Cancels the currently selected item
    RemoveItem,
    /// view the current order
    ViewOrder,
}

type ParseResult = std::result::Result<Command, String>;

pub fn parse_command(message: &str) -> ParseResult {
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
        "/end_order" => Ok(EndOrder),
        "/order" => {
            if args.len() == 0 {
                Err(
                    "Specify the name of the item you wish to order. For example, /order chocolate"
                        .into(),
                )
            } else {
                Ok(AddItem(args.join(" ")))
            }
        }
        "/cancel" => Ok(RemoveItem),
        "/view_order" => Ok(ViewOrder),
        _ => Err(format!("Invalid command")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;

    #[test]
    fn parse_unrecognized_command_errors() {
        assert!(parse_command("/invalid_command").is_err());
    }

    #[test]
    fn parse_start_order() {
        assert_eq!(
            parse_command("/start_order "),
            Err("Specify the name of the order. For example, /start_order waffles".into())
        );
        assert_eq!(
            parse_command("/start_order waffles"),
            Ok(StartOrder("waffles".into()))
        );
        assert_eq!(
            parse_command("/Start_order WAFFLES "),
            parse_command("/start_order waffles"),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/start_order waffles @food_ordering_bot"),
            parse_command("/start_order waffles"),
            "@mentions are ignored"
        );
        assert_eq!(
            parse_command("/start_order mr bean"),
            Err("Order names must not contain spaces. Try /start_order mr-bean".into())
        );
        assert_eq!(
            parse_command("/start_order mr-bean"),
            Ok(StartOrder("mr-bean".into()))
        );
    }

    #[test]
    fn parse_end_order() {
        assert_eq!(parse_command("/end_order"), Ok(EndOrder));
        assert_eq!(
            parse_command("/End_order  "),
            parse_command("/end_order"),
            "whitespace and capitalization are ignored"
        );
        assert_eq!(
            parse_command("/end_order waffles"),
            parse_command("/end_order"),
            "Excess arguments are ignored"
        );
    }

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
}
