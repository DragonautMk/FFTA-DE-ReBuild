//! Bounded, in-memory plans for the isolated interpreter. No code is applied
//! until the owning frontend explicitly reaches its selected frame boundary.
use crate::debug_cheat_decoder::{Footprint, Plan};
use serde_json::{json, Value};

const MAX_ARMED: usize = 8;
const MAX_FRAME_WRITES: usize = 256;

struct Armed {
    id: String,
    plan: Plan,
    footprint: Footprint,
    identity: Value,
}

#[derive(Default)]
pub(crate) struct ArmedSet {
    entries: Vec<Armed>,
}

impl ArmedSet {
    pub(crate) fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub(crate) fn len(&self) -> usize { self.entries.len() }
    pub(crate) fn contains(&self, id: &str, source_sha256: &str) -> bool {
        self.entries.iter().any(|entry| entry.id == id && entry.identity["source_sha256"].as_str() == Some(source_sha256))
    }

    pub(crate) fn arm(&mut self, id: &str, plan: Plan, identity: Value) -> Result<(), String> {
        if self.entries.len() >= MAX_ARMED { return Err("At most eight cheats may be armed".into()); }
        if self.entries.iter().any(|entry| entry.id == id) { return Err("This cheat is already armed".into()); }
        let footprint = plan.footprint()?;
        let writes = self.entries.iter().map(|entry| entry.footprint.max_writes()).sum::<usize>();
        if writes + footprint.max_writes() > MAX_FRAME_WRITES { return Err("Armed cheats exceed the per-frame write limit".into()); }
        if self.entries.iter().any(|entry| entry.footprint.conflicts(&footprint)) {
            return Err("Cheat read/write footprint overlaps an armed cheat".into());
        }
        self.entries.push(Armed { id: id.into(), plan, footprint, identity });
        Ok(())
    }

    pub(crate) fn disarm(&mut self, id: &str) -> Result<(), String> {
        let index = self.entries.iter().position(|entry| entry.id == id).ok_or("Cheat is not armed")?;
        self.entries.remove(index);
        Ok(())
    }

    pub(crate) fn clear(&mut self) { self.entries.clear(); }

    pub(crate) fn status(&self) -> Value {
        json!({"active":!self.is_empty(),"entries":self.entries.iter().map(|entry|json!({"id":entry.id,"identity":entry.identity,"footprint":entry.footprint})).collect::<Vec<_>>(),
            "timing":"Before each interpreter frame after input latch","scope":"Frozen source-bound plans; disarming stops future writes and does not restore prior bytes"})
    }

    pub(crate) fn apply<B: gba_core::Bus>(&self, bus: &mut B) -> Result<usize, String> {
        // Preflight the whole set before its first write, even if a future
        // decoder change makes a previously admitted plan invalid.
        for entry in &self.entries { entry.plan.footprint()?; }
        let mut writes = 0;
        for entry in &self.entries { writes += entry.plan.apply_once(bus)?; }
        Ok(writes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gba_core::Bus;

    fn plan(lines: &[&str]) -> Plan {
        crate::debug_cheat_decoder::inspect(&lines.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn armed_set_rejects_overlap_and_disarm_stops_future_writes() {
        let mut set = ArmedSet::default();
        set.arm("CH-0001", plan(&["82000002 1234"]), json!({})).unwrap();
        assert!(set.arm("CH-0001", plan(&["82000004 5678"]), json!({})).is_err());
        assert!(set.arm("CH-0002", plan(&["32000003 0055"]), json!({})).is_err());
        assert!(set.arm("CH-0003", plan(&["72000002 1234", "82000006 7777"]), json!({})).is_err());
        set.arm("CH-0004", plan(&["82000004 5678"]), json!({})).unwrap();
        let mut machine = gba_core::Machine::new(vec![]);
        assert_eq!(set.apply(&mut machine.bus).unwrap(), 2);
        assert_eq!(machine.bus.read16(0x02000002), 0x1234);
        assert_eq!(machine.bus.read16(0x02000004), 0x5678);
        set.disarm("CH-0001").unwrap();
        machine.bus.write16(0x02000002, 0x9999);
        assert_eq!(set.apply(&mut machine.bus).unwrap(), 1);
        assert_eq!(machine.bus.read16(0x02000002), 0x9999);
        assert!(set.disarm("CH-0001").is_err());
        set.clear();
        assert!(set.is_empty());
        set.arm("CH-0003", plan(&["72000002 1234", "82000006 7777"]), json!({})).unwrap();
        assert!(set.arm("CH-0001", plan(&["82000002 1234"]), json!({})).is_err());
    }

    #[test]
    fn armed_set_caps_count_and_total_frame_work_without_partial_changes() {
        let mut set=ArmedSet::default();
        for index in 0..8 {
            let line=format!("8{:07X} 1234", 0x02000000u32+2*index);
            set.arm(&format!("CH-{index:04}"),plan(&[&line]),json!({})).unwrap();
        }
        assert!(set.arm("CH-0008",plan(&["82000020 1234"]),json!({})).is_err());
        assert_eq!(set.len(),8);
        set.clear();
        set.arm("CH-0001",plan(&["42000000 0000","00000100 0002"]),json!({})).unwrap();
        assert!(set.arm("CH-0002",plan(&["82000400 1234"]),json!({})).is_err());
        assert_eq!(set.len(),1);
    }
}
