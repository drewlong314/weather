use serde::Deserialize;
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct Orange {
        latitude: f32,
    }

    let resp =
        reqwest::get("https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41")
            .await?
            .json::<Orange>()
            .await?;
    println!("{:#?}", resp.latitude);

    Ok(())
}
