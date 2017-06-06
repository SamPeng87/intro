//use log::LogLevel;
use log::{LogLevel, LogLevelFilter, LogLocation, SetLoggerError, LogMetadata, LogRecord};
use std::collections::HashMap;
use std::hash::Hash;
use std::cell::Cell;
use ansi_term::Colour;
use ansi_term::Style;
use std::io;


pub fn get_color_by_level(l: LogLevel, msg: &str) -> String {
    let res = match l {
        LogLevel::Error => Colour::Red.paint(msg),
        LogLevel::Trace => Colour::White.paint(msg),
        LogLevel::Debug => Colour::Green.paint(msg),
        LogLevel::Warn => Colour::Purple.paint(msg),
        _ => Style::default().paint(msg)
    };
    res.to_string()
}

