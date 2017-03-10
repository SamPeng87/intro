use mio::{channel, Events, Poll, PollOpt, Token, Ready};


use std::thread;
use output::ReceiverData;
use output::Output;
use std::sync::{Arc, Mutex};


pub struct EventPool;

impl EventPool
{
    #[inline]
    pub fn new<T>(o: Arc<T>) -> channel::Sender<String>
        where T: Output
    {
        let mut events = Events::with_capacity(10);
        let poll = Poll::new().unwrap();
        let (tx, rx): (channel::Sender<String>, channel::Receiver<String>) = channel::channel();


        assert_eq!(0, events.len());

        poll.register(&rx, Token(0), Ready::all(), PollOpt::edge()).unwrap();

        thread::spawn(move || {
            loop {
                poll.poll(&mut events, None).unwrap();
                for event in &events {
                    match event.token() {
                        Token(0) => {
                            loop {
                                match rx.try_recv() {
                                    Ok(v) => {
                                        o.clone().push(v);
                                    },
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
        });

        tx
    }
}


#[test]
fn test_send_event() {
    use output::stdout::Direction;
    use output::stdout::Std;
    use std::time::Duration;
}

