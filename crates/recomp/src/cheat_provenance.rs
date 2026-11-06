//! Bounded, conservative cheat ancestry for portable snapshots, not replay data.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MAX_EVENTS: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct History {
    pub(crate) schema: u8,
    pub(crate) events: Vec<Value>,
    pub(crate) truncated: bool,
}

impl Default for History {
    fn default() -> Self { Self { schema: 1, events: Vec::new(), truncated: false } }
}

impl History {
    pub(crate) fn is_empty(&self) -> bool { self.events.is_empty() && !self.truncated }

    /// Retain only identities/hashes, never raw third-party codes or plans.
    pub(crate) fn record(&mut self, kind: &str, identity: Value, frame: u64, clock: u64, writes: usize) {
        if let Some(last) = self.events.last_mut() {
            if last["kind"] == kind && last["identity"] == identity
                && last["last_frame"].as_u64().and_then(|v| v.checked_add(1)) == Some(frame) {
                last["last_frame"] = json!(frame);
                last["last_clock"] = json!(clock);
                last["applications"] = json!(last["applications"].as_u64().unwrap_or(0).saturating_add(1));
                last["writes"] = json!(last["writes"].as_u64().unwrap_or(0).saturating_add(writes as u64));
                return;
            }
        }
        if self.events.len() >= MAX_EVENTS { self.truncated = true; return; }
        self.events.push(json!({"kind":kind,"identity":identity,"first_frame":frame,"first_clock":clock,
            "last_frame":frame,"last_clock":clock,"applications":1,"writes":writes}));
    }

    /// Union conservatively: a clean restore cannot prove cartridge data is clean.
    pub(crate) fn include(&mut self, other: &Self) {
        for event in &other.events {
            if self.events.contains(event) { continue; }
            if self.events.len() >= MAX_EVENTS { self.truncated = true; break; }
            self.events.push(event.clone());
        }
        self.truncated |= other.truncated;
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema != 1 || self.events.len() > MAX_EVENTS || (self.events.is_empty() && !self.truncated) {
            return Err("Invalid cheat ancestry".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestry_is_bounded_and_survives_conservative_merges() {
        let mut source = History::default();
        for frame in 0..70 { source.record("once", json!({"id":frame}), frame, frame * 2, 1); }
        assert_eq!(source.events.len(), 64);
        assert!(source.truncated);
        let mut loaded = History::default();
        loaded.include(&source);
        loaded.include(&History::default());
        assert!(!loaded.is_empty());
        assert!(loaded.truncated);
        assert_eq!(loaded.events, source.events);
    }

    #[test]
    fn maintained_intervals_keep_order_without_raw_cheat_lines() {
        let mut history = History::default();
        for frame in 10..13 { history.record("maintained", json!([{"id":"synthetic"}]), frame, frame * 100, 2); }
        assert_eq!(history.events.len(), 1);
        assert_eq!(history.events[0]["first_frame"], 10);
        assert_eq!(history.events[0]["last_frame"], 12);
        assert_eq!(history.events[0]["applications"], 3);
        assert_eq!(history.events[0]["writes"], 6);
        history.record("once", json!({"id":"another"}), 13, 1300, 1);
        assert_eq!(history.events.len(), 2);
    }
}
