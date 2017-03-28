
use output::Output;
use std::sync::{Arc};
use LogEntry;
use Channeled;
use log::LogLevelFilter;
use Formatter;
use std::mem;
use std::collections::HashMap;
use futures_cpupool::CpuPool;
use futures_cpupool::Builder;

struct EventRouter {
    formatter: Arc<Formatter>,
    output: Vec<Arc<Output>>,
}

impl EventRouter {
    #[inline]
    fn gateway(&self, record: Arc<LogEntry>, pool: &CpuPool) {

        for output in &self.output {
            let formatter = self.formatter.clone();
            let o = output.clone();
            let record_arc = record.clone();
            let future = pool.spawn_fn(move || {
                let data = formatter.parse(&record_arc);
                o.push(&data);
                let res: Result<bool, ()> = Ok(true);
                res
            });
            future.forget();
        }
    }
}

pub struct EventRouterBuilder {
    formatter: Arc<Formatter>,
    output: Vec<Arc<Output>>,
}

impl EventRouterBuilder
{
    pub fn new(formater: Arc<Formatter>)
               -> Self
    {
        EventRouterBuilder {
            formatter: formater.clone(),
            output: vec!(),
        }
    }

    pub fn add<T: Output>(&mut self, output: Arc<T>) -> &mut Self {
        self.output.push(output);
        self
    }

    fn build(&mut self) -> EventRouter {
        EventRouter {
            formatter: self.formatter.clone(),
            output: self.output.clone(),
        }
    }
}

pub struct EventPool
{
    router: HashMap<Option<i32>, Vec<EventRouter>>,
    pool: CpuPool,
}

pub struct EventPoolBuilder {
    router: HashMap<Option<i32>, Vec<EventRouter>>,
    worker: usize,
}

impl EventPoolBuilder {
    #[inline]
    pub fn new(worker_num: usize) -> EventPoolBuilder {
        EventPoolBuilder {
            router: HashMap::new(),
            worker: worker_num
        }
    }

    pub fn add(&mut self, level: LogLevelFilter, router: &mut EventRouterBuilder) -> &mut Self {
        self.router.entry(Some(level.to_log_level().unwrap() as i32)).or_insert(Vec::new()).push(router.build());
        self
    }
    pub fn default(&mut self, router: &mut EventRouterBuilder) -> &mut Self {
        self.router.entry(None).or_insert(Vec::new()).push(router.build());
        self
    }

    #[inline]
    pub fn build(&mut self) -> EventPool {
        let mut tmp = Builder::new();
        let mut builder = tmp.name_prefix("intro-");
        let pool = if self.worker == 0 {
            builder.create()
        } else {
            builder.pool_size(self.worker).create()
        };
        EventPool {
            router: mem::replace(&mut self.router, HashMap::new()),
            pool: pool
        }
    }
}

impl Channeled for EventPool {
    #[inline]
    fn send(&self, data: Arc<LogEntry>) {
        let routers = match self.router.get(&Some(data.level() as i32)) {
            Some(router) => {
                router
            }
            None => {
                self.router.get(&None).expect("this channel not have default executer")
            }
        };
        for r in routers {
            r.gateway(data.clone(), &self.pool);
        }
    }
}


#[test]
fn test_send_event() {
}

