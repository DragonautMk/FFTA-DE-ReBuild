//! Private source-only M4A oracle. No sequencing/mixer replacement or managed activation.
use super::*;
use gba_core::{rom_data::audio_directory::*, Machine};
use std::collections::BTreeSet;

const MAGIC: u32 = 0x68736d53;
const SOUND_INFO: usize = 0x84e0;
pub(super) const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;

#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Live {
    pub capacity: Option<u8>,
    pub reject_priority: bool,
    pub locked: bool,
}
fn put(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn setup(m: &mut Machine, d: &AudioDirectory, argument: u32, live: Live) {
    m.bus.ewram.fill(0);
    m.bus.iwram.fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    put(&mut m.bus.iwram, 0x7ff0, 0x02000000 + SOUND_INFO as u32);
    put(&mut m.bus.ewram, SOUND_INFO, MAGIC);
    for p in &d.players {
        let at = (p.player_address - 0x02000000) as usize;
        m.bus.ewram[at + 8] = p.track_capacity;
        m.bus.ewram[at + 11] = p.priority_gate as u8;
        put(&mut m.bus.ewram, at + 0x2c, p.tracks_address);
        put(&mut m.bus.ewram, at + 0x34, MAGIC);
    }
    if let Some((_, p, _)) = d.selected(argument) {
        let at = (p.player_address - 0x02000000) as usize;
        if let Some(cap) = live.capacity {
            m.bus.ewram[at + 8] = cap;
        }
        if live.reject_priority {
            put(&mut m.bus.ewram, at, 0x08150940);
            put(&mut m.bus.ewram, at + 4, 1);
            m.bus.ewram[at + 9] = 255;
            m.bus.ewram[at + 11] = 1;
        }
        if live.locked {
            put(&mut m.bus.ewram, at + 0x34, MAGIC + 1);
        }
    }
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[0] = argument;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08148b84;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn run(
    m: &mut Machine,
    d: &AudioDirectory,
    argument: u32,
    live: Live,
    visited: &mut BTreeSet<u32>,
    reads: &mut BTreeSet<u32>,
) -> usize {
    setup(m, d, argument, live);
    let (s, p, h) = d.selected(argument).unwrap();
    let at = (p.player_address - 0x02000000) as usize;
    let tracks = (p.tracks_address - 0x03000000) as usize;
    let capacity = live.capacity.unwrap_or(p.track_capacity);
    let reject = live.locked || (live.reject_priority && h.priority < 255);
    let mut ewram = m.bus.ewram.to_vec();
    let mut iwram = m.bus.iwram.to_vec();
    let saved = m.cpu.regs;
    if !reject {
        put(&mut ewram, at, s.header_address);
        put(&mut ewram, at + 4, 0);
        ewram[at + 9] = h.priority;
        put(&mut ewram, at + 0x30, h.tone_bank_address);
        ewram[at + 0xc..at + 0x10].fill(0);
        ewram[at + 0x1c..at + 0x22].copy_from_slice(&[150, 0, 0, 1, 150, 0]);
        ewram[at + 0x22..at + 0x26].fill(0);
        for (i, ptr) in h
            .track_addresses
            .iter()
            .take(usize::from(capacity))
            .enumerate()
        {
            let off = tracks + i * 80;
            iwram[off] = 0xc0;
            // r8 is captured as zero before the loop; the backedge skips that capture.
            put(&mut iwram, off + 0x20, 0);
            put(&mut iwram, off + 0x40, *ptr);
        }
        if h.reverb & 0x80 != 0 {
            ewram[SOUND_INFO + 5] = h.reverb & 0x7f;
        }
    }
    let mut count = 0;
    let mut started = false;
    while m.cpu.regs[15] != RETURN {
        assert!(count < 4000, "{argument} {live:?} pc{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert(pc);
        // Observe original loads before executing them; no extra bus reads.
        let load = match pc {
            0x08148b90 => Some((m.cpu.regs[0] + 4, 2)),
            0x08148b9a => Some((m.cpu.regs[1], 4)),
            0x08148b9c => Some((m.cpu.regs[0], 4)),
            0x08149284 | 0x081492a8 => Some((m.cpu.regs[7] + 2, 1)),
            0x081492be => Some((m.cpu.regs[7] + 4, 4)),
            0x081492d8 | 0x08149306 => Some((m.cpu.regs[7], 1)),
            0x081492fe => Some((m.cpu.regs[0], 4)),
            0x08149334 | 0x0814933c => Some((m.cpu.regs[7] + 3, 1)),
            _ => None,
        };
        if let Some((address, width)) = load {
            for i in 0..width {
                reads.insert(address - 0x08000000 + i);
            }
        }
        if pc == 0x08149270 {
            assert!(!started);
            assert_eq!(
                (m.cpu.regs[0], m.cpu.regs[1]),
                (p.player_address, s.header_address)
            );
            started = true;
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert!(started);
    assert_eq!(
        m.bus.ewram.iter().zip(&ewram).position(|(a, b)| a != b),
        None,
        "complete EWRAM {argument} {live:?}"
    );
    // All IWRAM except actual call-stack scratch must match the independently modeled writes.
    assert_eq!(
        m.bus.iwram[..0x7c00]
            .iter()
            .zip(&iwram[..0x7c00])
            .position(|(a, b)| a != b),
        None,
        "IWRAM tracks {argument} {live:?}"
    );
    assert_eq!(
        m.bus.iwram[0x7e00..]
            .iter()
            .zip(&iwram[0x7e00..])
            .position(|(a, b)| a != b),
        None,
        "IWRAM tail {argument} {live:?}"
    );
    assert!(m.bus.iwram[0x7c00..0x7d00].iter().all(|v| *v == 0x5a));
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], saved[i], "callee r{i}");
    }
    count
}
fn set_source(m: &mut Machine, d: &AudioDirectory) {
    for (at, bytes) in d.source_spans() {
        m.bus.rom[at as usize..at as usize + bytes.len()].copy_from_slice(&bytes);
    }
}

#[test]
#[ignore = "private EUR M4A directory/header oracle; Wine only, Linux/USA pending"]
fn european_m4a_directory_matches_original_selection_headers_aliases_and_live_limits() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUDIO_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: AudioDirectory = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_AUDIO_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(), 457);
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 15236);
    for ((at, bytes), source) in data.source_spans().iter().zip(&catalog.sources) {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        assert_eq!(source.sha256, gba_core::ffta_rom::digest(bytes));
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<AudioDirectory>(&encoded).unwrap(),
        *data
    );
    let shape = serde_json::to_value(data).unwrap();
    let mut unknown = shape.clone();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<AudioDirectory>(unknown).is_err());
    let mut guards = 1;
    for edit in 0..14 {
        let mut bad = data.clone();
        match edit {
            0 => {
                bad.players.pop();
            }
            1 => bad.players[0].id = 1,
            2 => bad.players[0].source_offset += 4,
            3 => bad.players[0].player_address = 0x03000000,
            4 => bad.players[0].tracks_address = 0x03007ffc,
            5 => bad.players[0].track_capacity = 17,
            6 => {
                bad.songs.pop();
            }
            7 => bad.songs[0].id = 1,
            8 => bad.songs[0].player_index = 3,
            9 => bad.songs[0].header_id = 1,
            10 => bad.songs[0].header_address += 4,
            11 => {
                bad.headers.pop();
            }
            12 => bad.headers[0].source_offset += 4,
            _ => bad.headers[0].track_count = 1,
        }
        assert!(bad.validate().is_err(), "guard {edit}");
        guards += 1;
    }
    let mut changed = data.clone();
    changed.songs[0].opaque06 ^= 1;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.headers[0].tone_bank_address ^= 4;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&[]).is_err());
    assert!(data.selected(640).is_none());
    assert!(data.selected(65535).is_none());
    assert!(data.selected(u32::MAX).is_none());
    let mut m = Machine::new(rom.clone());
    let mut visited = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    let mut cases = 0;
    for id in 0..640 {
        instructions += run(&mut m, data, id, Live::default(), &mut visited, &mut reads);
        cases += 1;
    }
    for argument in [0x10000, 0xffff0001, 0x1234027f] {
        instructions += run(
            &mut m,
            data,
            argument,
            Live::default(),
            &mut visited,
            &mut reads,
        );
        cases += 1;
    }
    // Changed live inputs are explicit targeted setup, not an ordinary gameplay checkpoint.
    for capacity in [0, 1, 6, 16] {
        instructions += run(
            &mut m,
            data,
            0,
            Live {
                capacity: Some(capacity),
                ..Live::default()
            },
            &mut visited,
            &mut reads,
        );
        cases += 1;
    }
    for live in [
        Live {
            reject_priority: true,
            ..Live::default()
        },
        Live {
            locked: true,
            ..Live::default()
        },
    ] {
        instructions += run(&mut m, data, 0, live, &mut visited, &mut reads);
        cases += 1;
    }
    // Stop an out-of-catalog raw lookup at the actual start entry, before dereferencing its raw null header.
    setup(&mut m, data, 640, Live::default());
    let mut external = BTreeSet::new();
    for _ in 0..100 {
        let pc = m.cpu.regs[15];
        if pc == 0x08149270 {
            break;
        }
        if pc == 0x08148b90 {
            external.insert(m.cpu.regs[0] + 4);
        }
        if pc == 0x08148b9c {
            external.insert(m.cpu.regs[0]);
        }
        m.step();
    }
    assert_eq!(m.cpu.regs[15], 0x08149270);
    assert_eq!(
        (m.cpu.regs[0], m.cpu.regs[1]),
        (data.players[0].player_address, 0)
    );
    assert_eq!(external, BTreeSet::from([0x08150940, 0x08150944]));
    let real_reads = reads.clone();
    let mut synthetic = 0;
    for (priority, reverb) in [(0, 0), (1, 0x80), (255, 0xff)] {
        let mut edited = data.clone();
        let id = edited.songs[0].header_id;
        let h = edited.headers.iter_mut().find(|h| h.id == id).unwrap();
        h.priority = priority;
        h.reverb = reverb;
        h.opaque01 = 0xa5;
        edited.players[0].opaque09 = 0x5a;
        edited.players[0].priority_gate |= 0xab00;
        edited.songs[0].opaque06 = 0xffff;
        edited.validate().unwrap();
        set_source(&mut m, &edited);
        for live in [
            Live::default(),
            Live {
                reject_priority: true,
                ..Live::default()
            },
        ] {
            instructions += run(&mut m, &edited, 0, live, &mut visited, &mut reads);
            synthetic += 1;
        }
    }
    set_source(&mut m, data);
    assert_eq!(m.bus.rom, rom);
    // Observed source reads deliberately exclude preserved opaque/padding bytes.
    assert_eq!(
        real_reads
            .iter()
            .filter(|a| (DIRECTORY_OFFSET..DIRECTORY_END).contains(a))
            .count(),
        640 * 6
    );
    assert_eq!(
        real_reads
            .iter()
            .filter(|a| (PLAYER_OFFSET..DIRECTORY_OFFSET).contains(a))
            .count(),
        12
    );
    for h in &data.headers {
        assert!(!real_reads.contains(&(h.source_offset + 1)));
        for i in 0..usize::from(h.track_count) * 4 {
            assert!(real_reads.contains(&(h.source_offset + 8 + i as u32)));
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("audio-directory.json"), encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "players":3,"songs":640,"unique_headers":455,"source_bytes":15236,"source_ranges":457,
        "real_and_live_input_cases":cases,"external_raw_lookup_probes":1,"synthetic_cases":synthetic,
        "instructions_coverage":instructions,"pcs":visited.len(),"original_distinct_read_bytes":real_reads.len(),
        "all_source_roundtrip":true,"strict_json_link_shape_source_guards":guards,
        "all640_original_selection_start_full_ewram_iwram_tracks_sp_callee_guards":true,
        "dummy_header_wave_alias_low16_live_min_capacity_priority_lock_reverb":true,
        "unused_fields":"preserved; not falsely counted as original reads",
        "synthetic":"unauthenticated/unpublished targeted source/input edits; no ordinary claim",
        "external_raw_lookup_addresses":external,"managed":"EUR e40-d39 unchanged",
        "bank_sample_sequences":"roots only; extents unproved","linux":"pending","usa":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
