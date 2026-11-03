//! Deferred single edges give almost-simultaneous shoulder chords priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    VolumeDown,
    VolumeUp,
    Mute,
    SpeedDown,
    SpeedUp,
    FastToggle,
}
#[derive(Default)]
struct Pair {
    pending: Option<(u8, u64)>,
    latched: bool,
    suppressed: bool,
}
impl Pair {
    fn poll(&mut self, bits: u8, enabled: bool, ms: u64) -> Option<u8> {
        if !enabled {
            self.pending = None;
            self.latched = false;
            self.suppressed = bits != 0;
            return None;
        }
        if self.suppressed {
            if bits == 0 {
                self.suppressed = false;
            }
            return None;
        }
        if self.latched {
            if bits == 0 {
                self.latched = false;
            }
            return None;
        }
        if bits == 3 {
            self.pending = None;
            self.latched = true;
            return Some(3);
        }
        if let Some((button, since)) = self.pending {
            if bits == 0 {
                self.pending = None;
                return Some(button);
            }
            if ms.saturating_sub(since) >= 80 {
                self.pending = None;
                self.latched = true;
                return Some(button);
            }
        } else if bits != 0 {
            self.pending = Some((bits, ms));
        }
        None
    }
}
#[derive(Default)]
pub struct ControllerShortcuts {
    volume: Pair,
    speed: Pair,
}
impl ControllerShortcuts {
    /// Call while closed/capturing too, so held controls cannot become fresh menu edges.
    pub fn poll(&mut self, down: [bool; 4], enabled: bool, ms: u64) -> Vec<Shortcut> {
        let mut out = Vec::new();
        for (pair, bits, actions) in [
            (
                &mut self.volume,
                u8::from(down[0]) | u8::from(down[1]) * 2,
                [Shortcut::VolumeDown, Shortcut::VolumeUp, Shortcut::Mute],
            ),
            (
                &mut self.speed,
                u8::from(down[2]) | u8::from(down[3]) * 2,
                [Shortcut::SpeedDown, Shortcut::SpeedUp, Shortcut::FastToggle],
            ),
        ] {
            if let Some(edge) = pair.poll(bits, enabled, ms) {
                out.push(actions[usize::from(edge - 1)]);
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chords_win_before_single_and_partial_release_never_repeats() {
        let mut s = ControllerShortcuts::default();
        assert!(s.poll([true, false, false, false], true, 0).is_empty());
        assert_eq!(
            s.poll([true, true, false, false], true, 40),
            [Shortcut::Mute]
        );
        assert!(s.poll([false, true, false, false], true, 100).is_empty());
        assert!(s.poll([false; 4], true, 120).is_empty());
        assert!(s.poll([false, false, true, false], true, 140).is_empty());
        assert_eq!(
            s.poll([false, false, true, true], true, 170),
            [Shortcut::FastToggle]
        );
        assert!(s.poll([false, false, true, false], true, 250).is_empty());
    }
    #[test]
    fn singles_are_once_and_capture_or_closed_controls_require_neutral() {
        let mut s = ControllerShortcuts::default();
        s.poll([true, false, false, false], true, 0);
        assert_eq!(
            s.poll([true, false, false, false], true, 80),
            [Shortcut::VolumeDown]
        );
        assert!(s.poll([true, false, false, false], true, 400).is_empty());
        s.poll([false; 4], true, 410);
        s.poll([false, true, false, false], false, 500);
        assert!(s.poll([false, true, false, false], true, 600).is_empty());
        s.poll([false; 4], true, 610);
        s.poll([false, true, false, false], true, 620);
        assert_eq!(s.poll([false; 4], true, 630), [Shortcut::VolumeUp]);
    }
}
