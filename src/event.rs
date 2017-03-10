use mio::{channel, Events, Poll, PollOpt, Token, Ready};


use std::thread;
use output::ReceiverData;
use output::Output;
use std::sync::{Arc, Mutex};


pub struct EventPool<T, R>
    where R: ReceiverData + 'static, T: Output<R>
{
    out: T,
    tx: Arc<Mutex<channel::Sender<R>>>,
}

impl<T,R> EventPool<T,R>
    where R: ReceiverData + 'static, T: Output<R>
{
    pub fn new(o: T) -> EventPool<T,R>
    {
        let mut events = Events::with_capacity(10);
        let poll = Poll::new().unwrap();
        let (tx, rx): (channel::Sender<R>, channel::Receiver<R>) = channel::channel();


        assert_eq!(0, events.len());

        poll.register(&rx, Token(0), Ready::all(), PollOpt::edge()).unwrap();
        let event_pool = EventPool {
            out: o.clone(),
            tx: Arc::new(Mutex::new(tx)),
        };
        let out = Arc::new(o);

        thread::spawn(move || {
            loop {
                poll.poll(&mut events, None).unwrap();
                for event in &events {
                    match event.token() {
                        Token(0) => {
                            loop {
                                match rx.try_recv() {
                                    Ok(v) => {
                                        out.clone().push(v);
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

        event_pool
    }

    pub fn send(&self, data: R) {
        let tx_arc = self.tx.clone();
        let tx = tx_arc.lock().unwrap().clone();

        tx.send(data).unwrap();
    }

    #[allow(dead_code)]
    pub fn sync_send(&self, data: R) {
        self.out.push(data);
    }
}


#[test]
fn test_send_event() {
    use output::stdout::Direction;
    use output::stdout::Std;
    use output::stdout::StdData;
    use std::time::Duration;

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
    thread::sleep(Duration::from_millis(1000))
}

