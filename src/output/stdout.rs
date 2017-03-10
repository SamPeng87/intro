use super::ReceiverData;
use super::Output;
use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex};
use mio::channel;
use Channeled;

#[derive(Clone)]
pub enum Direction {
    STDOUT,
    STDERR,
}

#[derive(Clone)]
pub struct Std {
    direction: Direction
}

pub struct StdChannel {
    tx: Arc<Mutex<channel::Sender<String>>>,
    out: Arc<Std>
}

impl StdChannel {
    pub fn new(tx: channel::Sender<String>, out: Arc<Std>) -> Self {
        StdChannel {
            tx: Arc::new(Mutex::new(tx)),
            out: out.clone()
        }
    }
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

impl Std {
    pub fn new(dir: Direction) -> Self {
        Std {
            direction: dir
        }
    }
}

impl Output for Std {
    fn push(&self, out: String)
    {
        match self.direction {
            Direction::STDOUT => {
                let _ = writeln!(&mut io::stdout(), "{}", out);
            },
            Direction::STDERR => {
                let _ = writeln!(&mut io::stderr(), "{}", out);
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
