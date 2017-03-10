use super::ReceiverData;
use super::Output;
use std::io;
use std::io::Write;

#[derive(Clone)]
pub enum Direction {
    STDOUT,
    STDERR,
}

#[derive(Clone)]
pub struct Std {
    direction: Direction
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
