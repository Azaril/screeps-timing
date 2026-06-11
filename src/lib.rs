use serde::*;
use std::cell::RefCell;

pub type IdentStr = &'static str;

thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

pub fn start_trace(clock: Box<dyn Fn() -> u64>) {
    TRACE.with(|t| {
        let mut trace = t.borrow_mut();
        if trace.is_some() {
            panic!("Expected trace to not be set!");
        }
        *trace = Some(Trace::new(clock));
    });
}

pub fn stop_trace() -> Trace {
    TRACE.with(|t| t.borrow_mut().take().unwrap())
}

pub fn with_trace<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Trace) -> R,
{
    TRACE.with(|t| t.borrow_mut().as_mut().map(f))
}

#[derive(Serialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    events: Vec<Event>,
    #[serde(skip_serializing)]
    clock: Box<dyn Fn() -> u64>,
}

impl Trace {
    fn new(clock: Box<dyn Fn() -> u64>) -> Trace {
        Trace {
            clock,
            events: Vec::new(),
        }
    }

    pub fn get_time(&self) -> u64 {
        (self.clock)()
    }
}

#[derive(Serialize)]
struct TracingEvent {
    #[serde(rename = "name")]
    name: IdentStr,
    #[serde(rename = "pid")]
    process_id: u32,
    #[serde(rename = "tid")]
    thread_id: u32,
    #[serde(rename = "ts")]
    timestamp: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct BeginEvent {
    name: IdentStr,
    time: u64,
}

impl From<BeginEvent> for TracingEvent {
    fn from(event: BeginEvent) -> TracingEvent {
        TracingEvent {
            name: event.name,
            process_id: 0,
            thread_id: 0,
            timestamp: event.time,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct EndEvent {
    name: IdentStr,
    time: u64,
}

impl From<EndEvent> for TracingEvent {
    fn from(event: EndEvent) -> TracingEvent {
        TracingEvent {
            name: event.name,
            process_id: 0,
            thread_id: 0,
            timestamp: event.time,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "ph")]
enum Event {
    #[serde(rename = "B")]
    Begin(BeginEvent),
    #[serde(rename = "E")]
    End(EndEvent),
}

#[must_use = "The guard is immediately dropped after instantiation. This is probably not
what you want! Consider using a `let` binding to increase its lifetime."]
pub struct SpanGuard {
    name: IdentStr,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        end(self.name);
    }
}

pub fn start_guard<S: Into<IdentStr>>(name: S) -> SpanGuard {
    let name = name.into();
    start(name);
    SpanGuard { name }
}

fn start<S: Into<IdentStr>>(name: S) {
    with_trace(|trace| {
        let event = BeginEvent {
            name: name.into(),
            time: trace.get_time(),
        };

        trace.events.push(Event::Begin(event));
    });
}

fn end<S: Into<IdentStr>>(name: S) {
    let name = name.into();

    with_trace(|trace| {
        let event = EndEvent {
            name,
            time: trace.get_time(),
        };

        trace.events.push(Event::End(event));
    });
}
