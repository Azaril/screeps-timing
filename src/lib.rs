use serde::*;

pub type IdentStr = &'static str;

static mut TRACE: Option<Trace> = None;

pub fn start_trace(clock: Box<dyn Fn() -> u64>) {
    unsafe {
        if TRACE.is_some() {
            panic!("Expected trace to be not be set!");
        }

        TRACE = Some(Trace::new(clock));
    }
}

pub fn stop_trace() -> Trace {
    unsafe {
        TRACE.take().unwrap()
    }
}

pub fn get_mut_trace() -> Option<&'static mut Trace> {
    unsafe {
        TRACE.as_mut()
    }
}

#[derive(Serialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    events: Vec<Event>,
    #[serde(skip_serializing)]
    clock: Box<dyn Fn() -> u64>
}

impl Trace {
    fn new(clock: Box<dyn Fn() -> u64>) -> Trace {
        Trace {
            clock,
            events: Vec::new()
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
    timestamp: u64
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct BeginEvent {
    name: IdentStr,
    time: u64,
}

impl Into<TracingEvent> for BeginEvent {
    fn into(self) -> TracingEvent {
        TracingEvent {
            name: self.name,
            process_id: 0,
            thread_id: 0,
            timestamp: self.time
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct EndEvent {
    name: IdentStr,
    time: u64
}

impl Into<TracingEvent> for EndEvent {
    fn into(self) -> TracingEvent {
        TracingEvent {
            name: self.name,
            process_id: 0,
            thread_id: 0,
            timestamp: self.time
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "ph")]
enum Event {
    #[serde(rename = "B")]
    Begin(BeginEvent),
    #[serde(rename = "E")]
    End(EndEvent)
}

#[must_use = "The guard is immediately dropped after instantiation. This is probably not
what you want! Consider using a `let` binding to increase its lifetime."]
pub struct SpanGuard {
    name: IdentStr
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
    if let Some(trace) = get_mut_trace() {
        let event = BeginEvent {
            name: name.into(),
            time: trace.get_time()
        };

        trace.events.push(Event::Begin(event));
    }
}

fn end<S: Into<IdentStr>>(name: S) {
    let name = name.into();

    if let Some(trace) = get_mut_trace() {
        let event = EndEvent {
            name,
            time: trace.get_time()
        };

        trace.events.push(Event::End(event));
    }
}