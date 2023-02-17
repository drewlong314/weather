use std::env::args;

use serde::Deserialize;
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug)]
    struct CurrentWeather {
        temperature: f32,
        weathercode: i32,
    }

    #[derive(Deserialize)]
    struct Response {
        current_weather: CurrentWeather,
    }

    // TODO: Add option to set location and Temperature Unit in an env
    // TODO: Show full day's forecast
    // TODO: Show full week's Highs and Low's
    // TODO: Create a lib.rs so that main.rs is more readable

    let mut temperature_unit = "C";
    let mut url = String::from("https://api.open-meteo.com/v1/forecast?latitude=33.52&longitude=-86.80&current_weather=true");
    let args = args();
    if args.len() > 0 {
        for arg in args {
            match arg.as_str() {
                "F" => {
                    url = url + "&temperature_unit=fahrenheit";
                    temperature_unit = "F";
                }
                _ => (),
            }
        }
    }

    let resp = reqwest::get(url).await?.json::<Response>().await?;

    let current_weather = resp.current_weather;
    println!(
        "Current Temperature: {:#?}°{}",
        current_weather.temperature as i32, temperature_unit
    );
    println!(
        "Current Weather Condition: {}",
        convert_to_weather_condition(current_weather.weathercode)
    );

    Ok(())
}

// TODO: Add the rest of the weather conditions
fn convert_to_weather_condition(code: i32) -> String {
    match code {
        51 => String::from("Light Drizzle"),
        _ => String::from("Unknown Weather Code"),
    }
}
