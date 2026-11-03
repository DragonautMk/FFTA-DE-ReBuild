//! Exact menu percentages; legacy 2/4/8 multipliers keep their existing meaning.
pub const PERCENTS: [u32; 36] = [
    100, 120, 140, 160, 180, 200, 220, 240, 260, 280, 300, 320, 340, 360, 380, 400, 420, 440, 460,
    480, 500, 520, 540, 560, 580, 600, 620, 640, 660, 680, 700, 720, 740, 760, 780, 800,
];
pub fn selected_percent(value: u32) -> u32 {
    match value {
        2 | 4 | 8 => value * 100,
        100..=800 if value % 20 == 0 => value,
        _ => 400,
    }
}
pub fn effective_percent(value: u32, latched: bool, held: bool) -> u32 {
    if latched ^ held {
        selected_percent(value)
    } else {
        100
    }
}
/// Distributes whole guest frames across host pacing ticks without rounding the chosen rate.
#[derive(Default, Debug)]
pub struct FrameClock {
    remainder: u32,
}
impl FrameClock {
    pub fn reset(&mut self) {
        self.remainder = 0;
    }
    pub fn next_batch(&mut self, percent: u32) -> u32 {
        let n = self.remainder + selected_percent(percent);
        self.remainder = n % 100;
        n / 100
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_rates_and_activation_preserve_legacy_values() {
        for p in [100, 120, 140, 200, 400, 800] {
            let mut clock = FrameClock::default();
            let n: u32 = (0..5).map(|_| clock.next_batch(p)).sum();
            assert_eq!(n, p / 20);
            assert_eq!(effective_percent(p, false, false), 100);
            assert_eq!(effective_percent(p, true, false), p);
            assert_eq!(effective_percent(p, true, true), 100);
        }
        for (old, p) in [(2, 200), (4, 400), (8, 800)] {
            assert_eq!(selected_percent(old), p);
        }
        let mut clock = FrameClock::default();
        assert_eq!(
            (0..5).map(|_| clock.next_batch(120)).collect::<Vec<_>>(),
            [1, 1, 1, 1, 2]
        );
        clock.next_batch(120);
        clock.reset();
        assert_eq!(clock.next_batch(140), 1);
    }
}
#[cfg(test)]
mod config_tests {
    #[test]
    fn percentages_roundtrip_without_changing_legacy_configs() {
        use crate::AvConfig;
        for percent in super::PERCENTS {
            let cfg = AvConfig::parse(&format!("ff.percent = {percent}\n"));
            assert_eq!(cfg.ff_speed, percent);
            assert_eq!(AvConfig::parse(&cfg.serialize()), cfg);
        }
        for legacy in [2, 4, 8] {
            let cfg = AvConfig::parse(&format!("ff.speed = {legacy}\n"));
            assert_eq!(cfg.ff_speed, legacy);
            assert_eq!(AvConfig::parse(&cfg.serialize()), cfg);
        }
        assert_eq!(AvConfig::parse("ff.percent = 110\n").ff_speed, 4);
    }
}
