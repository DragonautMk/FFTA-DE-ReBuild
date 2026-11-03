//! Typed player/directory/header fields. Original selection, track state and mixing remain guest work.
use super::*;
use crate::rom_data::audio_directory as model;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioDirectoryStats {
    pub host_reads: u64,
    pub host_source_bytes: u64,
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub player_bytes: u64,
    pub directory_bytes: u64,
    pub header_bytes: u64,
    pub track_pointer_bytes: u64,
    pub player_mask: u64,
    pub song_masks: [u64; 10],
    pub header_masks: [u64; 8],
}
struct Span {
    start: usize,
    end: usize,
    kind: usize,
    record: usize,
}
pub(super) struct AudioDirectoryResources {
    data: model::AudioDirectory,
    spans: Vec<Span>,
    counters: [AtomicU64; 6],
    host: [AtomicU64; 2],
    player_mask: AtomicU64,
    song_masks: [AtomicU64; 10],
    header_masks: [AtomicU64; 8],
}
impl AudioDirectoryResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x14f..=0x150 | 0x338..=0x370)
    }
    pub(super) fn new(
        data: model::AudioDirectory,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        let decoded = data.source_spans();
        if sources.len() != decoded.len() {
            return Err("M4A source binding count mismatch".into());
        }
        let mut spans = Vec::with_capacity(sources.len());
        for (i, (source, (offset, bytes))) in sources.iter().zip(decoded).enumerate() {
            let role = match i {
                0 => "audio/m4a-players".into(),
                1 => "audio/m4a-directory".into(),
                _ => format!("audio/m4a-header/{}", data.headers[i - 2].id),
            };
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("M4A typed source binding mismatch".into());
            }
            spans.push(Span {
                start: offset as usize,
                end: offset as usize + bytes.len(),
                kind: i.min(2),
                record: i.saturating_sub(2),
            });
        }
        Ok(Self {
            data,
            spans,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            host: std::array::from_fn(|_| AtomicU64::new(0)),
            player_mask: AtomicU64::new(0),
            song_masks: std::array::from_fn(|_| AtomicU64::new(0)),
            header_masks: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize, usize)> {
        let i = self
            .spans
            .partition_point(|s| s.start <= at)
            .checked_sub(1)?;
        let span = &self.spans[i];
        if at >= span.end {
            return None;
        }
        let offset = at - span.start;
        match span.kind {
            0 => Some((
                self.data.players[offset / 12].byte(offset % 12)?,
                2,
                offset / 12,
            )),
            1 => Some((self.data.songs[offset / 8].byte(offset % 8)?, 3, offset / 8)),
            _ => Some((
                self.data.headers[span.record].byte(offset)?,
                if offset < 8 { 4 } else { 5 },
                span.record,
            )),
        }
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 6];
        let mut players = 0;
        let mut songs = [0u64; 10];
        let mut headers = [0u64; 8];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, kind, record)) = fields[i] {
                counts[1] += 1;
                counts[kind] += 1;
                match kind {
                    2 => players |= 1 << record,
                    3 => songs[record / 64] |= 1 << (record % 64),
                    _ => headers[record / 64] |= 1 << (record % 64),
                }
                value
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.player_mask.fetch_or(players, Ordering::Relaxed);
        for (counter, mask) in self.song_masks.iter().zip(songs) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        for (counter, mask) in self.header_masks.iter().zip(headers) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> AudioDirectoryStats {
        let [bus_reads, source_bytes, player_bytes, directory_bytes, header_bytes, track_pointer_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        AudioDirectoryStats {
            host_reads: self.host[0].load(Ordering::Relaxed),
            host_source_bytes: self.host[1].load(Ordering::Relaxed),
            bus_reads,
            source_bytes,
            player_bytes,
            directory_bytes,
            header_bytes,
            track_pointer_bytes,
            player_mask: self.player_mask.load(Ordering::Relaxed),
            song_masks: self
                .song_masks
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            header_masks: self
                .header_masks
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<Span>()
            + self.data.players.capacity() * std::mem::size_of::<model::Player>()
            + self.data.songs.capacity() * std::mem::size_of::<model::Song>()
            + self.data.headers.capacity() * std::mem::size_of::<model::Header>()
            + self
                .data
                .headers
                .iter()
                .map(|h| h.track_addresses.capacity() * 4)
                .sum::<usize>()
    }
    pub(super) fn host_read<const N: usize>(&self,index:usize,rom:&[u8],prior:Option<[u8;N]>)->Option<[u8;N]>{
        let fields:[_;N]=std::array::from_fn(|i|self.byte(index+i));let owned=fields.iter().filter(|v|v.is_some()).count();if owned==0{return prior;}
        self.host[0].fetch_add(1,Ordering::Relaxed);self.host[1].fetch_add(owned as u64,Ordering::Relaxed);
        Some(std::array::from_fn(|i|fields[i].map_or_else(||prior.as_ref().map_or_else(||rom[index+i],|v|v[i]),|v|v.0)))
    }
}
