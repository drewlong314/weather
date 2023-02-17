use serde::Deserialize;
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug)]
    struct CurrentWeather {
        temperature: f32,
        weathercode: i32,
    }

    #[derive(Deserialize)]
    struct Orange {
        current_weather: CurrentWeather,
    }

    let resp =
        reqwest::get("https://api.open-meteo.com/v1/forecast?latitude=33.52&longitude=-86.80&current_weather=true&temperature_unit=fahrenheit")
            .await?
            .json::<Orange>()
            .await?;

    let current_weather = resp.current_weather;
    println!(
        "Current Temperature: {:#?}°F",
        current_weather.temperature as i32
    );
    println!(
        "Current Weather Condition: {}",
        convert_to_weather_condition(current_weather.weathercode)
    );

    Ok(())
}

fn convert_to_weather_condition(code: i32) -> String {
    match code {
        51 => String::from("Light Drizzle"),
        _ => String::from("Unknown Weather Code"),
    }
}
