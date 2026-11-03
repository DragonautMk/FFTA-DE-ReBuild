//! Bounded read-only event window. Collection and view state are independent.
use serde_json::{json, Value};
use std::{collections::{BTreeSet, VecDeque}, sync::Arc, time::Instant};

const MAX_EVENTS: usize = 1024;
const MAX_BYTES: usize = 256 * 1024;
const MAX_EVENT_BYTES: usize = 8 * 1024;
const MAX_QUERY_CHARS: usize = 256;
const MAX_EVENTS_PER_TICK: usize = 32;
const MAX_INDEXED_BYTES: usize = 1024 * 1024;

struct Entry {
    encoded: String,
    searchable: String,
    category: String,
    severity: String,
    bytes: usize,
}
impl Entry {
    fn value(&self) -> Value { serde_json::from_str(&self.encoded).expect("retained valid event") }
}

/// Filters only the view; they never decide what enters the collector.
#[derive(Clone, Default)]
pub(crate) struct Filter {
    query: String,
    categories: BTreeSet<String>,
    levels: BTreeSet<String>,
}
impl Filter {
    pub fn set_query(&mut self, query: &str) {
        self.query = query.chars().take(MAX_QUERY_CHARS).collect::<String>().to_lowercase();
    }
    pub fn set_categories(&mut self, categories: impl IntoIterator<Item = String>) {
        self.categories = categories.into_iter().collect();
    }
    pub fn set_levels(&mut self, levels: impl IntoIterator<Item = String>) {
        self.levels = levels.into_iter().collect();
    }
    fn matches(&self, e: &Entry) -> bool {
        (self.categories.is_empty() || self.categories.contains(&e.category))
            && (self.levels.is_empty() || self.levels.contains(&e.severity))
            && (self.query.is_empty() || e.searchable.contains(&self.query))
    }
}

/// A frozen view owns a bounded snapshot. The live collector keeps advancing,
/// and its current retention/loss state remains separately observable.
pub(crate) struct Window {
    collecting: bool,
    started: Option<Instant>,
    entries: VecDeque<Arc<Entry>>,
    bytes: usize,
    attempted: u64,
    evicted: u64,
    oversize: u64,
    budget_dropped: u64,
    tick_accepted: usize,
    indexed_bytes: usize,
    paused: Option<Vec<Arc<Entry>>>,
    pub filter: Filter,
    pub autoscroll: bool,
    pub coalesce_polls: bool,
}
impl Default for Window {
    fn default() -> Self { Self { collecting:false, started:None, entries:VecDeque::new(), bytes:0, attempted:0, evicted:0, oversize:0, budget_dropped:0, tick_accepted:0, indexed_bytes:0, paused:None, filter:Filter::default(), autoscroll:false, coalesce_polls:true } }
}
impl Window {
    pub fn start(&mut self) {
        if !self.collecting {
            self.collecting = true;
            if self.started.is_none() { self.started = Some(Instant::now()); self.autoscroll = true; }
        }
    }
    pub fn stop(&mut self) { self.collecting = false; }
    pub fn enabled(&self) -> bool { self.collecting }
    /// Replenish only at the frontend event-loop boundary.
    pub fn begin_tick(&mut self) { self.tick_accepted=0; }
    pub fn accept(&mut self, event: &crate::debug_events::Event) {
        if !self.collecting { return; }
        let sequence = self.attempted;
        self.attempted = self.attempted.saturating_add(1);
        if self.tick_accepted >= MAX_EVENTS_PER_TICK {
            self.budget_dropped = self.budget_dropped.saturating_add(1);
            return;
        }
        self.tick_accepted += 1;
        // Local aliases are independent of canonical event_id/event_sequence.
        let elapsed = self.started.expect("enabled collector has a start").elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let value = event.value(sequence, elapsed);
        let encoded = serde_json::to_string(&value).expect("JSON values are serializable");
        let bytes = encoded.len();
        if bytes > MAX_EVENT_BYTES {
            self.oversize = self.oversize.saturating_add(1);
            return;
        }
        let searchable = encoded.to_lowercase();
        let indexed = encoded.len() + searchable.len();
        if indexed > MAX_INDEXED_BYTES { self.oversize=self.oversize.saturating_add(1); return; }
        while self.entries.len() >= MAX_EVENTS || self.bytes + bytes > MAX_BYTES || self.indexed_bytes + indexed > MAX_INDEXED_BYTES {
            let old = self.entries.pop_front().expect("bounded nonempty window");
            self.bytes -= old.bytes;
            self.indexed_bytes -= old.encoded.len() + old.searchable.len();
            self.evicted = self.evicted.saturating_add(1);
        }
        self.bytes += bytes;
        self.indexed_bytes += indexed;
        let category=value["category"].as_str().unwrap_or("unknown").to_owned();
        let severity=value["severity"].as_str().unwrap_or("info").to_owned();
        self.entries.push_back(Arc::new(Entry { encoded, searchable, category, severity, bytes }));
    }
    pub fn pause_view(&mut self, pause: bool) {
        if pause && self.paused.is_none() {
            self.paused = Some(self.entries.iter().cloned().collect());
        } else if !pause {
            self.paused = None;
        }
    }
    pub fn view_paused(&self) -> bool { self.paused.is_some() }
    fn source(&self) -> Box<dyn Iterator<Item = &Arc<Entry>> + '_> {
        match &self.paused {
            Some(entries) => Box::new(entries.iter()),
            None => Box::new(self.entries.iter()),
        }
    }
    /// Decode only the requested viewport; filtering inspects bounded cached text.
    pub fn visible_count(&self) -> usize { self.source().filter(|e|self.filter.matches(e)).count() }
    pub fn visible_page(&self, start:usize, count:usize) -> Vec<Value> {
        self.source().filter(|e|self.filter.matches(e)).skip(start).take(count).map(|e|e.value()).collect()
    }
    pub fn visible(&self) -> Vec<Value> { self.visible_page(0, MAX_EVENTS) }
    pub fn status(&self) -> Value {
        json!({"collecting":self.collecting,"view_paused":self.view_paused(),"autoscroll":self.autoscroll,
            "poll_coalescing":self.coalesce_polls,"retained":self.entries.len(),"bytes":self.bytes,"attempted":self.attempted,
            "evicted":self.evicted,"oversize":self.oversize,"budget_dropped":self.budget_dropped,
            "indexed_bytes":self.indexed_bytes,"snapshot_rows":self.paused.as_ref().map_or(0,Vec::len),
            "snapshot_indexed_bytes":self.paused.as_ref().map_or(0,|v|v.iter().map(|e|e.encoded.len()+e.searchable.len()).sum::<usize>()),
            "first_event_id":self.entries.front().and_then(|e|e.value()["event_id"].as_str().map(str::to_owned)),
            "last_event_id":self.entries.back().and_then(|e|e.value()["event_id"].as_str().map(str::to_owned)),
            "max_events":MAX_EVENTS,"max_bytes":MAX_BYTES,"max_indexed_bytes":MAX_INDEXED_BYTES,"max_event_bytes":MAX_EVENT_BYTES,"max_events_per_tick":MAX_EVENTS_PER_TICK})
    }
    /// Serialize the displayed window without mutating the underlying log.
    pub fn export(&self) -> Value {
        json!({"schema":1,"kind":"ffta-event-console-window","collector":self.status(),
            "filter":{"query":self.filter.query,"categories":self.filter.categories,"levels":self.filter.levels},
            "events":self.visible(),"event_coverage":crate::debug_events::coverage(),
            "scope":"Filtered view of retained host observations; no guest instruction chronology"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_events::Clock;
    #[test]
    fn filters_and_frozen_view_do_not_change_the_live_log() {
        let clock=Clock::default();let mut window=Window::default();window.start();
        window.accept(&clock.event(0,0,"keyboard",json!({"key":"Alpha"})));
        window.accept(&clock.event(0,0,"error",json!({"message":"Beta"})));
        let before=window.status();window.filter.set_query("BETA");window.filter.set_levels(["error".into()]);
        assert_eq!(window.visible().len(),1);assert_eq!(window.status()["retained"],before["retained"]);
        window.pause_view(true);let frozen=window.export();
        window.accept(&clock.event(0,0,"error",json!({"message":"Beta later"})));
        assert_eq!(window.visible().len(),1);assert_eq!(window.status()["retained"],3);
        assert_eq!(window.export()["events"],frozen["events"]);
        window.pause_view(false);assert_eq!(window.visible().len(),2);
        window.filter.set_categories(["input".into()]);assert!(window.visible().is_empty());
        window.filter=Filter::default();assert_eq!(window.visible().len(),3);
    }
    #[test]
    fn caps_losses_and_event_ids_are_explicit() {
        let clock=Clock::default();let mut window=Window::default();
        window.accept(&clock.event(0,0,"keyboard",json!({"key":"ignored"})));assert_eq!(window.status()["attempted"],0);
        window.start();for i in 0..1200 {window.begin_tick();window.accept(&clock.event(0,0,"keyboard",json!({"n":i})));}
        window.begin_tick();window.accept(&clock.event(0,0,"error",json!({"huge":"x".repeat(MAX_EVENT_BYTES)})));
        let status=window.status();assert!(status["retained"].as_u64().unwrap()<=MAX_EVENTS as u64);
        assert!(status["bytes"].as_u64().unwrap()<=MAX_BYTES as u64);assert!(status["evicted"].as_u64().unwrap()>0);
        assert_eq!(status["oversize"],1);let view=window.visible();assert_eq!(status["first_event_id"],view.first().unwrap()["event_id"]);
        assert_eq!(status["last_event_id"],view.last().unwrap()["event_id"]);assert_eq!(window.export()["events"].as_array().unwrap().len(),view.len());
    }
    #[test]
    fn sustained_burst_respects_budget_and_snapshot_survives_eviction() {
        let clock=Clock::default();let mut window=Window::default();window.start();
        for i in 0..32 { window.accept(&clock.event(0,0,"keyboard",json!({"n":i}))); }
        window.pause_view(true);let saved=window.export()["events"].clone();
        let started=Instant::now();
        for tick in 0..2000 {
            window.begin_tick();
            for n in 0..64 {window.accept(&clock.event(0,0,"keyboard",json!({"tick":tick,"n":n,"text":"é".repeat(40)})));}
        }
        let elapsed=started.elapsed();let status=window.status();
        assert_eq!(status["budget_dropped"],64000);
        assert_eq!(status["attempted"],128032);
        assert!(status["retained"].as_u64().unwrap()<=MAX_EVENTS as u64);
        assert!(status["bytes"].as_u64().unwrap()<=MAX_BYTES as u64);
        assert!(status["indexed_bytes"].as_u64().unwrap()<=MAX_INDEXED_BYTES as u64);
        assert!(status["evicted"].as_u64().unwrap()>0);
        assert_eq!(window.export()["events"],saved);
        assert_eq!(status["snapshot_rows"],32);
        window.pause_view(false);assert_ne!(window.export()["events"],saved);
        println!("sustained 128000 attempts in {:?}; status={}",elapsed,status);
    }

}
