use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};

#[derive(Debug, Clone)]
struct Event {
    date: NaiveDate,
    description: String,
    category: Option<String>,
    fg_color: Option<String>,
    bg_color: Option<String>,
    original_year: Option<i32>,
}

#[derive(Debug)]
struct Config {
    num_months: usize,
    start_month: u32,
    start_year: i32,
    monday_first: bool,
    show_calendar: bool,
    show_events: bool,
    num_columns: usize,
    show_week_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        let today = chrono::Local::now().naive_local().date();
        Config {
            num_months: 1,
            start_month: today.month(),
            start_year: today.year(),
            monday_first: true,
            show_calendar: true,
            show_events: true,
            num_columns: 3,
            show_week_numbers: true,
        }
    }
}

fn main() {
    let mut config = Config::default();
    let mut events_file = String::from("events.txt");
    let args: Vec<String> = std::env::args().collect();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--num-months" => {
                if i + 1 < args.len() {
                    config.num_months = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid number of months provided. Using 1.");
                        1
                    });
                    i += 2;
                } else { i += 1; }
            }
            "-m" | "--month" => {
                if i + 1 < args.len() {
                    config.start_month = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid month provided. Using current month.");
                        chrono::Local::now().naive_local().date().month()
                    });
                    if !(1..=12).contains(&config.start_month) {
                        eprintln!("Warning: Month must be between 1 and 12. Using current month.");
                        config.start_month = chrono::Local::now().naive_local().date().month();
                    }
                    i += 2;
                } else { i += 1; }
            }
            "-y" | "--year" => {
                if i + 1 < args.len() {
                    config.start_year = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid year provided. Using current year.");
                        chrono::Local::now().naive_local().date().year()
                    });
                    i += 2;
                } else { i += 1; }
            }
            "-cols" | "--columns" => {
                if i + 1 < args.len() {
                    config.num_columns = args[i + 1].parse().unwrap_or(3).max(1);
                    i += 2;
                } else { i += 1; }
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    events_file = args[i + 1].clone();
                    i += 2;
                } else { i += 1; }
            }
            "-sun" | "--sunday-first" => { config.monday_first = false; i += 1; }
            "-mon" | "--monday-first" => { config.monday_first = true; i += 1; }
            "-c" | "--calendar-only" => { config.show_calendar = true; config.show_events = false; i += 1; }
            "-e" | "--events-only" => { config.show_calendar = false; config.show_events = true; i += 1; }
            "-w" | "--weeks" => {
                if i + 1 < args.len() {
                    match args[i + 1].to_lowercase().as_str() {
                        "off" | "false" | "0" | "no" => { config.show_week_numbers = false; i += 2; }
                        "on" | "true" | "1" | "yes" => { config.show_week_numbers = true; i += 2; }
                        _ => { config.show_week_numbers = true; i += 1; }
                    }
                } else { config.show_week_numbers = true; i += 1; }
            }
            "-h" | "--help" => { print_help(); return; }
            _ => { i += 1; }
        }
    }

    let events = load_events(&events_file, &config);

    // OPTIMIZATION: Build a HashMap for O(1) event lookups during calendar rendering
    let mut event_map: HashMap<NaiveDate, &Event> = HashMap::new();
    for e in events.iter() {
        event_map.entry(e.date).or_insert(e); // Keeps the first event if multiple exist on the same day
    }

    if config.show_calendar {
        display_calendars(&config, &event_map);
    }
    if config.show_events {
        display_events_list(&config, &events);
    }
}

fn print_help() {
    println!("");
    println!("Calendar with Events");
    println!("----------------------------------------------------------------------------------");
    println!("\x1b[1m\x1b[33mUsage: ecal [OPTIONS]\x1b[0m");
    println!(" \x1b[1m\x1b[34m -m\x1b[0m    | \x1b[34m--month        \x1b[0m \x1b[32m<MONTH>\x1b[0m  Start month");
    println!(" \x1b[1m\x1b[34m -y\x1b[0m    | \x1b[34m--year         \x1b[0m \x1b[32m<YEAR>\x1b[0m   Start year");
    println!(" \x1b[1m\x1b[34m -n\x1b[0m    | \x1b[34m--num-months   \x1b[0m \x1b[32m<NUM>\x1b[0m    Number of months to display (1-12)");
    println!(" \x1b[1m\x1b[34m -cols\x1b[0m | \x1b[34m--columns      \x1b[0m \x1b[32m<NUM>\x1b[0m    Number of calendar columns per row (default: 3)");
    println!(" \x1b[1m\x1b[34m -mon\x1b[0m  | \x1b[34m--monday-first \x1b[0m          Week starts on Monday (default)");
    println!(" \x1b[1m\x1b[34m -sun\x1b[0m  | \x1b[34m--sunday-first \x1b[0m          Week starts on Sunday");
    println!(" \x1b[1m\x1b[34m -w\x1b[0m    | \x1b[34m--weeks        \x1b[0m \x1b[32m[on|off]\x1b[0m Show week numbers (default: on)");
    println!(" \x1b[1m\x1b[34m -c\x1b[0m    | \x1b[34m--calendar-only\x1b[0m          Show only calendar");
    println!(" \x1b[1m\x1b[34m -e\x1b[0m    | \x1b[34m--events-only  \x1b[0m          Show only events");
    println!(" \x1b[1m\x1b[34m -f\x1b[0m    | \x1b[34m--file         \x1b[0m \x1b[32m<PATH>\x1b[0m   Path to events file (default: events.txt)");
    println!(" \x1b[1m\x1b[34m -h\x1b[0m    | \x1b[34m--help         \x1b[0m          Display this help message");
}

fn parse_fixed_date_rule(rule: &str) -> Option<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(rule, "%d-%m-%Y") { return Some(date); }
    if let Ok(date) = NaiveDate::parse_from_str(rule, "%m/%d/%Y") { return Some(date); }
    if let Ok(date) = NaiveDate::parse_from_str(rule, "%Y-%m-%d") { return Some(date); }
    None
}

fn load_events(filename: &str, config: &Config) -> Vec<Event> {
    let mut events = Vec::new();
    let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + config.num_months as i64;
    let end_year_check = ((total_months_from_epoch - 1) / 12) as i32;

    if let Ok(file) = fs::File::open(filename) {
        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }

            let parts: Vec<&str> = line.splitn(2, ';').collect();
            let rule_part = parts[0].trim();
            let mut category: Option<String> = None;
            let mut fg_color: Option<String> = None;
            let mut bg_color: Option<String> = None;

            let description_text = if parts.len() > 1 {
                let rest = parts[1].trim();
                if rest.starts_with('[') {
                    if let Some(end_bracket) = rest.find(']') {
                        let meta_block = &rest[1..end_bracket];
                        let meta_parts: Vec<&str> = meta_block.split(',').map(|s| s.trim()).collect();
                        if let Some(cat) = meta_parts.first() { if !cat.is_empty() { category = Some(cat.to_string()); } }
                        if let Some(fg) = meta_parts.get(1) { if !fg.is_empty() { fg_color = Some(fg.to_string()); } }
                        if let Some(bg) = meta_parts.get(2) { if !bg.is_empty() { bg_color = Some(bg.to_string()); } }
                        rest[end_bracket + 1..].trim().to_string()
                    } else { rest.to_string() }
                } else { rest.to_string() }
            } else {
                match rule_part.split_once(char::is_whitespace) {
                    Some((_, desc)) => desc.trim().to_string(),
                    None => "".to_string(),
                }
            };

            let years_to_check = config.start_year..=end_year_check;
            let mut base_date: Option<NaiveDate> = None;
            let mut is_anniversary_rule = false;

            if let Some(date) = parse_fixed_date_rule(rule_part) {
                base_date = Some(date);
                if let Some(ref cat) = category {
                    if cat == "bday" || cat == "anni" { is_anniversary_rule = true; }
                }
                if !is_anniversary_rule {
                    if (config.start_year..=end_year_check).contains(&date.year()) {
                        if let Some(d) = NaiveDate::from_ymd_opt(date.year(), date.month(), date.day()) {
                            events.push(Event { date: d, description: description_text.clone(), category: category.clone(), fg_color: fg_color.clone(), bg_color: bg_color.clone(), original_year: None });
                        }
                    }
                    continue;
                }
            }

            let mut added_dates = std::collections::HashSet::new();
            for year in years_to_check {
                let mut date_to_add: Option<NaiveDate> = None;
                let mut original_year_to_store: Option<i32> = None;

                if is_anniversary_rule {
                    let bd = base_date.unwrap();
                    if year >= bd.year() {
                        date_to_add = NaiveDate::from_ymd_opt(year, bd.month(), bd.day());
                        original_year_to_store = Some(bd.year());
                    }
                } else if base_date.is_none() {
                    date_to_add = calculate_date_from_rule(rule_part, year);
                }

                if let Some(date) = date_to_add {
                    if added_dates.insert(date) {
                        events.push(Event { date, description: description_text.clone(), category: category.clone(), fg_color: fg_color.clone(), bg_color: bg_color.clone(), original_year: original_year_to_store });
                    }
                }
            }
        }
    } else {
        eprintln!("Info: Event file '{}' not found. Continuing without events.", filename);
    }

    events.sort_by_key(|e| e.date);
    events
}

fn calculate_date_from_rule(rule: &str, year: i32) -> Option<NaiveDate> {
    let rule = rule.trim();
    if rule.starts_with('E') {
        let offset = if rule == "E" { 0 } else { rule[1..].parse::<i64>().ok()? };
        return calculate_easter_date(year).map(|date| date + Duration::days(offset));
    }
    if let Some(hash_pos) = rule.find('#') {
        let date_part = &rule[0..hash_pos];
        let n_str = &rule[hash_pos + 1..];
        let mut parts = date_part.split('/');
        let month = parts.next()?.parse::<u32>().ok()?;
        let dow_num = parts.next()?.parse::<u32>().ok()?;
        let dow_num = if dow_num == 0 { 7 } else { dow_num };
        let n = n_str.parse::<u32>().ok()?;
        return find_nth_dow(year, month, dow_num, n);
    }
    if let Some(q_pos) = rule.find('?') {
        let date_part = &rule[0..q_pos];
        let condition_part = &rule[q_pos + 1..];
        let mut parts = date_part.split('/');
        let month = parts.next()?.parse::<u32>().ok()?;
        let day = parts.next()?.parse::<u32>().ok()?;
        let target_date = NaiveDate::from_ymd_opt(year, month, day)?;
        if condition_part.len() >= 3 {
            let target_dow_num = condition_part.chars().next()?.to_digit(10)?;
            let operator = condition_part.chars().nth(1)?;
            let offset = condition_part[2..].parse::<i64>().ok()?;
            let target_weekday = match target_dow_num {
                0 => Weekday::Sun, 1 => Weekday::Mon, 2 => Weekday::Tue, 3 => Weekday::Wed,
                4 => Weekday::Thu, 5 => Weekday::Fri, 6 => Weekday::Sat, _ => return None,
            };
            if target_date.weekday() == target_weekday {
                let duration = Duration::days(offset);
                let final_offset = match operator { '+' => duration, '-' => -duration, _ => return None };
                return Some(target_date + final_offset);
            }
        }
        if condition_part.is_empty() || condition_part.chars().all(|c| c.is_digit(10)) { return Some(target_date); }
        return None;
    }
    if rule.contains('/') && rule.chars().filter(|c| *c == '/').count() == 1 {
        let mut parts = rule.split('/');
        let month = parts.next()?.parse::<u32>().ok()?;
        let day = parts.next()?.parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    None
}

fn calculate_easter_date(year: i32) -> Option<NaiveDate> {
    if year < 1583 { return None; }
    let a = year % 19; let b = year / 100; let c = year % 100;
    let d = b / 4; let e = b % 4; let f = (b + 8) / 25; let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30; let i = c / 4; let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7; let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31; let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
}

fn find_nth_dow(year: i32, month: u32, dow_num: u32, n: u32) -> Option<NaiveDate> {
    if n == 0 || n > 5 || dow_num == 0 || dow_num > 7 { return None; }
    let target_weekday = match dow_num {
        1 => Weekday::Mon, 2 => Weekday::Tue, 3 => Weekday::Wed, 4 => Weekday::Thu,
        5 => Weekday::Fri, 6 => Weekday::Sat, 7 => Weekday::Sun, _ => return None,
    };
    let mut current_date = NaiveDate::from_ymd_opt(year, month, 1)?;
    while current_date.month() == month && current_date.weekday() != target_weekday {
        current_date += Duration::days(1);
    }
    if current_date.month() == month {
        current_date += Duration::weeks((n - 1) as i64);
        if n == 5 && current_date.month() != month { current_date -= Duration::weeks(1); }
        if current_date.month() == month { return Some(current_date); }
    }
    None
}

fn display_calendars(config: &Config, event_map: &HashMap<NaiveDate, &Event>) {
    let months_per_row = if config.num_months == 1 { 1 } else { config.num_columns.max(1) };
    let num_rows = (config.num_months + months_per_row - 1) / months_per_row;
    for row in 0..num_rows {
        let start_idx = row * months_per_row;
        let end_idx = std::cmp::min(start_idx + months_per_row, config.num_months);
        display_month_row(config, event_map, start_idx, end_idx);
        if row < num_rows - 1 { println!(); }
    }
}

fn display_month_row(config: &Config, event_map: &HashMap<NaiveDate, &Event>, start_idx: usize, end_idx: usize) {
    let mut dates = Vec::new();
    for idx in start_idx..end_idx {
        let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + idx as i64 - 1;
        let year = (total_months_from_epoch / 12) as i32;
        let month = (total_months_from_epoch % 12 + 1) as u32;
        dates.push(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
    }

    let calendar_width = if config.show_week_numbers { 24 } else { 21 };

    for (idx, date) in dates.iter().enumerate() {
        let month_name_str = format!("{} {}", month_name(date.month()), date.year());
        let padding = (calendar_width - month_name_str.len()) / 2;
        print!("{}\x1b[1m{}\x1b[0m", " ".repeat(padding), month_name_str);
        print!("{}", " ".repeat(calendar_width - padding - month_name_str.len()));
        if idx < dates.len() - 1 { print!("    "); }
    }
    println!();

    for idx in 0..dates.len() {
        print_weekday_header(config);
        if idx < dates.len() - 1 { print!("     "); }
    }
    println!();

    let max_weeks = dates.iter().map(|d| weeks_in_month(*d, config.monday_first)).max().unwrap_or(6);
    for week in 0..max_weeks {
        let is_empty_row = dates.iter().all(|&date| {
            let week_start_day = get_week_start_day(date, week, config.monday_first);
            let days_in_month = days_in_month(date.year(), date.month());
            week_start_day > days_in_month as i32 || week_start_day + 6 < 1
        });
        if !is_empty_row {
            for (idx, date) in dates.iter().enumerate() {
                print_week_row(*date, week, config, event_map);
                if idx < dates.len() - 1 { print!("    "); }
            }
            println!();
        }
    }
    if max_weeks < 6 { println!(); }
}

fn get_week_start_day(month_start: NaiveDate, week_num: usize, monday_first: bool) -> i32 {
    let offset = if monday_first {
        month_start.weekday().num_days_from_monday()
    } else {
        month_start.weekday().num_days_from_sunday()
    };
    (week_num * 7) as i32 - offset as i32 + 1
}

fn print_weekday_header(config: &Config) {
    if config.show_week_numbers {
        if config.monday_first { print!("\x1b[34mWk\x1b[0m Mo Tu We Th Fr \x1b[31mSa Su\x1b[0m"); }
        else { print!("\x1b[34mWk\x1b[0m \x1b[31mSu\x1b[0m Mo Tu We Th Fr \x1b[31mSa\x1b[0m"); }
    } else {
        if config.monday_first { print!("Mo Tu We Th Fr \x1b[31mSa Su\x1b[0m"); }
        else { print!("\x1b[31mSu\x1b[0m Mo Tu We Th Fr \x1b[31mSa\x1b[0m"); }
    }
}

fn print_week_row(month_start: NaiveDate, week_num: usize, config: &Config, event_map: &HashMap<NaiveDate, &Event>) {
    let days_in_month = days_in_month(month_start.year(), month_start.month());
    let start_day = get_week_start_day(month_start, week_num, config.monday_first);
    let today = chrono::Local::now().naive_local().date();

    if config.show_week_numbers {
        if start_day <= days_in_month as i32 && start_day + 6 >= 1 {
            let week_date = month_start + Duration::days((start_day - 1).max(0) as i64);
            print!("\x1b[34m{:2}\x1b[0m ", week_date.iso_week().week());
        } else {
            print!("   ");
        }
    }

    for day_offset in 0..7 {
        let day = start_day + day_offset;
        if day > 0 && day <= days_in_month as i32 {
            let current_date = NaiveDate::from_ymd_opt(month_start.year(), month_start.month(), day as u32).unwrap();
            let event_for_day = event_map.get(&current_date);
            let is_today = current_date == today;
            let is_weekend = matches!(current_date.weekday(), Weekday::Sat | Weekday::Sun);

            // OPTIMIZATION: Use a pre-allocated Vec to avoid String heap allocations in the inner loop
            let mut styles = Vec::with_capacity(4);

            if is_today {
                let bg = event_for_day.and_then(|e| e.bg_color.as_ref()).and_then(|c| get_ansi_color_code(c, false)).unwrap_or("\x1b[43m");
                styles.push(bg);
                styles.push("\x1b[1m"); // Bold
                styles.push("\x1b[30m"); // Black text
            } else if is_weekend {
                styles.push("\x1b[31m"); // Red for weekends
                if event_for_day.is_some() { styles.push("\x1b[1m"); }
            } else if let Some(event) = event_for_day {
                if let Some(fg) = event.fg_color.as_ref().and_then(|c| get_ansi_color_code(c, true)) { styles.push(fg); }
                if let Some(bg) = event.bg_color.as_ref().and_then(|c| get_ansi_color_code(c, false)) { styles.push(bg); }
                else { styles.push("\x1b[7m"); } // Inverse video fallback
                styles.push("\x1b[1m"); // Bold for events
            }

            for style in styles { print!("{}", style); }
            print!("{:2}\x1b[0m ", day);
        } else {
            print!("   ");
        }
    }
}

fn get_ordinal_suffix(n: i32) -> &'static str {
    if n % 100 >= 11 && n % 100 <= 13 { "th" }
    else {
        match n % 10 {
            1 => "st", 2 => "nd", 3 => "rd", _ => "th",
        }
    }
}

fn display_events_list(config: &Config, events: &Vec<Event>) {
    let today = chrono::Local::now().naive_local().date();
    let start_date = NaiveDate::from_ymd_opt(config.start_year, config.start_month, 1).unwrap();
    let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + config.num_months as i64;
    let end_year = ((total_months_from_epoch - 1) / 12) as i32;
    let end_month = ((total_months_from_epoch - 1) % 12 + 1) as u32;
    let end_date = NaiveDate::from_ymd_opt(end_year, end_month, 1).unwrap();

    let filtered_events: Vec<&Event> = events.iter().filter(|e| e.date >= start_date && e.date < end_date).collect();
    if filtered_events.is_empty() { return; }

    println!("\n\x1b[1mEvents:\x1b[0m");
    println!("{}", "-".repeat(80));

    for event in filtered_events {
        let fg_code = event.fg_color.as_ref().and_then(|c| get_ansi_color_code(c, true)).unwrap_or("");
        let bg_code = event.bg_color.as_ref().and_then(|c| get_ansi_color_code(c, false)).unwrap_or("");

        let mut full_description = event.description.clone();
        if let Some(original_year) = event.original_year {
            if let Some(cat) = &event.category {
                let (label, qualifies) = match cat.as_str() {
                    "bday" => ("Birthday", true),
                    "anni" => ("Anniversary", true),
                    _ => ("", false),
                };
                if qualifies {
                    let anniversary_num = event.date.year() - original_year;
                    if anniversary_num > 0 {
                        full_description.push_str(&format!(" ({}{} {})", anniversary_num, get_ordinal_suffix(anniversary_num), label));
                    }
                }
            }
        }

        let days_diff = event.date.signed_duration_since(today).num_days();
        let relative_days_label = if days_diff == 0 {
            format!(" \x1b[1m\x1b[33m(Today 📌)\x1b[0m")
        } else if days_diff > 0 {
            format!(" \x1b[32m(In \x1b[1m{}\x1b[0m\x1b[32m days)\x1b[0m", days_diff)
        } else {
            format!(" \x1b[34m(\x1b[1m{}\x1b[0m\x1b[34m days ago)\x1b[0m", days_diff.abs())
        };
        full_description.push_str(&relative_days_label);

        println!("{}{}{}\x1b[0m - {}", bg_code, fg_code, event.date.format("%a, %d %b %Y"), full_description);
    }
}

fn get_ansi_color_code(color_name: &str, is_fg: bool) -> Option<&'static str> {
    match color_name.to_lowercase().as_str() {
        "black" => Some(if is_fg { "\x1b[30m" } else { "\x1b[40m" }),
        "red" => Some(if is_fg { "\x1b[31m" } else { "\x1b[41m" }),
        "green" => Some(if is_fg { "\x1b[32m" } else { "\x1b[42m" }),
        "yellow" => Some(if is_fg { "\x1b[33m" } else { "\x1b[43m" }),
        "blue" => Some(if is_fg { "\x1b[34m" } else { "\x1b[44m" }),
        "magenta" => Some(if is_fg { "\x1b[35m" } else { "\x1b[45m" }),
        "cyan" => Some(if is_fg { "\x1b[36m" } else { "\x1b[46m" }),
        "white" => Some(if is_fg { "\x1b[37m" } else { "\x1b[47m" }),
        _ => None,
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March", 4 => "April", 5 => "May", 6 => "June",
        7 => "July", 8 => "August", 9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(if month == 12 { year + 1 } else { year }, if month == 12 { 1 } else { month + 1 }, 1)
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

fn weeks_in_month(month_start: NaiveDate, monday_first: bool) -> usize {
    let offset = if monday_first {
        month_start.weekday().num_days_from_monday()
    } else {
        month_start.weekday().num_days_from_sunday()
    };
    let days = days_in_month(month_start.year(), month_start.month());
    ((offset + days + 6) / 7) as usize
}
