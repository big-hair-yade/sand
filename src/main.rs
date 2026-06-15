use clap::Parser;
use crossterm::{cursor, queue, terminal};
use regex::Regex;
use std::io::{Write, stdout};
use std::thread::sleep;
use std::time::Duration;

mod circle;
use crate::circle::{Circle, DrawCircle, resize_hourglass};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short,long, default_value_t = String::from("01h30m10s"))]
    time: String,
    #[arg(short, long, default_value_t = 250)]
    refresh: u32,
}

fn main() {
    let args = Args::parse();
    let days: i32 = parse_time(args.time.clone(), String::from("d"));
    let hours: i32 = parse_time(args.time.clone(), String::from("h"));
    let minutes: i32 = parse_time(args.time.clone(), String::from("m"));
    let seconds: i32 = parse_time(args.time.clone(), String::from("s"));
    let refresh = args.refresh.clone();

    println!(
        "Extracted {} days, {} hours, {} minutes, and {} seconds",
        days.to_string(),
        hours.to_string(),
        minutes.to_string(),
        seconds.to_string()
    );

    if hours < 0 || minutes < 0 || seconds < 0 || days < 0 {
        eprintln!("Error: Negative time values are not supported");
        std::process::exit(1);
    }

    let total_secs = (days * 86400 + hours * 3600 + minutes * 60 + seconds) as u64;
    let mut top_circle = Circle::default();
    let mut bottom_circle = Circle::default();

    let (mut old_size_x, mut old_size_y): (u16, u16) = (0, 0);
    let (mut size_x, mut size_y);
    let num_draws: u32 = total_secs as u32 * 1000 / refresh;
    let mut current_frac: f32;
    let _ = queue!(
        stdout(),
        terminal::Clear(terminal::ClearType::Purge),
        cursor::Hide
    );
    for redraw in 0..num_draws {
        (size_x, size_y) = terminal::size().expect("crossterm failed to report terminal size");
        if size_x != old_size_x || size_y != old_size_y {
            let _ = queue!(stdout(), terminal::Clear(terminal::ClearType::Purge));
            resize_hourglass(&mut top_circle, &mut bottom_circle, size_x, size_y);
            (old_size_x, old_size_y) = (size_x, size_y);
        }
        current_frac = redraw as f32 / num_draws as f32;
        top_circle.draw_circle(1.0 - current_frac, '#', '.');
        bottom_circle.draw_circle(current_frac, '#', '.');
        let _ = stdout().flush();
        sleep(Duration::from_millis(refresh as u64));
    }
    top_circle.draw_circle(0.0, '#', '.');
    bottom_circle.draw_circle(1.01, '#', '.'); //1.01 just to make sure that we don't deal with any
    let _ = stdout().flush();
    sleep(Duration::new(1, 0));

    let _ = queue!(
        stdout(),
        terminal::Clear(terminal::ClearType::Purge),
        cursor::MoveTo(0, 0)
    );
}

fn parse_time(time_string: String, unit_marker: String) -> i32 {
    let re = Regex::new(format!(r"(?<time>-?[0-9]+){}", unit_marker).as_str()).unwrap();
    let hay = &time_string;
    let Some(caps) = re.captures(hay) else {
        return 0;
    };
    caps["time"].parse().expect("Failed to parse time value")
}
