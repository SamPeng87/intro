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
use std::cmp::Eq;
use std::hash::Hash;
use std::hash::Hasher;

const DEFAULT_FORMAT_STRING: &'static str = "%{level}:\t%{modulePath}\t%{message}";


trait Channeled: Send + Sync {
    fn send(&self, strings: String);
    fn sync_send(&self, strings: String);
}

#[derive(Copy, Eq, Debug)]
struct LogDirective {
    name: Option<&'static str>,
    level: LogLevelFilter,
}


impl Clone for LogDirective {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for LogDirective {
    #[inline]
    fn eq(&self, other: &LogDirective) -> bool {
        self.name == other.name && self.level == other.level
    }
}

struct LogExecute {
    //控制输出条件的不同格式化
    formatter: HashMap<Option<i32>, format::Formater>,
    //控制过滤条件到指定的channel
    outputs: HashMap<Option<i32>, Vec<Box<Channeled>>>,
    //控制是否输出
    directive: Vec<LogDirective>,
    sync: bool,
}

impl LogExecute {
    fn control(&self, record: &LogRecord) {
        if self.decision_directive(record) {
            let formatter = self.decision_formatter(record);
            let outputs = self.decision_outouts(record);
            for output in outputs {
                if self.sync {
                    output.sync_send(formatter.parse(|part| -> String{
                        parse(part, record)
                    }));
                } else {
                    output.send(formatter.parse(|part| -> String{
                        parse(part, record)
                    }));
                }
            }
        }
    }

    fn decision_outouts(&self, record: &LogRecord) -> &Vec<Box<Channeled>> {
        let level = Some(record.level() as i32);
        match self.outputs.get(&level) {
            Some(output) => {
                &output
            },
            None => {
                &self.outputs.get(&None).expect("no have any default outputs for this log execute")
            }
        }
    }

    fn decision_formatter(&self, record: &LogRecord) -> &format::Formater {
        let level = Some(record.level() as i32);
        match self.formatter.get(&level) {
            Some(formatter) => {
                &formatter
            },
            None => {
                &self.formatter.get(&None).expect("no have any default formatter for this log execute")
            }
        }
    }

    fn decision_directive(&self, record: &LogRecord) -> bool {
        let level = record.level();
        let target = record.target();
        for dir in self.directive.iter().rev() {
            match dir.name {
                Some(..) | None => {
                    return level <= dir.level
                }
            }
        }
        false
    }
}

struct LogExecuteBuilder {
    formatter: HashMap<Option<i32>, format::Formater>,
    outputs: HashMap<Option<i32>, Vec<Box<Channeled>>>,
    directive: Vec<LogDirective>,
    sync: bool,
}

impl LogExecuteBuilder {
    pub fn new() -> LogExecuteBuilder {
        LogExecuteBuilder {
            sync: false,
            formatter: HashMap::new(),
            outputs: HashMap::new(),
            directive: vec!(),
        }
    }
    pub fn set_formatter(&mut self, level: LogLevelFilter, formatter: format::Formater) -> &mut Self {
        self.formatter.insert(Some(level.to_log_level().unwrap() as i32), formatter);
        self
    }

    pub fn set_default_formatter(&mut self, formatter: format::Formater) -> &mut Self {
        self.formatter.insert(None, formatter);
        self
    }

    pub fn add_log_directive(&mut self, name: Option<&'static str>, level: LogLevelFilter) -> &mut Self {
        self.directive.push(LogDirective {
            name: name,
            level: level,
        });
        self
    }

    pub fn add_output(&mut self, level: LogLevelFilter, channeled: Box<Channeled>) -> &mut Self {
        self.outputs.entry(Some(level.to_log_level().unwrap() as i32)).or_insert(Vec::new()).push(channeled);
        self
    }

    pub fn set_sync(&mut self, sync: bool) -> &mut Self {
        self.sync = sync;
        self
    }

    pub fn build(&mut self) -> LogExecute {
        LogExecute {
            formatter: mem::replace(&mut self.formatter, HashMap::new()),
            directive: mem::replace(&mut self.directive, vec!()),
            outputs: mem::replace(&mut self.outputs, HashMap::new()),
            sync: self.sync,
        }
    }
}


#[allow(dead_code)]
struct Logger {
    formatter: HashMap<Option<&'static str>, HashMap<Option<&'static str>, LogExecute>>,
}

impl Logger {
    fn control(&self, record: &LogRecord) {
        let target = Some(record.target());
        let module = Some(record.location().module_path());

        let execute = match self.formatter.get(&module) {
            Some(executes) => {
                match executes.get(&target) {
                    Some(execute) => {
                        execute
                    }
                    None => {
                        executes.get(&None).unwrap()
                    }
                }
            },
            None => {
                self.formatter.get(&None).unwrap().get(&None).unwrap()
            }
        };
        execute.control(record);
    }
}


impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &LogRecord) {
        self.control(record);

        //        match self.target_formater.get(name) {
        //            Some(formater) => {
        //                self.write(&formater, record);
        //            }
        //            None => {
        //                match self.module_formater.get(name) {
        //                    Some(formater) => {
        //                        self.write(&formater, record);
        //                    }
        //                    None => {
        //                        self.write(&self.root_formater, record);
        //                    }
        //                }
        //            }
        //        }
    }
}

impl Logger {
    #[inline]
    fn write(&self, formater: &format::Formater, record: &LogRecord) {
        if record.location().module_path() == "intro" {
            //            self.outputs[0].send(formater.parse(|part| -> String{
            //                parse(part, record)
            //            }));
        }
    }
}

#[allow(dead_code)]
struct LoggerBuilder {
    formatter: HashMap<Option<&'static str>, HashMap<Option<&'static str>, LogExecute>>,
}


#[allow(dead_code)]
impl LoggerBuilder {
    fn new() -> Self {
        LoggerBuilder {
            formatter: HashMap::new()
        }
    }

    #[inline]
    fn add_default_module_of_target_formatter(&mut self, target: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.__add_formatter(None, Some(target), builder);
        self
    }

    #[inline]
    fn add_default_target_of_module_formatter(&mut self, module: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.__add_formatter(Some(module), None, builder);
        self
    }

    #[inline]
    fn add_formatter(&mut self, module: &'static str, target: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.__add_formatter(Some(module), Some(target), builder);
        self
    }

    #[inline]
    fn add_default_formater(&mut self, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.__add_formatter(None, None, builder);
        self
    }


    fn build(&mut self) -> Logger {
        Logger {
            formatter: mem::replace(&mut self.formatter, HashMap::new()),
        }
    }

    fn init_logger(&mut self) -> Result<(), SetLoggerError> {
        log::set_logger(|max_level| {
            max_level.set(LogLevelFilter::Trace);
            Box::new(self.build())
        })
    }

    fn __add_formatter(&mut self, module: Option<&'static str>, target: Option<&'static str>, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.formatter.entry(module).or_insert(HashMap::new()).insert(target, builder.build());
        self
    }
}

//pub fn init() -> Result<(), SetLoggerError> {
//    log::set_logger(|max_level| {
//        let logger = LoggerBuilder::new().
//            add_target_formater("test", "%{message}").
//            builder();
//
//        max_level.set(LogLevelFilter::Trace);
//        Box::new(logger)
//    })
//}

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
fn format_log_execute_build() {
    let o = Arc::new(Std::new(Direction::STDOUT));
    let execute = LogExecuteBuilder::new()
        .add_log_directive(None, LogLevelFilter::Info)
        .add_log_directive(None, LogLevelFilter::Debug)
        .add_output(LogLevelFilter::Info, Box::new(StdChannel::new(event::EventPool::new(o.clone()), o.clone())))
        .add_output(LogLevelFilter::Debug, Box::new(StdChannel::new(event::EventPool::new(o.clone()), o.clone())))
        .set_formatter(LogLevelFilter::Info, format::Formater::new(DEFAULT_FORMAT_STRING))
        .set_sync(false)
        .build();
    //    assert_eq!(execute.directive.len(), 1);
    //
    //    let ref dir = execute.directive[0];
    //    assert_eq!(dir.name, None);
    //    assert_eq!(dir.level, LogLevelFilter::Info);
    //    assert_eq!(execute.sync, false);
}


#[test]
fn format_parse() {
    use std::thread;
    use std::time::{SystemTime};
    let o = Arc::new(Std::new(Direction::STDOUT));
    let mut execute = LogExecuteBuilder::new();

    execute
        .add_log_directive(None, LogLevelFilter::Info)
        .add_output(LogLevelFilter::Info, Box::new(StdChannel::new(event::EventPool::new(o.clone()), o.clone())))
        .add_output(LogLevelFilter::Debug, Box::new(StdChannel::new(event::EventPool::new(o.clone()), o.clone())))
        .set_formatter(LogLevelFilter::Info, format::Formater::new(DEFAULT_FORMAT_STRING))
        .set_sync(false);

    LoggerBuilder::new()
        .add_default_formater(&mut execute)
        .init_logger();

    let now = SystemTime::now();
    info!("{}", "test o");
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



    //    use std::thread;
    //    use std::time::{SystemTime};
    //    let _ = init();
    //    let mut children = vec![];
    //
    //    let now = SystemTime::now();
    //    for i in 0..5 {
    //        children.push(thread::spawn(move || {
    //            for j in 0..1000 {
    //                info!("{} {}", i, j);
    //            }
    //        }));
    //    }
    //    for child in children {
    //        child.join().unwrap()
    //    }
    //    match now.elapsed() {
    //        Ok(elapsed) => {
    //            // it prints '2'
    //            println!("time is {}", elapsed.subsec_nanos());
    //        }
    //        Err(e) => {
    //            // an error occured!
    //            println!("Error: {:?}", e);
    //        }
    //    }
}
