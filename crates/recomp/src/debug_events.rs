//! Common observation identities. This clock never dispatches an action or performs I/O.
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize)]
pub(crate) struct Correlation {
    pub request_id: Option<String>,
    pub job_id: Option<String>,
    pub source: &'static str,
}
impl Default for Correlation {
    fn default() -> Self { Self { request_id: None, job_id: None, source: "unattributed" } }
}
impl Correlation {
    pub fn request(id: &str, source: &'static str, job: bool) -> Self {
        // Control request IDs already have a 100-byte protocol limit. Keep the
        // envelope bounded even for other callers; never truncate into another ID.
        assert!(id.len() <= 100);
        Self { request_id: Some(id.into()), job_id: job.then(|| id.into()), source }
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct Session {
    pub run_id: String,
    pub manifest_sha256: Option<String>,
}
impl Default for Session {
    fn default() -> Self {
        Self { run_id: format!("host-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()), manifest_sha256: None }
    }
}
struct State { started: Instant, next: u64, requests: u64, session: Session }
#[derive(Clone)]
pub(crate) struct Clock {
    state: Arc<Mutex<State>>,
    context: Arc<Mutex<Correlation>>,
}
impl Default for Clock { fn default() -> Self { Self::new(Session::default()) } }
impl Clock {
    pub fn new(session: Session) -> Self {
        Self { state: Arc::new(Mutex::new(State { started: Instant::now(), next: 0, requests: 0, session })), context: Arc::new(Mutex::new(Correlation::default())) }
    }
    pub fn context(&self, value: Correlation) -> Context {
        let previous = std::mem::replace(&mut *self.context.lock().unwrap(), value);
        Context { context: self.context.clone(), previous }
    }
    pub fn current(&self) -> Correlation { self.context.lock().unwrap().clone() }
    pub fn request(&self, source: &'static str) -> Correlation {
        let mut state = self.state.lock().unwrap();
        let id = format!("host-{}", state.requests);
        state.requests = state.requests.checked_add(1).expect("host request counter exhausted");
        Correlation::request(&id, source, false)
    }
    pub fn event(&self, frame: u64, clock: u64, stage: &str, data: Value) -> Event {
        let correlation = self.current();
        let mut state = self.state.lock().unwrap();
        let sequence = state.next;
        state.next = state.next.checked_add(1).expect("event sequence exhausted");
        let elapsed = state.started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        let (category, producer, observation) = classification(stage);
        let outcome=data.get("outcome").or_else(||data.pointer("/details/outcome")).and_then(Value::as_str);
        let severity = if stage == "error" || outcome.is_some_and(|v|["refused","error","failed","timeout","incompatible"].contains(&v)) { "error" } else { "info" };
        Event { metadata: json!({"event_schema":1,"event_id":format!("{}:{sequence}",state.session.run_id),"event_sequence":sequence,
            "host_elapsed_ns":elapsed,"session":state.session,"category":category,"severity":severity,
            "origin":{"producer":producer,"source":correlation.source},"request_id":correlation.request_id,"job_id":correlation.job_id,"observation":observation}),
            frame, clock, stage: stage.into(), data }
    }
    pub fn status(&self) -> Value {
        let state = self.state.lock().unwrap();
        json!({"schema":1,"kind":"ffta-event-clock","session":state.session,"next_event_sequence":state.next,
            "ordering":"Host publication order within this run; guest frame/clock can repeat or rewind. Gaps include inactive collectors, deduplication and collector losses"})
    }
}
/// Owned guard permits nesting around existing handlers without retaining a
/// borrow of the frontend. Context restoration also covers early returns.
pub(crate) struct Context { context: Arc<Mutex<Correlation>>, previous: Correlation }
impl Drop for Context { fn drop(&mut self) { *self.context.lock().unwrap() = self.previous.clone(); } }

pub(crate) struct Event { metadata: Value, pub frame: u64, pub clock: u64, pub stage: String, pub data: Value }
impl Event {
    /// Local sequence/time aliases remain compatible with existing schema-1
    /// reports. Compare the canonical event_* fields across collectors.
    pub fn value(&self, sequence: u64, host_ms: u64) -> Value {
        let mut value = self.metadata.clone();
        let object = value.as_object_mut().unwrap();
        object.extend([(String::from("sequence"), json!(sequence)), (String::from("host_ms"), json!(host_ms)),
            (String::from("frame"), json!(self.frame)), (String::from("clock"), json!(self.clock)),
            (String::from("stage"), json!(self.stage)), (String::from("data"), self.data.clone())]);
        value
    }
}

/// Poll deduplication happens once, before fanout. Neither consumer owns the
/// shared ordering, and a full/frozen consumer cannot suppress the other one.
pub(crate) struct Hub { pub clock: Clock, poll: Option<Value>, pad: Option<Value> }
impl Hub {
    pub fn new(session: Session) -> Self { Self { clock: Clock::new(session), poll: None, pad: None } }
    pub fn reset_polls(&mut self) { self.poll = None; self.pad = None; }
    #[cfg(test)]
    pub fn publish(&mut self, frame: u64, clock: u64, stage: &str, data: Value) -> Option<Event> {
        self.publish_with_poll_coalescing(frame,clock,stage,data,true)
    }
    pub fn publish_with_poll_coalescing(&mut self, frame: u64, clock: u64, stage: &str, data: Value, coalesce: bool) -> Option<Event> {
        let previous = match stage { "mapped-poll" => Some(&mut self.poll), "controller-poll" => Some(&mut self.pad), _ => None };
        if let Some(previous) = previous {
            if coalesce && previous.as_ref() == Some(&data) { return None; }
            *previous = Some(data.clone());
        }
        Some(self.clock.event(frame, clock, stage, data))
    }
}

fn classification(stage: &str) -> (&'static str, &'static str, &'static str) {
    match stage {
        "runtime-event" => ("runtime", "frontend.lifecycle", "host-boundary"),
        "keyboard" => ("input", "frontend.keyboard", "host-token"),
        "mouse-input" => ("input", "frontend.mouse", "host-token"),
        "host-text-input" => ("input", "frontend.editor", "host-token"),
        "host-input-consumed" | "host-pointer-dispatch" => ("input", "frontend.routing", "host-boundary"),
        "host-menu-baseline" | "host-menu-change" => ("selection", "frontend.menu-state", "host-boundary"),
        "binding-capture-result" => ("selection", "frontend.bindings", "host-boundary"),
        "controller-poll" => ("input", "frontend.xinput", "host-poll-change"),
        "mapped-poll" => ("input", "frontend.mapping", "host-poll-change"),
        "shortcut-consumed" | "text-input-consumed" | "modal-input-consumed" | "menu-consumed-by-remapping" | "menu-debounce-refused" => ("input", "frontend.routing", "host-boundary"),
        "bindings-changed" | "pad-bindings-changed" => ("selection", "frontend.bindings", "host-boundary"),
        "host-input-request" | "host-menu-dispatch" => ("selection", "frontend.menu", "host-boundary"),
        "host-action" | "host-action-result" | "host-action-refused" => ("command", "frontend.menu", "host-boundary"),
        "control-request" | "control-result" | "control-complete" => ("command", "frontend.control", "host-boundary"),
        "guest-input-latch" | "guest-frame" | "guest-input" => ("guest", "frontend.frame", "guest-boundary"),
        "rom-read-batch" | "frame" => ("guest", "recent.frame", "guest-frame-aggregate"),
        "recognized-ui-transition" => ("guest", "recent.recognizer", "partial-guest-signature"),
        "timeline-restored" => ("runtime", "frontend.restore", "host-boundary"),
        "armed" | "freeze" => ("capture", "recent.collector", "collector-boundary"),
        "error" => ("diagnostic", "frontend.error", "host-boundary"),
        _ => ("unknown", "unregistered", "unknown"),
    }
}

/// Published with each report, and readable via control status. This describes
/// actual hooks rather than promising future DBG2-09/10 coverage.
pub(crate) fn coverage() -> Value {
    json!({"schema":1,"kind":"ffta-event-coverage","event_schema":1,"runtime_families":crate::debug_lifecycle::coverage(),
        "ordering":"event_sequence/host_elapsed_ns are common publication order/time since run start; sequence/host_ms are legacy collector-local aliases. No cross-process or guest instruction chronology",
        "producers":[
            {"id":"frontend.keyboard","state":"supported","scope":"Winit physical keyboard tokens while a collector is active; repeat/synthetic flags retained"},
            {"id":"frontend.xinput","state":"conditional","scope":"Windows XInput token polls; identical polls normally coalesced, optional live-console switch emits each poll within collector caps. Simulated backend is not physical hardware acceptance"},
            {"id":"frontend.mapping","state":"supported","scope":"Mapped polls; identical polls normally coalesced before fanout, optional live-console switch emits each poll within collector caps"},
            {"id":"frontend.mouse","state":"supported","scope":"Winit button/wheel tokens; hover emits semantic changes only. Queued pointer dispatch retains request context"},
            {"id":"frontend.editor","state":"conditional","scope":"Winit IME preedit/commit tokens and effective bounded editor text/focus changes; physical IME acceptance remains platform-specific"},
            {"id":"frontend.routing","state":"supported","scope":"Existing shortcut/text/modal/debounce/remap/editor/pointer consumption boundaries; raw tokens remain distinct from mapped and consumed observations"},
            {"id":"frontend.bindings","state":"partial","scope":"Existing keyboard/pad binding rebuild sites"},
            {"id":"frontend.menu","state":"supported","scope":"Existing accepted keyboard/pad navigation dispatch and emitted menu actions, including inline panel cancel; handler return is not asynchronous completion"},
            {"id":"frontend.menu-state","state":"supported","scope":"Changed semantic page/item ID/label/value/dropdown/editor/content/tool/binding/visibility properties at frontend input/handler boundaries; repeated identical hover/state emits nothing. Baseline explicit. Up to96properties/256character prefixes with full hashes on truncation"},
            {"id":"frontend.control","state":"supported","scope":"Non-status shared actions, accepted/refused and local-control pending completion; real mailbox request ID, step/search job ID. Handler return is not asynchronous job completion"},
            {"id":"frontend.frame","state":"supported","scope":"Actual input latch/completed frame boundaries only; no guest instruction stream"},
            {"id":"recent.frame","state":"conditional","scope":"Recent enabled; per-frame dispatch counts and up to16 sorted unique ROM read keys with rom-data-read-audit, explicit unavailable/dropped states. Not within-frame chronology"},
            {"id":"recent.recognizer","state":"partial","scope":"Changed recognized frontend guest UI signatures; unrecognized is unknown"},
            {"id":"frontend.restore","state":"partial","scope":"Existing trace stop at state restore; broader lifecycle hooks are DBG2-10"},
            {"id":"recent.collector","state":"supported","scope":"Existing recent arm/freeze boundaries; export/rendering never dispatches recursively"},
            {"id":"frontend.error","state":"partial","scope":"Existing recent_error and correlated action refusal sites; not every runtime error path"}
        ],
        "missing":["Raw Gilrs and DirectInput token hooks; effective mapped navigation and semantic changes remain observed","Touch/pen/other input routes absent from the frontend","Initial CLI extraction/cache compilation before frontend collection; worker attach/load is separately observed","CPU instruction chronology, native block interiors, all guest writes/scenes/scripts","Raw audio callback event stream, maintained cheat lifecycle and hardware-hook chronology, events before capture or after freeze"],
        "consumers":{"ui":"Explicit start;1024events/30seconds/600guest-frame events/2MiB report; own stop reason","recent":"Explicit start;1024events/256KiB total/8KiB event; eviction/oversize counters; freeze/error policy","console":"Optional live window;1024events/256KiB serialized/8KiB event; independent view pause/filter and eviction/oversize counters"},
        "correlation":"Nullable request_id/job_id; absence means no observed request context. A capture started by an action can retain its result without the preceding request. Metadata fanout never calls an action handler"})
}

#[cfg(test)] mod tests;
mod selections;
pub(crate) use selections::Selections;
