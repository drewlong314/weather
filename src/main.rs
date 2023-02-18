use std::env::args;

use weather::config;
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args();

    let config = config(args);

    println!("{}", config.url);
    weather::run(config).await.unwrap_or(());

    Ok(())
}
