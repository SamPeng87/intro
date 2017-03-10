use mio::channel::Receiver;
use super::ReceiverData;
use super::Output;
use std::io;
use std::io::Write;

pub enum Direction {
    STDOUT,
    STDERR,
}

pub struct Std;

pub struct StdData {
    pub string: String,
    pub direction: Direction
}


impl ReceiverData for StdData {
    type Type = Direction;
    fn get_direction(&self) -> &Direction {
        &self.direction
    }
    fn get_string(&self) -> &str {
        &self.string
    }
}


impl Output<StdData> for Std{
    fn push(&self, out: StdData){
        match out.get_direction(){
            &Direction::STDOUT =>{
                println!("push stdout");
                writeln!(&mut io::stdout(), "{}", out.get_string());
            },
            &Direction::STDERR=>{
                println!("push stderr");
                writeln!(&mut io::stderr(), "{}", out.get_string());
            }

        }
    }
}

#[test]
fn output_stdout() {
    let a = Std {};

    a.push(StdData{
        direction: Direction::STDOUT,
        string : "test123".to_string()
    });


    a.push(StdData{
        direction: Direction::STDERR,
        string : "test321".to_string()
    });

}
