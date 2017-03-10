use regex::Regex;
use std::str;
use std::string::String;
use std::marker::Sized;

pub struct Part {
    name: String,
    layout: Option<String>,
}

impl Part {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn layout(&self) -> &Option<String> {
        &(self.layout)
    }
}

pub struct Formater {
    parts: Vec<Part>,
}

pub trait FormaterParseCase: Sized {
    type ParamType: ? Sized;

    fn parse(&self, part: &Part, args: &Self::ParamType) -> String;
}


impl Formater {
    pub fn new(layout: &str) -> Formater {
        let parts = Formater::parse_parts(layout);
        Formater {
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

impl Formater {
    pub fn parse<F>(&self, parse_factory: F) -> String
        where F: Fn(&Part) -> String
    {
        let mut res = String::with_capacity(1000);
        for part in &self.parts {
            res += &parse_factory(part)
        }
        res
    }
}

