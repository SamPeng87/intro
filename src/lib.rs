#[macro_use]
extern crate log;

extern crate regex;
extern crate mio;


use log::{LogLevel, LogLevelFilter, SetLoggerError, LogMetadata, LogRecord};
use std::collections::HashMap;
use std::mem;


mod format;
mod output;
mod event;

use output::stdout::*;
use output::*;
use std::sync::{Arc, Mutex};
use mio::channel;

const DEFAULT_FORMAT_STRING: &'static str = "%{level}:\t%{modulePath}\t%{message}";


trait Channeled: Send + Sync {
    fn send(&self, strings: String);
    fn sync_send(&self, strings: String);
}

struct StdChannel {
    tx: Arc<Mutex<channel::Sender<String>>>,
    out: Arc<Std>
}

impl Channeled for StdChannel {
    #[inline]
    fn send(&self, strings: String) {
        let tx_arc = self.tx.clone();
        let tx = tx_arc.lock().unwrap().clone();
        tx.send(strings).unwrap();
    }

    #[inline]
    fn sync_send(&self, strings: String) {
        self.out.clone().push(strings);
    }
}


#[allow(dead_code)]
struct Logger {
    root_formater: format::Formater,
    target_formater: HashMap<String, format::Formater>,
    module_formater: HashMap<String, format::Formater>,
    global_part: HashMap<String, String>,
    outputs: Vec<Box<Channeled>>
}


impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &LogRecord) {
        let name = record.metadata().target();
        match self.target_formater.get(name) {
            Some(formater) => {
                self.write(&formater, record);
            }
            None => {
                match self.module_formater.get(name) {
                    Some(formater) => {
                        self.write(&formater, record);
                    }
                    None => {
                        self.write(&self.root_formater, record);
                    }
                }
            }
        }
    }
}

impl Logger {
    #[inline]
    fn write(&self, formater: &format::Formater, record: &LogRecord) {
        if record.location().module_path() == "intro" {
            self.outputs[0].send(formater.parse(|part| -> String{
                parse(part, record)
            }));
        }
    }
}

#[allow(dead_code)]
struct LoggerBuilder {
    target_formater: HashMap<String, format::Formater>,
    module_formater: HashMap<String, format::Formater>,
    global_part: HashMap<String, String>,
    root_formater: format::Formater,
    outputs: Vec<Box<Channeled>>
}

#[allow(dead_code)]
impl LoggerBuilder {
    #[inline]
    fn new() -> LoggerBuilder {
        let o = Arc::new(Std::new(Direction::STDOUT));
        LoggerBuilder {
            target_formater: HashMap::new(),
            module_formater: HashMap::new(),
            global_part: HashMap::new(),
            root_formater: format::Formater::new(DEFAULT_FORMAT_STRING),
            outputs: vec![
                Box::new(StdChannel {
                    tx: Arc::new(Mutex::new(event::EventPool::new(o.clone()))),
                    out: o.clone()
                })
            ],
        }
    }
    #[inline]
    fn add_target_formater(&mut self, name: &str, format: &str) -> &mut Self {
        self.target_formater.insert(name.to_string(), format::Formater::new(format));
        self
    }
    #[inline]
    fn add_module_formater(&mut self, name: &str, format: &str) -> &mut Self {
        self.target_formater.insert(name.to_string(), format::Formater::new(format));
        self
    }
    #[inline]
    fn root_formater(&mut self, format: &str) -> &mut Self {
        self.root_formater = format::Formater::new(format);
        self
    }

    #[inline]
    fn global_part(&mut self, part: Box<HashMap<String, String>>) -> &mut Self {
        self.global_part = *part;
        self
    }

    fn builder(&mut self) -> Logger {
        Logger {
            target_formater: mem::replace(&mut self.target_formater, HashMap::new()),
            module_formater: mem::replace(&mut self.module_formater, HashMap::new()),
            global_part: mem::replace(&mut self.global_part, HashMap::new()),
            root_formater: mem::replace(&mut self.root_formater, format::Formater::new("")),
            outputs: mem::replace(&mut self.outputs, vec!()),
        }
    }
}


pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(|max_level| {
        let logger = LoggerBuilder::new().
            add_target_formater("test", "%{message}").
            builder();

        max_level.set(LogLevelFilter::Trace);
        Box::new(logger)
    })
}

#[inline]
fn parse(part: &format::Part, args: &LogRecord) -> String {
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


#[cfg(test)]
mod tests;

#[test]
fn format_parse() {
    use std::thread;
    use std::time::{SystemTime};
    let _ = init();
    let mut children = vec![];

    let now = SystemTime::now();
    for i in 0..5 {
        children.push(thread::spawn(move || {
            for j in 0..1000 {
                info!("{} {}", i, j);
            }
        }));
    }
    for child in children {
        child.join().unwrap()
    }
    match now.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!("time is {}", elapsed.subsec_nanos());
        }
        Err(e) => {
            // an error occured!
            println!("Error: {:?}", e);
        }
    }
}
