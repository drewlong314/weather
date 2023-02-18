use std::{env::args, process};

use weather::config;
#[tokio::main]

async fn main() {
    let args = args();

    let config = config(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("{}", config.url);
    weather::run(config).await.unwrap_or(());
}
