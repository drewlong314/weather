use std::env::Args;

use chrono::{Datelike, Duration, Local};
use serde::Deserialize;

// TODO: Add option to set location and Temperature Unit in an env
// TODO: Show full day's forecast
pub struct Config {
    pub url: String,
    pub temperature_unit: String,
}

pub fn config(args: Args) -> Config {
    let mut temperature_unit = String::from("C");
    let mut url =
        String::from("https://api.open-meteo.com/v1/forecast?latitude=33.52&longitude=-86.80");

    if args.len() > 0 {
        for arg in args {
            match arg.as_str() {
                "F" => {
                    url = url + "&temperature_unit=fahrenheit";
                    temperature_unit = String::from("F");
                }
                "current" => {
                    url = url + "&current_weather=true&timezone=auto";
                }
                "week" => {
                    let now = Local::now();
                    let week = (now + Duration::days(7)).format("%Y-%m-%d");
                    let formatted_now = now.format("%Y-%m-%d");
                    url = url + &format!("&daily=weathercode,temperature_2m_max,temperature_2m_min&timezone=auto&start_date={formatted_now}&end_date={week}");
                }
                _ => (),
            }
        }
    }

    Config {
        url,
        temperature_unit,
    }
}

pub async fn run(
    Config {
        url,
        temperature_unit,
    }: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug)]
    struct CurrentWeather {
        temperature: f32,
        weathercode: i32,
    }

    #[derive(Deserialize, Debug)]
    struct Daily {
        time: Vec<String>,
        weathercode: Vec<i32>,
        temperature_2m_max: Vec<f32>,
        temperature_2m_min: Vec<f32>,
    }

    #[derive(Deserialize)]
    struct Response {
        current_weather: Option<CurrentWeather>,
        daily: Option<Daily>,
    }

    let resp = reqwest::get(url).await?.json::<Response>().await?;

    if resp.current_weather.is_some() {
        let current_weather = resp.current_weather.unwrap();
        println!(
            "Current Temperature: {:#?}°{}",
            current_weather.temperature as i32, temperature_unit
        );
        println!(
            "Current Weather Condition: {}",
            convert_to_weather_condition(current_weather.weathercode)
        );
    }

    if resp.daily.is_some() {
        let daily_weather = resp.daily.unwrap();

        for i in 0..7 {
            let date: Vec<&str> = daily_weather.time[i].split("-").collect();
            println!(
                "{:?} - {}\nHigh: {}°{temperature_unit}\nLow: {}°{temperature_unit}\n",
                chrono::NaiveDate::from_ymd_opt(
                    date[0].parse::<i32>().unwrap(),
                    date[1].parse::<u32>().unwrap(),
                    date[2].parse::<u32>().unwrap()
                )
                .unwrap()
                .weekday(),
                convert_to_weather_condition(daily_weather.weathercode[i]),
                daily_weather.temperature_2m_max[i] as i32,
                daily_weather.temperature_2m_min[i] as i32
            )
        }
    }

    Ok(())
}

// TODO: Add the rest of the weather conditions
fn convert_to_weather_condition(code: i32) -> String {
    match code {
        51 => String::from("Light Drizzle"),
        _ => String::from("Unknown Weather Code"),
    }
}
