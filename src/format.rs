use regex::Regex;
use std::str;
use std::string::String;
use super::Formatter;
use super::Parted;
use super::LogEntry;
use chrono::prelude::*;
use chrono::{NaiveDateTime,TimeZone, NaiveDate,Local};
use time;
use time::{Timespec,Tm};



pub struct Part {
    name: String,
    layout: Option<String>,
}

impl Parted for Part {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn layout(&self) -> &Option<String> {
        &(self.layout)
    }
}

pub struct StringFormatter {
    parts: Vec<Part>,
}

impl StringFormatter {
    #[inline]
    pub fn new(layout: &str) -> StringFormatter {
        let parts = StringFormatter::parse_parts(layout);
        StringFormatter {
            parts: parts,
        }
    }

    #[inline]
    fn parse_parts(layout: &str) -> Vec<Part> {
        let regex = Regex::new(r"%\{([a-zA-Z]+)(?::(.*?[^\\]))?\}").unwrap();

        let mut parts = Vec::new();

        let mut prev = 0usize;

        let iter = regex.find_iter(&layout);

        for pos in iter {
            let (start, end) = (pos.start(), pos.end());
            if start > prev {
                //prev have string
                let substrings = layout[prev..start].to_string().clone();

                parts.push(Part {
                    name: "string".to_string(),
                    layout: Some(substrings)
                });
            };

            let substrings = layout[start + 2..end - 1].to_string();
            let mut split: Vec<String> = substrings.split(":").map(String::from).collect();
            let name = split[0].clone();
            let layout = match split.len() {
                2 => Some(split[1].clone()),
                0|1 => None,
                _ => {
                    split.remove(0);
                    Some(split.connect(":"))
                },
            };
            print!("{:?}", split.len());
            print!("{:?}", layout);


            parts.push(Part {
                name: name,
                layout: layout,
            });


            prev = end;
        };

        if prev < layout.len() {
            let substrings = layout[prev..layout.len()].to_string().clone();
            parts.push(Part {
                name: "string".to_string(),
                layout: Some(substrings)
            })
        }
        return parts;
    }
}


impl Formatter for StringFormatter {
    #[inline]
    fn parse(&self, record: &LogEntry) -> String
    {
        let mut res = String::with_capacity(100);
        for part in &self.parts {
            res += &parse(part, record);
        }
        res
    }
}

#[inline]
fn parse(part: &Part, args: &LogEntry) -> String {

    match part.name() {
        "string" => {
            match part.layout() {
                &Some(ref layout) =>
                    return layout.clone(),
                _ =>
                    return "".to_string(),
            };
        }
        "datetime" => {
            let now = get_record_date_time(args.time);
            match part.layout() {
                &Some(ref layout) =>
                    match layout.as_str() {
                        "rfc2822" =>
                            return now.to_rfc2822(),
                        "rfc3339" =>
                            return now.to_rfc3339(),
                        _ =>
                            return now.format(layout).to_string(),
                    },
                _ =>
                    return now.to_string(),
            };
        }
        "line" => {
            return format!("{}", args.location().line());
        }
        "level" => {
            return format!("{}", args.level());
        }

        "file" => {
            return format!("{}", args.location().file());
        }
        "modulePath" => {
            return format!("{}", args.location().module_path());
        }
        "message" => {
            return format!("{}", args.args());
        }

        _ => {
            return format!("{}", "");
        }
    }
}

#[inline]
fn get_record_date_time(ts: Timespec) -> DateTime<Local>{
    let mut tm = time::at(ts);

    if tm.tm_sec >= 60 {
        tm.tm_nsec += (tm.tm_sec - 59) * 1_000_000_000;
        tm.tm_sec = 59;
    }

    #[cfg(not(windows))]
    fn tm_to_naive_date(tm: &time::Tm) -> NaiveDate {
        // from_yo is more efficient than from_ymd (since it's the internal representation).
        NaiveDate::from_yo(tm.tm_year + 1900, tm.tm_yday as u32 + 1)
    }

    #[cfg(windows)]
    fn tm_to_naive_date(tm: &oldtime::Tm) -> NaiveDate {
        // ...but tm_yday is broken in Windows (issue #85)
        NaiveDate::from_ymd(tm.tm_year + 1900, tm.tm_mon as u32 + 1, tm.tm_mday as u32)
    }

    let date = tm_to_naive_date(&tm);
    let time = NaiveTime::from_hms_nano(tm.tm_hour as u32, tm.tm_min as u32,
                                        tm.tm_sec as u32, tm.tm_nsec as u32);
    let offset = FixedOffset::east(tm.tm_utcoff);
    DateTime::from_utc(date.and_time(time) - offset, offset)

}
