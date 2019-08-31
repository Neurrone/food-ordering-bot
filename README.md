# Food-ordering-bot: a Telegram bot to facilitate group food orders, written in Rust

## Usage

Add `food_ordering_bot` to a group on Telegram, and use `/help` for a list of commands.

```sh
/start <order name> - starts an order. For example, /start waffles.
/view - shows active orders.

The following commands will ask for the order name, if there are multiple active orders.

/order [order name] <item> - adds an item to an order, or replaces the previously chosen one.
/cancel [order name] - removes your previously selected item from an order.
/end [order name] - stops an order.
```

## Building from source

As usual for Cargo, build with `cargo build`, run with `cargo run` and run unit tests with `cargo test`.

The bot expects its telegram bot token to be provided using the `TELEGRAM_BOT_TOKEN` environment variable, and panicks if not found.

## Running On Docker

```Rust
docker build -t food-ordering-bot .
docker run -e TELEGRAM_BOT_TOKEN=<token> -it food-ordering-bot
```

## License

MIT
