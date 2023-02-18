use std::env::args;

use weather::config;
#[tokio::main]

async fn main() {
    let args = args();

    let config = config(args);

    println!("{}", config.url);
    weather::run(config).await.unwrap_or(());
}
