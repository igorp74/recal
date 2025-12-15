use chrono::{Datelike, Duration, NaiveDate, Weekday};
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
        // Get today's date for default start month/year
        let today = chrono::Local::now().naive_local().date();
        Config {
            num_months: 1,
            start_month: today.month(),
            start_year: today.year(),
            monday_first: true,
            show_calendar: true,
            show_events: true,
            // DEFAULT: 3 columns for multi-month view
            num_columns: 3,
            // DEFAULT: Show week numbers
            show_week_numbers: true,
        }
    }
}

fn main() {
    let mut config = Config::default();
    let mut events_file = String::from("events.txt");
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
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
                } else {
                    i += 1;
                }
            }
            "-m" | "--month" => {
                if i + 1 < args.len() {
                    config.start_month = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid month provided. Using current month.");
                        chrono::Local::now().naive_local().date().month()
                    });
                    // Validate month is in range 1-12
                    if config.start_month < 1 || config.start_month > 12 {
                        eprintln!("Warning: Month must be between 1 and 12. Using current month.");
                        config.start_month = chrono::Local::now().naive_local().date().month();
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-y" | "--year" => {
                if i + 1 < args.len() {
                    config.start_year = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid year provided. Using current year.");
                        chrono::Local::now().naive_local().date().year()
                    });
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-cols" | "--columns" => {
                if i + 1 < args.len() {
                    config.num_columns = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid number of columns provided. Using 3.");
                        3
                    });
                    if config.num_columns == 0 {
                         config.num_columns = 1;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    events_file = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-sun" | "--sunday-first" => {
                config.monday_first = false;
                i += 1;
            }
            "-mon" | "--monday-first" => {
                config.monday_first = true;
                i += 1;
            }
            "-c" | "--calendar-only" => {
                config.show_calendar = true;
                config.show_events = false;
                i += 1;
            }
            "-e" | "--events-only" => {
                config.show_calendar = false;
                config.show_events = true;
                i += 1;
            }
            "-w" | "--weeks" => {
                // Peek at next arg to see if it's "on" or "off"
                if i + 1 < args.len() {
                    match args[i + 1].to_lowercase().as_str() {
                        "off" | "false" | "0" | "no" => {
                            config.show_week_numbers = false;
                            i += 2;
                        }
                        "on" | "true" | "1" | "yes" => {
                            config.show_week_numbers = true;
                            i += 2;
                        }
                        _ => {
                            // Next arg is not a boolean switch, so treat -w as a simple flag (enable)
                            config.show_week_numbers = true;
                            i += 1;
                        }
                    }
                } else {
                    // End of args, just enable
                    config.show_week_numbers = true;
                    i += 1;
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Load events from file
    let events = load_events(&events_file, &config);

    // Display calendar and/or events
    if config.show_calendar {
        display_calendars(&config, &events);
    }

    if config.show_events {
        display_events_list(&config, &events);
    }
}

fn print_help() {
    println!("Calendar with Events");
    println!("----------------------------------------------------------------------------------");
    println!("\x1b[1mUsage: ecal [OPTIONS]\x1b[0m");
    println!(" \x1b[1m\x1b[32m -m\x1b[0m   ,  --mont <MONTH>      Start month");
    println!(" \x1b[1m\x1b[32m -y\x1b[0m   ,  --year <YEAR>       Start year");
    println!(" \x1b[1m\x1b[32m -n\x1b[0m   ,  --num-months <NUM>  Number of months to display (1-12)");
    println!(" \x1b[1m\x1b[32m -cols\x1b[0m,  --columns <NUM>     Number of calendar columns per row (default: 3)");
    println!(" \x1b[1m\x1b[32m -mon\x1b[0m ,  --monday-first      Week starts on Monday (default)");
    println!(" \x1b[1m\x1b[32m -sun\x1b[0m ,  --sunday-first      Week starts on Sunday");
    println!(" \x1b[1m\x1b[32m -w\x1b[0m   ,  --weeks [on|off]    Show week numbers (default: on)");
    println!(" \x1b[1m\x1b[32m -c\x1b[0m   ,  --calendar-only     Show only calendar");
    println!(" \x1b[1m\x1b[32m -e\x1b[0m   ,  --events-only       Show only events");
    println!(" \x1b[1m\x1b[32m -f\x1b[0m   ,  --file <PATH>       Path to events file (default: events.txt)");
    println!(" \x1b[1m\x1b[32m -h\x1b[0m   ,  --help              Display this help message");
}

/// Helper to parse fixed dates in DD-MM-YYYY or MM/DD/YYYY format.
fn parse_fixed_date_rule(rule: &str) -> Option<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(rule, "%d-%m-%Y") {
        Some(date)
    } else if let Ok(date) = NaiveDate::parse_from_str(rule, "%m/%d/%Y") {
        Some(date)
    } else if let Ok(date) = NaiveDate::parse_from_str(rule, "%Y-%m-%d") {
        Some(date)
    } else {
        None
    }
}


fn load_events(filename: &str, config: &Config) -> Vec<Event> {
    let mut events = Vec::new();

    // Determine the range of years we need to check for recurring events.
    let _start_date = NaiveDate::from_ymd_opt(config.start_year, config.start_month, 1).unwrap();

    // Calculate end date (exclusive) to find the latest year we need to check.
    let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + config.num_months as i64;
    let end_year_check = ((total_months_from_epoch - 1) / 12) as i32;

    if let Ok(file) = fs::File::open(filename) {
        let reader = io::BufReader::new(file);

        for (_line_num, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Split rule from description/metadata (using the first semicolon)
                let parts: Vec<&str> = line.splitn(2, ';').collect();
                let rule_part = parts[0].trim();

                let mut category: Option<String> = None;
                let mut fg_color: Option<String> = None;
                let mut bg_color: Option<String> = None;

                // 1. Extract the base description AND colors/category
                let description_text = if parts.len() > 1 {
                    let rest = parts[1].trim();

                    // Check for and parse the metadata block: [category, fg, bg, emoji]
                    if rest.starts_with('[') {
                        if let Some(end_bracket) = rest.find(']') {
                            let meta_block = &rest[1..end_bracket];
                            let meta_parts: Vec<&str> = meta_block.split(',')
                                .map(|s| s.trim())
                                .collect();

                            // The format is [category, fg_color, bg_color, emoji]

                            // 0. Category
                            if meta_parts.len() > 0 && !meta_parts[0].is_empty() {
                                category = Some(meta_parts[0].to_string());
                            }
                            // 1. fg_color
                            if meta_parts.len() > 1 && !meta_parts[1].is_empty() {
                                fg_color = Some(meta_parts[1].to_string());
                            }
                            // 2. bg_color
                            if meta_parts.len() > 2 && !meta_parts[2].is_empty() {
                                bg_color = Some(meta_parts[2].to_string());
                            }

                            // Return the description starting after the closing bracket
                            rest[end_bracket + 1..].trim().to_string()
                        } else {
                            // No closing bracket, treat the whole rest as description
                            rest.to_string()
                        }
                    } else {
                        // No metadata block, treat the whole rest as description
                        rest.to_string()
                    }
                } else {
                    // For rules without semicolon, try to get description (less common for eCal rules)
                    match rule_part.split_once(char::is_whitespace) {
                        Some((_, desc)) => desc.trim().to_string(),
                        None => "".to_string(),
                    }
                };

                // 2. Determine the year range and recurrence
                let years_to_check = config.start_year..=end_year_check;
                let mut base_date: Option<NaiveDate> = None;
                let mut is_anniversary_rule = false;

                // Check for Fixed Date Rule
                if let Some(date) = parse_fixed_date_rule(rule_part) {
                    base_date = Some(date);
                    // Check if category is 'bday' or 'anni' to enable annual recurrence.
                    if let Some(ref cat) = category {
                        if cat == "bday" || cat == "anni" {
                            is_anniversary_rule = true;
                        }
                    }

                    // If it's a fixed date rule but NOT an anniversary/bday,
                    // we only process it for the exact year it specifies.
                    if !is_anniversary_rule {
                        // Check if the fixed year matches the current display range start year
                        if date.year() >= config.start_year && date.year() <= end_year_check {
                             if let Some(date_to_add) = NaiveDate::from_ymd_opt(date.year(), date.month(), date.day()) {
                                 events.push(Event {
                                    date: date_to_add,
                                    description: description_text.clone(),
                                    category: category.clone(),
                                    fg_color: fg_color.clone(),
                                    bg_color: bg_color.clone(),
                                    original_year: None,
                                 });
                             }
                        }
                        continue;
                    }
                }

                let mut added_years = std::collections::HashSet::new();

                for year in years_to_check {
                    let mut date_to_add: Option<NaiveDate> = None;
                    let mut original_year_to_store: Option<i32> = None;

                    if is_anniversary_rule {
                        // Recur the anniversary from the base date
                        let bd = base_date.unwrap();
                        if year >= bd.year() {
                            date_to_add = NaiveDate::from_ymd_opt(year, bd.month(), bd.day());
                            original_year_to_store = Some(bd.year());
                        }
                    } else if base_date.is_none() {
                        // Standard eCal rule (E+1, 5/1#1, 7/4)
                        date_to_add = calculate_date_from_rule(rule_part, year);
                    }

                    if let Some(date) = date_to_add {
                        if added_years.insert(date) {
                            events.push(Event {
                                date,
                                description: description_text.clone(),
                                category: category.clone(),
                                fg_color: fg_color.clone(),
                                bg_color: bg_color.clone(),
                                original_year: original_year_to_store,
                            });
                        }
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

// ====================================================================
// DATE RULE PARSING LOGIC
// ====================================================================

/// Tries to calculate the date for a given rule string and target year.
fn calculate_date_from_rule(rule: &str, year: i32) -> Option<NaiveDate> {
    let rule = rule.trim();

    // 1. Easter relative rule: E[+-]N (E+1, E-2, E)
    if rule.starts_with('E') {
        let offset = if rule == "E" {
            0
        } else if rule.len() > 1 {
            rule[1..].parse::<i64>().ok()?
        } else {
            return None;
        };
        return calculate_easter_date(year).map(|date| date + Duration::days(offset));
    }

    // 2. Nth Day of Week rule: MM/DOW#N
    if let Some(hash_pos) = rule.find('#') {
        let date_part = &rule[0..hash_pos];
        let n_str = &rule[hash_pos + 1..];

        let mut parts = date_part.split('/');
        let month = parts.next()?.parse::<u32>().ok()?;
        let dow_num = parts.next()?.parse::<u32>().ok()?; // DOW: 1=Mon..0=Sun
        let dow_num = if dow_num == 0 { 7 } else { dow_num };
        let n = n_str.parse::<u32>().ok()?;

        return find_nth_dow(year, month, dow_num, n);
    }

    // 3. Conditional/Bank Holiday rule: MM/DD?D[+-]N
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
                0 => Weekday::Sun,
                1 => Weekday::Mon,
                2 => Weekday::Tue,
                3 => Weekday::Wed,
                4 => Weekday::Thu,
                5 => Weekday::Fri,
                6 => Weekday::Sat,
                _ => return None,
            };

            if target_date.weekday() == target_weekday {
                let duration = Duration::days(offset);
                let final_offset = match operator {
                    '+' => duration,
                    '-' => -duration,
                    _ => return None,
                };
                return Some(target_date + final_offset);
            }
        }
        if condition_part.is_empty() || condition_part.chars().all(|c| c.is_digit(10)) {
             return Some(target_date);
        }
        return None;
    }

    // 4. Annual rule (MM/DD)
    if rule.contains('/') && rule.chars().filter(|c| *c == '/').count() == 1 {
        let mut parts = rule.split('/');
        let month = parts.next()?.parse::<u32>().ok()?;
        let day = parts.next()?.parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }

    None
}

/// Calculates the date of Easter Sunday for a given year using the Gauss algorithm.
fn calculate_easter_date(year: i32) -> Option<NaiveDate> {
    if year < 1583 { return None; }

    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;

    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;

    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
}

/// Finds the Nth day of week (DOW) in a given month of a year.
fn find_nth_dow(year: i32, month: u32, dow_num: u32, n: u32) -> Option<NaiveDate> {
    if n == 0 || n > 5 || dow_num == 0 || dow_num > 7 {
        return None;
    }

    let target_weekday = match dow_num {
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        7 => Weekday::Sun,
        _ => return None,
    };

    let first_day_of_month = NaiveDate::from_ymd_opt(year, month, 1)?;
    let mut current_date = first_day_of_month;

    // Find the first occurrence of the target weekday
    while current_date.month() == month && current_date.weekday() != target_weekday {
        current_date += Duration::days(1);
    }

    // Jump forward (n-1) weeks
    if current_date.month() == month {
        current_date += Duration::weeks((n - 1) as i64);

        if n == 5 && current_date.month() != month {
            current_date -= Duration::weeks(1);
        }

        if current_date.month() == month {
            return Some(current_date);
        }
    }

    None
}

fn display_calendars(config: &Config, events: &Vec<Event>) {
    let months_per_row = if config.num_months == 1 {
        1
    } else {
        config.num_columns
    };

    if months_per_row == 0 {
        return;
    }

    let num_rows = (config.num_months + months_per_row - 1) / months_per_row;

    for row in 0..num_rows {
        let start_idx = row * months_per_row;
        let end_idx = std::cmp::min(start_idx + months_per_row, config.num_months);

        display_month_row(config, events, start_idx, end_idx);
        if row < num_rows - 1 {
            println!();
        }
    }
}

fn display_month_row(config: &Config, events: &Vec<Event>, start_idx: usize, end_idx: usize) {
    let mut dates = Vec::new();

    for idx in start_idx..end_idx {
        let months_offset = idx as i32;
        let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + months_offset as i64 - 1;
        let year = (total_months_from_epoch / 12) as i32;
        let month = (total_months_from_epoch % 12 + 1) as u32;

        dates.push(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
    }

    // Calculate dynamic width based on whether week numbers are shown
    // With weeks: 24 chars ("Wk " + 21 chars for days)
    // Without weeks: 21 chars (Just days)
    let calendar_width = if config.show_week_numbers { 24 } else { 21 };

    // Print month headers (centered over calculated width)
    for (idx, date) in dates.iter().enumerate() {
        let month_name_str = format!("{} {}", month_name(date.month()), date.year());
        let padding = (calendar_width - month_name_str.len()) / 2;
        print!("{}\x1b[1m{}\x1b[0m", " ".repeat(padding), month_name_str);
        // Ensure the padding is correct to match calendar_width exactly
        let r_padding = calendar_width - padding - month_name_str.len();
        print!("{}", " ".repeat(r_padding));
        
        if idx < dates.len() - 1 {
            print!("    ");
        }
    }
    println!();

    // Print weekday headers
    for idx in 0..dates.len() {
        print_weekday_header(config);
        if idx < dates.len() - 1 {
            print!("     ");
        }
    }
    println!();

    // Print calendar days
    let max_weeks = dates.iter().map(|d| weeks_in_month(*d, config.monday_first)).max().unwrap_or(6);

    for week in 0..max_weeks {
        // Check if the current week row across all months is entirely empty
        let is_empty_row = dates.iter().all(|&date| {
            let week_start_day = get_week_start_day(date, week, config.monday_first);
            let days_in_month = days_in_month(date.year(), date.month());
            week_start_day > days_in_month as i32 || week_start_day + 6 < 1
        });

        if !is_empty_row {
            for (idx, date) in dates.iter().enumerate() {
                print_week_row(*date, week, config, events);
                if idx < dates.len() - 1 {
                    print!("    ");
                }
            }
            println!();
        }
    }
    if max_weeks < 6 {
        println!();
    }
}

fn get_week_start_day(month_start: NaiveDate, week_num: usize, monday_first: bool) -> i32 {
    let first_weekday = month_start.weekday();
    let offset = if monday_first {
        first_weekday.num_days_from_monday()
    } else {
        first_weekday.num_days_from_sunday()
    };
    let start_day_offset = (week_num * 7) as i32;
    start_day_offset - offset as i32 + 1
}

fn print_weekday_header(config: &Config) {
    if config.show_week_numbers {
        if config.monday_first {
            print!("\x1b[34mWk\x1b[0m Mo Tu We Th Fr \x1b[31mSa Su\x1b[0m");
        } else {
            print!("\x1b[34mWk\x1b[0m \x1b[31mSu\x1b[0m Mo Tu We Th Fr \x1b[31mSa\x1b[0m");
        }
    } else {
        if config.monday_first {
            print!("Mo Tu We Th Fr \x1b[31mSa Su\x1b[0m");
        } else {
            print!("\x1b[31mSu\x1b[0m Mo Tu We Th Fr \x1b[31mSa\x1b[0m");
        }
    }
}

fn print_week_row(month_start: NaiveDate, week_num: usize, config: &Config, events: &Vec<Event>) {
    let days_in_month = days_in_month(month_start.year(), month_start.month());
    let start_day = get_week_start_day(month_start, week_num, config.monday_first);
    let today = chrono::Local::now().naive_local().date();

    // Only print week number column if enabled
    if config.show_week_numbers {
        let print_week_num = start_day <= days_in_month as i32 && start_day + 6 >= 1;
        if print_week_num {
            let week_date = month_start + Duration::days((start_day - 1) as i64).max(Duration::days(0));
            let iso_week = week_date.iso_week().week();
            print!("\x1b[34m{:2}\x1b[0m ", iso_week);
        } else {
            print!("   "); // Empty space for week number column
        }
    }

    for day_offset in 0..7 {
        let day = start_day + day_offset;

        if day > 0 && day <= days_in_month as i32 {
            let current_date = NaiveDate::from_ymd_opt(
                month_start.year(),
                month_start.month(),
                day as u32,
            ).unwrap();

            let event_for_day = events.iter().find(|e| e.date == current_date);
            let is_today = current_date == today;
            let chrono_weekday = current_date.weekday();
            let is_weekend = chrono_weekday == Weekday::Sat || chrono_weekday == Weekday::Sun;

            let (fg_code, bg_code, has_custom_color) = if let Some(event) = event_for_day {
                let fg = event.fg_color.as_ref().and_then(|c| get_ansi_color_code(c, true)).unwrap_or("");
                let bg = event.bg_color.as_ref().and_then(|c| get_ansi_color_code(c, false)).unwrap_or("");
                (fg, bg, !fg.is_empty() || !bg.is_empty())
            } else {
                ("", "", false)
            };

            const BOLD_CODE: &str = "\x1b[1m";
            const RESET_CODE: &str = "\x1b[0m";

            let mut format_codes = String::new();

            if is_weekend {
                format_codes.push_str("\x1b[31m");
                if event_for_day.is_some() {
                    format_codes.push_str(BOLD_CODE);
                }
            }

            if event_for_day.is_some() && !is_weekend {
                if has_custom_color {
                    format_codes.push_str(bg_code);
                    format_codes.push_str(fg_code);
                    format_codes.push_str(BOLD_CODE);
                } else {
                    if bg_code.is_empty() {
                         format_codes.push_str("\x1b[7m");
                    }
                }
            }

            if is_today {
                format_codes.clear();
                let final_bg = if bg_code.is_empty() { "\x1b[43m" } else { bg_code };
                let final_fg = if fg_code.is_empty() { "\x1b[30m" } else { fg_code };
                format_codes.push_str(final_bg);
                format_codes.push_str(final_fg);
            }

            print!("{}{:2}{} ", format_codes, day, RESET_CODE)

        } else {
            print!("   ");
        }
    }
}

/// Returns the correct ordinal suffix (st, nd, rd, th) for a number.
fn get_ordinal_suffix(n: i32) -> &'static str {
    if n % 100 >= 11 && n % 100 <= 13 {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}


fn display_events_list(config: &Config, events: &Vec<Event>) {
    let today = chrono::Local::now().naive_local().date();

    let start_date = NaiveDate::from_ymd_opt(
        config.start_year,
        config.start_month,
        1,
    ).unwrap();

    let total_months_from_epoch = config.start_year as i64 * 12 + config.start_month as i64 + config.num_months as i64;
    let end_year = ((total_months_from_epoch - 1) / 12) as i32;
    let end_month = ((total_months_from_epoch - 1) % 12 + 1) as u32;

    let end_date = NaiveDate::from_ymd_opt(end_year, end_month, 1).unwrap();

    let filtered_events: Vec<&Event> = events
        .iter()
        .filter(|e| e.date >= start_date && e.date < end_date)
        .collect();

    if filtered_events.is_empty() {
        return;
    }


    const BOLD_CODE: &str = "\x1b[1m";
    const RESET_CODE: &str = "\x1b[0m";

    println!("\n{}Events:{}",BOLD_CODE, RESET_CODE);
    println!("{}", "-".repeat(80));


    for event in filtered_events {
        let mut prefix_code = String::new();

        let fg_code = event.fg_color.as_ref().and_then(|c| get_ansi_color_code(c, true)).unwrap_or("");
        let bg_code = event.bg_color.as_ref().and_then(|c| get_ansi_color_code(c, false)).unwrap_or("");
        prefix_code.push_str(bg_code);
        prefix_code.push_str(fg_code);

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
                        let suffix = get_ordinal_suffix(anniversary_num);
                        let calculated_suffix = format!(" ({}{suffix} {label})", anniversary_num);
                        full_description.push_str(&calculated_suffix);
                    }
                }
            }
        }

        let days_diff = event.date.signed_duration_since(today).num_days();

        let relative_days_label = if days_diff == 0 {
            String::new()
        } else if days_diff > 0 {
            format!(" \x1b[32m(In {}{}{}\x1b[32m days){}", BOLD_CODE, days_diff, RESET_CODE, RESET_CODE)
        } else {
            format!(" \x1b[34m({}{}{}\x1b[34m days ago){}", BOLD_CODE, days_diff.abs(), RESET_CODE, RESET_CODE)
        };

        full_description.push_str(&relative_days_label);

        println!("{}{}{} - {}",
            prefix_code,
            event.date.format("%a, %d %b %Y"),
            RESET_CODE,
            full_description
        );
    }
}

// Maps common color names to ANSI escape codes
fn get_ansi_color_code(color_name: &str, is_fg: bool) -> Option<&'static str> {
    match color_name.to_lowercase().as_str() {
        "black"   => Some(if is_fg { "\x1b[30m" } else { "\x1b[40m" }),
        "red"     => Some(if is_fg { "\x1b[31m" } else { "\x1b[41m" }),
        "green"   => Some(if is_fg { "\x1b[32m" } else { "\x1b[42m" }),
        "yellow"  => Some(if is_fg { "\x1b[33m" } else { "\x1b[43m" }),
        "blue"    => Some(if is_fg { "\x1b[34m" } else { "\x1b[44m" }),
        "magenta" => Some(if is_fg { "\x1b[35m" } else { "\x1b[45m" }),
        "cyan"    => Some(if is_fg { "\x1b[36m" } else { "\x1b[46m" }),
        "white"   => Some(if is_fg { "\x1b[37m" } else { "\x1b[47m" }),
        _           => None,
    }
}


fn month_name(month: u32) -> &'static str {
    match month {
        1  => "January",
        2  => "February",
        3  => "March",
        4  => "April",
        5  => "May",
        6  => "June",
        7  => "July",
        8  => "August",
        9  => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _  => "Unknown",
    }
}

// Calculates the number of days in a given month/year
fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

// Calculates the total number of weeks needed to display a month
fn weeks_in_month(month_start: NaiveDate, monday_first: bool) -> usize {
    let first_weekday = month_start.weekday();
    let offset = if monday_first {
        first_weekday.num_days_from_monday()
    } else {
        first_weekday.num_days_from_sunday()
    };

    let days = days_in_month(month_start.year(), month_start.month());
    ((offset + days + 6) / 7) as usize
}
