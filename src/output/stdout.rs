use ReceiverData;
use Output;
use LogEntry;
use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex};
use mio::channel;
use Channeled;
use std::thread;
use format;
use Formatter;

#[derive(Clone)]
pub enum Direction {
    STDOUT,
    STDERR,
}

pub struct Std {
    direction: Direction,
    formatter: format::StringFormatter
}

pub struct StdChannel {
    tx: Arc<Mutex<channel::Sender<Arc<LogEntry>>>>,
    out: Arc<Std>,
}

impl StdChannel {
    pub fn new(tx: channel::Sender<Arc<LogEntry>>, out: Arc<Std>) -> Self {
        StdChannel {
            tx: Arc::new(Mutex::new(tx)),
            out: out.clone(),
        }
    }
}

impl Channeled for StdChannel {
    #[inline]
    fn send(&self, strings: Arc<LogEntry>) {
        let tx_arc = self.tx.clone();
        let tx = tx_arc.lock().unwrap().clone();
        tx.send(strings).unwrap();
    }

    #[inline]
    fn sync_send(&self, strings: Arc<LogEntry>) {
        self.out.clone().push(strings);
    }
}

impl Std {
    pub fn new(dir: Direction, formatter: format::StringFormatter) -> Self {
        Std {
            direction: dir,
            formatter: formatter
        }
    }
}

impl Output for Std {
    fn push(&self, out: Arc<LogEntry>)
    {
        let out_string = self.formatter.parse(&out.clone());


        match self.direction {
            Direction::STDOUT => {
                let _ = writeln!(&mut io::stdout(), "{}", out_string);
            },
            Direction::STDERR => {
                let _ = writeln!(&mut io::stderr(), "{}", out_string);
            }
        }
    }
}

#[test]
fn output_stdout() {
    //    let a = Std {};
    //
    //    a.push(StdData{
    //        direction: Direction::STDOUT,
    //        string : "test123".to_string()
    //    });
    //
    //
    //    a.push(StdData{
    //        direction: Direction::STDERR,
    //        string : "test321".to_string()
    //    });
}
