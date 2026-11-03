//! Typed audio fields serve guest bus reads and the host shadow sampler.
use super::*;
use crate::rom_data::{audio_directory, audio_tones as model};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioTonesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub tone_bytes: u64,
    pub keymap_bytes: u64,
    pub wave_header_bytes: u64,
    pub pcm_bytes: u64,
    pub endpoint_bytes: u64,
    pub cgb_bytes: u64,
    pub host_reads: u64,
    pub host_source_bytes: u64,
    pub host_tone_bytes: u64,
    pub host_keymap_bytes: u64,
    pub host_wave_header_bytes: u64,
    pub host_pcm_bytes: u64,
    pub host_endpoint_bytes: u64,
    pub host_cgb_bytes: u64,
    pub tone_masks: [u64; 7],
    pub map_masks: [u64; 6],
    pub wave_masks: [u64; 5],
    pub cgb_mask: u64,
    pub host_wave_masks: [u64; 5],
}
struct Span {
    start: usize,
    end: usize,
    kind: u8,
    record: usize,
}
pub(super) struct AudioTonesResources {
    data: model::AudioTones,
    spans: Vec<Span>,
    guest: [AtomicU64; 8],
    host: [AtomicU64; 8],
    tone_masks: [AtomicU64; 7],
    map_masks: [AtomicU64; 6],
    wave_masks: [AtomicU64; 5],
    cgb_mask: AtomicU64,
    host_wave_masks: [AtomicU64; 5],
}
impl AudioTonesResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x14d..=0x337)
    }
    pub(super) fn new(
        data: model::AudioTones,
        sources: &[SourceRange],
        directory: &audio_directory::AudioDirectory,
    ) -> Result<Self, String> {
        data.validate_directory(directory)?;
        let expected = data.source_spans();
        if expected.len() != sources.len() {
            return Err("Audio tone source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Audio tone typed source binding mismatch".into());
            }
        }
        let mut spans = Vec::new();
        for (i, t) in data.tones.iter().enumerate() {
            spans.push(Span {
                start: t.source_offset as usize,
                end: t.source_offset as usize + 12,
                kind: 0,
                record: i,
            });
        }
        for (i, m) in data.map_bytes.iter().enumerate() {
            spans.push(Span {
                start: m.source_offset as usize,
                end: m.source_offset as usize + 1,
                kind: 1,
                record: i,
            });
        }
        for (i, w) in data.pcm_waves.iter().enumerate() {
            let skip = if data.directory_aliases.iter().any(|a| a.wave == w.id) {
                4
            } else {
                0
            };
            spans.push(Span {
                start: w.source_offset as usize + skip,
                end: w.source_offset as usize
                    + 16
                    + w.samples.len()
                    + usize::from(w.interpolation_endpoint.is_some()),
                kind: 2,
                record: i,
            });
        }
        for (i, w) in data.cgb_waves.iter().enumerate() {
            spans.push(Span {
                start: w.source_offset as usize,
                end: w.source_offset as usize + 16,
                kind: 3,
                record: i,
            });
        }
        spans.sort_by_key(|s| s.start);
        Ok(Self {
            data,
            spans,
            guest: std::array::from_fn(|_| AtomicU64::new(0)),
            host: std::array::from_fn(|_| AtomicU64::new(0)),
            tone_masks: std::array::from_fn(|_| AtomicU64::new(0)),
            map_masks: std::array::from_fn(|_| AtomicU64::new(0)),
            wave_masks: std::array::from_fn(|_| AtomicU64::new(0)),
            host_wave_masks: std::array::from_fn(|_| AtomicU64::new(0)),
            cgb_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize, usize)> {
        let i = self
            .spans
            .partition_point(|s| s.start <= at)
            .checked_sub(1)?;
        let s = &self.spans[i];
        if at >= s.end {
            return None;
        }
        let (value, category) = match s.kind {
            0 => (
                self.data.tones[s.record]
                    .byte(at - self.data.tones[s.record].source_offset as usize)
                    .expect("validated tone byte"),
                2,
            ),
            1 => (self.data.map_bytes[s.record].selector, 3),
            2 => {
                let w = &self.data.pcm_waves[s.record];
                let offset = at - w.source_offset as usize;
                (
                    w.byte(offset).expect("validated PCM byte"),
                    if offset < 16 {
                        4
                    } else if offset < 16 + w.samples.len() {
                        5
                    } else {
                        6
                    },
                )
            }
            _ => (
                self.data.cgb_waves[s.record]
                    .byte(at - self.data.cgb_waves[s.record].source_offset as usize)
                    .expect("validated CGB byte"),
                7,
            ),
        };
        Some((value, category, s.record))
    }
    fn read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
        host: bool,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 8];
        let bytes = std::array::from_fn(|i| {
            if let Some((v, kind, record)) = fields[i] {
                counts[1] += 1;
                counts[kind] += 1;
                if host {
                    if (4..=6).contains(&kind) {
                        self.host_wave_masks[record / 64]
                            .fetch_or(1 << (record % 64), Ordering::Relaxed);
                    }
                } else {
                    let masks = match kind {
                        2 => &self.tone_masks[..],
                        3 => &self.map_masks[..],
                        4..=6 => &self.wave_masks[..],
                        _ => &[],
                    };
                    if kind == 7 {
                        self.cgb_mask.fetch_or(1 << record, Ordering::Relaxed);
                    } else {
                        masks[record / 64].fetch_or(1 << (record % 64), Ordering::Relaxed);
                    }
                }
                v
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        counts[0] = 1;
        let counters = if host { &self.host } else { &self.guest };
        for (counter, count) in counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, false)
    }
    pub(super) fn host_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, true)
    }
    pub(super) fn stats(&self) -> AudioTonesStats {
        let [bus_reads, source_bytes, tone_bytes, keymap_bytes, wave_header_bytes, pcm_bytes, endpoint_bytes, cgb_bytes] =
            self.guest.each_ref().map(|v| v.load(Ordering::Relaxed));
        let [host_reads, host_source_bytes, host_tone_bytes, host_keymap_bytes, host_wave_header_bytes, host_pcm_bytes, host_endpoint_bytes, host_cgb_bytes] =
            self.host.each_ref().map(|v| v.load(Ordering::Relaxed));
        AudioTonesStats {
            bus_reads,
            source_bytes,
            tone_bytes,
            keymap_bytes,
            wave_header_bytes,
            pcm_bytes,
            endpoint_bytes,
            cgb_bytes,
            host_reads,
            host_source_bytes,
            host_tone_bytes,
            host_keymap_bytes,
            host_wave_header_bytes,
            host_pcm_bytes,
            host_endpoint_bytes,
            host_cgb_bytes,
            tone_masks: self
                .tone_masks
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            map_masks: self.map_masks.each_ref().map(|v| v.load(Ordering::Relaxed)),
            wave_masks: self
                .wave_masks
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            host_wave_masks: self
                .host_wave_masks
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            cgb_mask: self.cgb_mask.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<Span>()
            + self.data.banks.capacity() * std::mem::size_of::<model::Bank>()
            + self.data.tones.capacity() * std::mem::size_of::<model::Tone>()
            + self.data.map_bytes.capacity() * std::mem::size_of::<model::MapByte>()
            + self.data.map_links.capacity() * std::mem::size_of::<model::MapLink>()
            + self.data.pcm_waves.capacity() * std::mem::size_of::<model::PcmWave>()
            + self.data.cgb_waves.capacity() * std::mem::size_of::<model::CgbWave>()
            + self
                .data
                .pcm_waves
                .iter()
                .map(|w| w.samples.capacity() + w.tone_ids.capacity() * 2)
                .sum::<usize>()
            + self
                .data
                .cgb_waves
                .iter()
                .map(|w| w.tone_ids.capacity() * 2)
                .sum::<usize>()
            + self
                .data
                .banks
                .iter()
                .map(|b| {
                    b.programs.capacity() * std::mem::size_of::<model::Program>()
                        + b.programs
                            .iter()
                            .map(|p| p.note_keys.capacity())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
