use std::{cmp::Ordering, env::Args};

use chrono::{Datelike, Duration, Local, NaiveDateTime, TimeZone};
use geocoding::{Forward, GeocodingError, Opencage, Point};
use serde::Deserialize;
use tokio::task::spawn_blocking;

pub struct Config {
    pub url: String,
    pub temperature_unit: String,
}

// Maybe put this in an impl
pub async fn config(args: Args) -> Result<Config, &'static str> {
    let mut temperature_unit = String::from("C");
    let mut url = String::from("https://api.open-meteo.com/v1/forecast");

    if args.len() == 1 {
        url = String::new();
    }

    if args.len() < 5 {
        for (i, arg) in args.enumerate() {
            if i == 1 {
                let orange = spawn_blocking(move || {
                    let oc = Opencage::new("ac6788448df84c38a6afac4786c87b37".to_string());
                    let address = arg;

                    let res: Result<Vec<Point<f64>>, GeocodingError> = oc.forward(&address);
                    match res {
                        Ok(ref res) => res,
                        Err(err) => return Err(err),
                    };
                    let first_result = res.unwrap()[0];

                    Ok(format!(
                        "?latitude={}&longitude={}",
                        first_result.y(),
                        first_result.x()
                    ))
                })
                .await
                .unwrap();

                match orange {
                    Ok(ref orange) => url = url + orange,
                    Err(_) => return Err("Invalid location"),
                };

                // 11.5761796, 48.1599218
            } else if i == 2 {
                match arg.as_str() {
                    "current" => {
                        url = url + "&current_weather=true&timezone=auto";
                    }
                    "week" => {
                        let now = Local::now();
                        let week = (now + Duration::days(7)).format("%Y-%m-%d");
                        let formatted_now = now.format("%Y-%m-%d");
                        url = url + &format!("&daily=weathercode,temperature_2m_max,temperature_2m_min&timezone=auto&start_date={formatted_now}&end_date={week}");
                    }
                    "today" => {
                        let now = Local::now();
                        let tomorrow = (now + Duration::days(1)).format("%Y-%m-%d");
                        let formatted_now = now.format("%Y-%m-%d");
                        url = url + &format!("&hourly=temperature_2m,weathercode&timezone=auto&start_date={formatted_now}&end_date={tomorrow}");
                    }
                    "help" => {
                        url = String::new();
                    }
                    "h" => {
                        url = String::new();
                    }
                    _ => url = String::new(),
                }
            } else if i == 3 {
                match arg.as_str() {
                    "-F" => {
                        url = url + "&temperature_unit=fahrenheit";
                        temperature_unit = String::from("F");
                    }
                    "-C" => (),
                    _ => return Err("Incorrect second argument"),
                }
            }
        }
    } else {
        return Err("Too many arguments");
    }

    Ok(Config {
        url,
        temperature_unit,
    })
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

    #[derive(Deserialize, Debug)]
    struct Hourly {
        time: Vec<String>,
        weathercode: Vec<i32>,
        temperature_2m: Vec<f32>,
    }

    #[derive(Deserialize)]
    struct Help {
        message: String,
    }

    #[derive(Deserialize)]
    struct Response {
        current_weather: Option<CurrentWeather>,
        daily: Option<Daily>,
        hourly: Option<Hourly>,
        help: Option<Help>,
    }

    let help_message = String::from(
        "Weather - A CLI Weather App

Usage:
    weather [COMMAND] [OPTION]

Commands:
    current
        Prints today's current forecast

    today
        Prints today's hourly forecast

    week
        Prints this week's forecast

    help
        Print this message

Options:
    -F
        Convert's the temperature from Celcius to Farenheit",
    );

    let resp = if url.len() > 1 {
        reqwest::get(url).await?.json::<Response>().await?
    } else {
        Response {
            current_weather: None,
            daily: None,
            hourly: None,
            help: Some(Help {
                message: help_message,
            }),
        }
    };

    if resp.current_weather.is_some() {
        let current_weather = resp.current_weather.unwrap();
        println!(
            "Current Temperature: {}°{}",
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
            let mut new_line = String::from("\n");

            if i == 6 {
                new_line = String::from("");
            }

            println!(
                "{:?} - {}\nHigh: {}°{temperature_unit}\nLow: {}°{temperature_unit}{}",
                chrono::NaiveDate::from_ymd_opt(
                    date[0].parse::<i32>().unwrap(),
                    date[1].parse::<u32>().unwrap(),
                    date[2].parse::<u32>().unwrap()
                )
                .unwrap()
                .weekday(),
                convert_to_weather_condition(daily_weather.weathercode[i]),
                daily_weather.temperature_2m_max[i] as i32,
                daily_weather.temperature_2m_min[i] as i32,
                new_line
            )
        }
    }

    if resp.hourly.is_some() {
        let daily_weather = resp.hourly.unwrap();
        let local_time = Local::now();

        let mut num = 0;
        for i in 0..48 {
            let date = &daily_weather.time[i];
            let from = NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M").unwrap();
            let date_time = Local.from_local_datetime(&from).unwrap();

            // If the date is in the future
            if local_time.cmp(&date_time) == Ordering::Less && num < 25 {
                num += 1;
                println!(
                    "{} - {} {}°{temperature_unit}",
                    date_time.format("%l%P"),
                    convert_to_weather_condition(daily_weather.weathercode[i]),
                    daily_weather.temperature_2m[i] as i32,
                )
            }
        }
    }

    if resp.help.is_some() {
        println!("{}", resp.help.unwrap().message);
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
