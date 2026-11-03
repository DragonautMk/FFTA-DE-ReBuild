//! Host event quarantine when a native modal dialog returns. No guest fields.
use std::time::{Duration, Instant};
const QUIET: Duration = Duration::from_millis(400);
#[derive(Default)]
pub(crate) struct ModalInput {
    active: bool,
    until: Option<Instant>,
    neutral_polls: u8,
}
impl ModalInput {
    pub fn begin(&mut self) {
        self.active = true;
        self.until = None;
        self.neutral_polls = 0;
    }
    pub fn finish(&mut self, now: Instant) {
        self.active = true;
        self.until = Some(now + QUIET);
        self.neutral_polls = 0;
    }
    pub fn focus_return(&mut self, now: Instant) {
        if self.active && self.until.is_some() {
            self.finish(now);
        }
    }
    pub fn blocked(&self) -> bool {
        self.active
    }
    /// At least two host polls and a neutral opening/closing gesture are needed.
    /// Keep draining queued and focus-synthetic events even after a long modal.
    pub fn observe(&mut self, now: Instant, neutral: bool) {
        if !self.active || self.until.is_none() {
            return;
        }
        self.neutral_polls = if neutral {
            self.neutral_polls.saturating_add(1)
        } else {
            0
        };
        if self.neutral_polls >= 2 && self.until.is_some_and(|until| now >= until) {
            self.active = false;
            self.until = None;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modal_never_releases_before_return_or_quiet_neutral_polls() {
        let now = Instant::now();
        let mut g = ModalInput::default();
        assert!(!g.blocked());
        g.begin();
        g.observe(now + Duration::from_secs(10), true);
        assert!(g.blocked());
        g.finish(now);
        g.observe(now + QUIET / 2, true);
        g.observe(now + QUIET / 2, true);
        assert!(g.blocked());
        g.observe(now + QUIET, false);
        assert!(g.blocked());
        g.observe(now + QUIET, true);
        assert!(g.blocked());
        g.observe(now + QUIET + Duration::from_millis(1), true);
        assert!(!g.blocked());
    }
    #[test]
    fn focus_restore_and_held_controller_extend_the_return_boundary() {
        let now = Instant::now();
        let mut g = ModalInput::default();
        g.begin();
        g.finish(now);
        g.focus_return(now + QUIET);
        g.observe(now + QUIET, true);
        g.observe(now + QUIET, true);
        assert!(g.blocked());
        g.observe(now + QUIET * 2, false);
        assert!(g.blocked());
        g.observe(now + QUIET * 2, true);
        assert!(g.blocked());
        g.observe(now + QUIET * 2, true);
        assert!(!g.blocked());
    }
}
