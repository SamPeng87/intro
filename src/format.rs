use regex::Regex;
use std::str;
use std::string::String;
use std::marker::Sized;
use super::Formatter;
use super::Parted;
use super::LogEntry;


pub struct Part {
    name: String,
    layout: Option<String>,
}

impl Parted for Part {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self) -> &Option<String> {
        &(self.layout)
    }
}

pub struct StringFormatter {
    parts: Vec<Part>,
}

impl StringFormatter {
    pub fn new(layout: &str) -> StringFormatter {
        let parts = StringFormatter::parse_parts(layout);
        StringFormatter {
            parts: parts,
        }
    }

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
            let split: Vec<String> = substrings.split(":").map(String::from).collect();
            let name = split[0].clone();
            let layout = match split.len() {
                2 => Some(split[1].clone()),
                _ => None,
            };

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
