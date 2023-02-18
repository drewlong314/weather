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

fn convert_to_weather_condition(code: i32) -> String {
    match code {
        0 => String::from("Clear Sky"),
        1 => String::from("Mainly Clear"),
        2 => String::from("Partly Cloudy"),
        3 => String::from("Overcast"),
        45 => String::from("Fog"),
        48 => String::from("Depositing Rime Fog"),
        51 => String::from("Light Drizzle"),
        53 => String::from("Moderate Drizzle"),
        55 => String::from("Dense Drizzle"),
        56 => String::from("Light Freezing Drizzle"),
        57 => String::from("Dense Freezing Drizzle"),
        61 => String::from("Slight Rain"),
        63 => String::from("Moderate Rain"),
        65 => String::from("Heavy Rain"),
        66 => String::from("Light Freezing Rain"),
        67 => String::from("Heavy Freezing Rain"),
        71 => String::from("Slight Snow"),
        73 => String::from("Moderate Snow"),
        75 => String::from("Heavy Snow"),
        77 => String::from("Snow Grains"),
        80 => String::from("Slight Rain Shower"),
        81 => String::from("Moderate Rain Shower"),
        82 => String::from("Violent Rain Shower"),
        85 => String::from("Slught Snow Shower"),
        86 => String::from("Heavy Snow Shower"),
        95 => String::from("Thunderstorm"),
        96 => String::from("Thunderstorm with Slight Hail"),
        99 => String::from("Thunderstorm with Heavy Hail"),
        _ => String::from("Unknown Weather Code"),
    }
}
