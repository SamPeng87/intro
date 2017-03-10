use mio::{channel, Events, Poll, PollOpt, Token, Ready};

use std::time::Duration;
use std::thread;
use std::marker::Send;
use output::ReceiverData;
use output::Output;
use std::sync::{Arc, Mutex};


pub struct EventPool<R>
{
    tx: Arc<Mutex<channel::Sender<R>>>,
}

impl<R> EventPool<R>
{
    pub fn new<T>(o: T) -> EventPool<R>
        where R: ReceiverData + 'static, T: Output<R>
    {
        let mut events = Events::with_capacity(10);
        let poll = Poll::new().unwrap();
        let (tx, rx): (channel::Sender<R>, channel::Receiver<R>) = channel::channel();


        assert_eq!(0, events.len());

        poll.register(&rx, Token(0), Ready::all(), PollOpt::edge()).unwrap();
        let out = Arc::new(o);
        let event_pool = EventPool {
            tx: Arc::new(Mutex::new(tx)),
        };

        thread::spawn(move || {
            loop {
                let num = poll.poll(&mut events, None).unwrap();
                for event in &events {
                    match event.token() {
                        Token(0) => {
                            loop {
                                match rx.try_recv() {
                                    Ok(v) => {
                                        out.push(v);
                                    },
                                    Err(err) => {
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

        event_pool
    }

    pub fn send(&self, data: R) {
        let tx_arc = self.tx.clone();
        let tx = tx_arc.lock().unwrap().clone();

        tx.send(data).unwrap();
    }
}


#[test]
fn test_send_event() {
    use output::stdout::Direction;
    use output::stdout::Std;
    use output::stdout::StdData;

    let a = Std;
    let b = EventPool::new(a);
    b.send(StdData {
        direction: Direction::STDERR,
        string: "test321".to_string()
    });

    b.send(StdData {
        direction: Direction::STDOUT,
        string: "test321".to_string()
    });
    thread::sleep_ms(1000)
}

