//! Typed EUR sequences/tables: publication, original guest/native paths and MP2K shadow reads.
use super::*;
use gba_core::{capi::RT_API, mp2k::MemView, rom_data::audio_sequences::*, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUDIO_SEQUENCES_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 38] {
    [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
        SHORT_TEXTS_FILE,
        TEXT_CATALOGS_FILE,
        FONT_BANK_FILE,
        NAME_KEYBOARD_FILE,
        OBJ_STATIC_LISTS_FILE,
        OBJ_ANIMATION_BANKS_FILE,
        OBJ_CUSTOM_GRAPHICS_FILE,
        UI_RAW_PALETTES_FILE,
        UI_CUSTOM_GRAPHICS_FILE,
        ITEM_RECORDS_FILE,
        ITEM_LINKS_FILE,
        METADATA16_FILE,
        METADATA52_FILE,
        METADATA4_FILE,
        METADATA2_FILE,
        DEFAULT48_FILE,
        CONSTRUCTOR_ALTERNATES_FILE,
        METADATA14_FILE,
        METADATA12_FILE,
        GLOBAL_PAIR_FLAGS_FILE,
        EVALUATION_PARAMETERS_FILE,
        PROGRESSION_REWARDS_FILE,
        PROGRESSION_GATES_FILE,
        PROGRESSION_SELECTION_GATES_FILE,
        AGGREGATE_FLAG_KEYS_FILE,
        TRADE_PARAMETERS_FILE,
        ITEM_EFFECT_PARAMETERS_FILE,
        AUDIO_DIRECTORY_FILE,
        AUDIO_TONES_FILE,
        AUDIO_SEQUENCES_FILE,
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &AudioSequences, rom: Option<&[u8]>) {
    for (at, bytes, _) in data.source_spans().unwrap() {
        let at = at as usize;
        let dst = &mut m.bus.rom[at..at + bytes.len()];
        if let Some(r) = rom {
            dst.copy_from_slice(&r[at..at + bytes.len()]);
        } else {
            for b in dst {
                *b ^= 0x5a;
            }
        }
    }
}
#[test]
#[ignore = "private EUR e43 M4A sequence publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_audio_sequences_managed_real_owners_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..37]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e42").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (42, 41));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (43, 42)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 38);
    assert_eq!(source.manifest.resources.len(), 37);
    for i in 1..37 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_audio_sequences(&files[37]).unwrap();
    assert_eq!((data.instructions.len(), data.tracks.len()), (105253, 1610));
    assert_eq!(
        files[37],
        fs::read(std::env::var_os("FFTA_STATE_TEST_AUDIO_SEQUENCES_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[36].sources.len(), 808);
    assert_eq!(
        source.manifest.resources[36].dependencies,
        vec![AUDIO_DIRECTORY_ID, AUDIO_TONES_ID]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(AUDIO_SEQUENCES_FILE)
    {
        Err("injected audio tone publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e42-d41/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names().iter().zip(&old) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let prepared = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e43-d42"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(AUDIO_SEQUENCES_FILE), b"partial").unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    let relative = repaired
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = r.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    assert_eq!(
        preparation::prepare(&moved, &mut source, &mut |_| Ok(()))
            .unwrap()
            .directory,
        moved.join(relative)
    );
    for (n, b) in names().iter().zip(&old) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(prior_dir.join(n)).unwrap(), *b);
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let dir = parse_audio_directory(&files[35]).unwrap();
    let tones = parse_audio_tones(&files[36]).unwrap();
    data.validate_owners(&dir, &tones).unwrap();
    let param = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Parameter { .. }))
        .unwrap();
    let implicit = data
        .instructions
        .iter()
        .position(|i| !i.explicit && matches!(i.code, Code::Note { .. }))
        .unwrap();
    let jump = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Jump { .. }))
        .unwrap();
    let end = data
        .instructions
        .iter()
        .find(|i| matches!(i.code, Code::End))
        .unwrap()
        .id;
    let ext = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Extended { .. }))
        .unwrap();
    for case in 0..25 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        fs::write(dataset().join(AUDIO_DIRECTORY_FILE), &files[35]).unwrap();
        fs::write(dataset().join(AUDIO_TONES_FILE), &files[36]).unwrap();
        match case {
            0 => {
                if let Code::Parameter { value, .. } = &mut forged.instructions[param].code {
                    *value ^= 1;
                }
            }
            1 => {
                if let Code::Note { clock, .. } = &mut forged.instructions[implicit].code {
                    *clock = (*clock + 1) % 49;
                }
                forged.validate().unwrap();
                assert_eq!(forged.source_spans().unwrap(), data.source_spans().unwrap());
                assert!(forged.validate_owners(&dir, &tones).is_err());
            }
            2 => forged.instructions[jump].code = Code::Jump { target: end },
            3 => {
                let mut owner = dir.clone();
                owner
                    .headers
                    .iter_mut()
                    .find(|h| h.id == data.tracks[0].directory_header)
                    .unwrap()
                    .track_addresses[0] += 1;
                assert!(data.validate_owners(&owner, &tones).is_err());
                let bytes = encode_audio_directory(&owner).unwrap();
                manifest.resources[34].sha256 = gba_core::ffta_rom::digest(&bytes);
                fs::write(dataset().join(AUDIO_DIRECTORY_FILE), bytes).unwrap();
            }
            4 => {
                let mut owner = tones.clone();
                owner.banks[0].programs[0].note_keys[0] ^= 1;
                owner.validate_directory(&dir).unwrap();
                assert_eq!(owner.source_spans(), tones.source_spans());
                assert!(data.validate_owners(&dir, &owner).is_err());
                let bytes = encode_audio_tones(&owner).unwrap();
                manifest.resources[35].sha256 = gba_core::ffta_rom::digest(&bytes);
                fs::write(dataset().join(AUDIO_TONES_FILE), bytes).unwrap();
            }
            5 => forged.driver.clocks[0] ^= 1,
            6 => forged.driver.commands[0] ^= 2,
            7 => forged.driver.extended8_9[0] ^= 2,
            8 => forged.tracks[0].entry += 1,
            9 => forged.instructions[1].id = 0,
            10 => manifest.resources[36].sources[0].length -= 1,
            11 => manifest.resources[36].sources[807].offset += 1,
            12 => manifest.resources[36].sources[0].role = "audio/unknown".into(),
            13 => manifest.resources[36].sources.swap(0, 807),
            14 => {
                manifest.resources[36].sources.pop();
            }
            15 => manifest.resources[36].path = "../audio-sequences.json".into(),
            16 => manifest.resources[36].dependencies.clear(),
            17 => manifest.resources[36].dependencies = vec![AUDIO_TONES_ID.into()],
            18 => manifest.resources[36].sources[0].sha256 = "0".repeat(64),
            19 => manifest.resources[36].sha256 = "0".repeat(64),
            20 => manifest.coverage.retain(|id| id != AUDIO_SEQUENCES_ID),
            21 => {
                forged.driver.clocks.pop();
            }
            22 => {
                forged.instructions[ext].code = Code::Extended {
                    selector: 7,
                    value: 0,
                }
            }
            23 => {
                forged.instructions[implicit].code = Code::Note {
                    clock: 0,
                    key: Some(128),
                    velocity: None,
                    extension: None,
                }
            }
            _ => forged.instructions[1].source_offset += 1,
        }
        let bytes = encode_audio_sequences(&forged).unwrap();
        if case != 19 {
            manifest.resources[36].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(AUDIO_SEQUENCES_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "guard {case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(AUDIO_DIRECTORY_FILE), &files[35]).unwrap();
    fs::write(dataset().join(AUDIO_TONES_FILE), &files[36]).unwrap();
    fs::write(dataset().join(AUDIO_SEQUENCES_FILE), &files[37]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(dataset().join(AUDIO_SEQUENCES_FILE), r.join("missing.json")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(r.join("missing.json"), dataset().join(AUDIO_SEQUENCES_FILE)).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("copy");
    fs::create_dir(&copy).unwrap();
    for n in names() {
        fs::copy(dataset().join(n), copy.join(n)).unwrap();
    }
    attach_from_directory(&mut m, &copy).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(copy, temp.path().join("unavailable")).unwrap();
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    slots.undo(&mut m).unwrap();
    assert_eq!(m.save_state().unwrap(), changed);
    let len = m.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; len]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups
        .restore(backups.choices(len)[0].generation.unwrap(), &mut m, &boot)
        .unwrap();
    m.bus.load_save_data(&vec![9; len]);
    crate::save_backups::restart(&mut m, &boot, &save).unwrap();
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    range(&mut m, &data, None);
    for (at, bytes, _) in data.source_spans().unwrap() {
        for i in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let stats = owner.audio_sequences_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.opcode_bytes,
            stats.operand_bytes,
            stats.clock_bytes,
            stats.command_bytes,
            stats.extended_bytes
        ),
        (177248, 91663, 85384, 49, 144, 8)
    );
    assert_eq!(stats.instructions_used, 105253);
    assert_eq!(stats.clock_mask, (1u64 << 49) - 1);
    assert_eq!(stats.command_mask, (1u64 << 36) - 1);
    assert_eq!(stats.extended_mask, 3);
    let view = MemView {
        rom: &m.bus.rom,
        ewram: &m.bus.ewram,
        iwram: &m.bus.iwram,
        resources: Some(&owner),
    };
    for at in [
        data.instructions[0].source_offset,
        CLOCK_OFFSET,
        COMMANDS_OFFSET,
        EXTENDED_OFFSET,
    ] {
        assert_eq!(
            view.bytes::<4>(0x08000000 + at).unwrap(),
            rom[at as usize..at as usize + 4]
        );
    }
    let after = owner.audio_sequences_stats().unwrap();
    assert!(after.host_reads > 0 && after.host_source_bytes > 0);
    assert_eq!(after.source_bytes, stats.source_bytes);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,43,42],"files":38,"resources":37,"source_ranges":808,"source_bytes":177248,"all36prior_decoded_files_and_e42_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":25,"samebytes_inherited_metadata_and_actual_directory_tone_bank_metadata_rejected":true,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_instruction_table_masks_and_generic_host_after_restore":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

const PLAYER: u32 = 0x02010000;
const TRACK: u32 = 0x02010100;
const RETURN: u32 = 0x08000300;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
#[derive(Clone, Debug)]
enum Probe {
    Dispatch(u32, Option<u8>),
    Note(u32),
    Jump(u32),
    Pattern(u32, u8),
    Parameter(u32),
    Extended(u32),
    EndTie(u32),
    Wait(u8),
    Copy,
    Raw,
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Dispatch(..) => (0x081485ac, 0x081485c2),
            Self::Note(_) => (0x081487cc, 0x08148812),
            Self::Jump(_) => (0x08148330, RETURN),
            Self::Pattern(..) => (0x08148350, RETURN),
            Self::Parameter(_) | Self::Raw => (0x08148324, RETURN),
            Self::Extended(_) => (0x08149e88, RETURN),
            Self::EndTie(_) => (0x081489cc, RETURN),
            Self::Wait(_) => (0x081485fa, 0x08148604),
            Self::Copy => (0x081482f0, RETURN),
        }
    }
    fn setup(&self, m: &mut Machine, data: &AudioSequences) {
        m.bus.ewram.fill(0);
        m.bus.iwram.fill(0);
        m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
        m.cpu.regs = [0; 16];
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        let id = match self {
            Self::Dispatch(id, _)
            | Self::Note(id)
            | Self::Jump(id)
            | Self::Pattern(id, _)
            | Self::Parameter(id)
            | Self::Extended(id)
            | Self::EndTie(id) => Some(*id),
            _ => None,
        };
        if let Some(id) = id {
            let i = data.instruction(id).unwrap();
            put(
                &mut m.bus.ewram,
                0x10140,
                0x08000000 + i.source_offset + u32::from(i.explicit),
            );
        }
        match self {
            Self::Dispatch(id, live) => {
                let i = data.instruction(*id).unwrap();
                m.cpu.regs[5] = TRACK;
                m.bus.ewram[0x10107] = live.unwrap_or(if i.explicit {
                    0xe0
                } else {
                    i.code.command().unwrap()
                });
                put(&mut m.bus.ewram, 0x10140, 0x08000000 + i.source_offset);
            }
            Self::Note(id) => {
                m.cpu.regs[0] =
                    u32::from(data.instruction(*id).unwrap().code.command().unwrap() - 0xcf);
                m.cpu.regs[1] = PLAYER;
                m.cpu.regs[2] = TRACK;
                m.bus.ewram[0x10105] = 44;
                m.bus.ewram[0x10106] = 55;
            }
            Self::Jump(_) | Self::Parameter(_) | Self::Extended(_) | Self::EndTie(_) => {
                m.cpu.regs[1] = TRACK
            }
            Self::Pattern(_, depth) => {
                m.cpu.regs[1] = TRACK;
                m.bus.ewram[0x10102] = *depth;
            }
            Self::Wait(clock) => {
                m.cpu.regs[1] = 0x80 + u32::from(*clock);
                m.cpu.regs[5] = TRACK;
            }
            Self::Copy => m.cpu.regs[0] = 0x02018000,
            Self::Raw => {
                m.cpu.regs[1] = TRACK;
                put(&mut m.bus.ewram, 0x10140, 0x08000000 + CLOCK_OFFSET + 49);
            }
        }
    }
}
fn original(m: &mut Machine, p: &Probe, data: &AudioSequences, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m, data);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 2000, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
thread_local! {static COMPILED:Cell<[u64;5]>=const{Cell::new([0;5])};}
fn usage(m: *mut std::ffi::c_void) -> [u64; 5] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .audio_sequences_stats()
            .unwrap()
    };
    [
        s.opcode_bytes,
        s.operand_bytes,
        s.clock_bytes,
        s.command_bytes,
        s.extended_bytes,
    ]
}
fn count(b: [u64; 5], a: [u64; 5]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..5 {
            x[i] += a[i] - b[i];
        }
        v.set(x);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read8)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read16)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read32)(m, a);
    count(b, usage(m));
    v
}
fn kind(c: &Code) -> u8 {
    match c {
        Code::Wait { .. } => 0,
        Code::End => 1,
        Code::Jump { .. } => 2,
        Code::Pattern { .. } => 3,
        Code::Return => 4,
        Code::Parameter { .. } => 5,
        Code::Extended { .. } => 6,
        Code::EndTie { .. } => 7,
        Code::Note { .. } => 8,
    }
}
#[test]
#[ignore = "private EUR e43 typed sequence original/native/bus/host boundaries; Wine only, Linux/USA pending"]
fn european_audio_sequences_native_live_running_and_typed_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.audio_sequences.as_ref().unwrap();
    let mut cases = Vec::new();
    let mut prefixes = BTreeSet::new();
    let mut notes = BTreeSet::new();
    let mut extended = BTreeSet::new();
    let mut ties = BTreeSet::new();
    for i in &data.instructions {
        if prefixes.insert((kind(&i.code), i.explicit)) {
            cases.push(Probe::Dispatch(i.id, None));
        }
        if matches!(i.code, Code::Note { .. }) && notes.insert((i.code.operand_len(), i.explicit)) {
            cases.push(Probe::Note(i.id));
        }
        if let Code::Extended { selector, .. } = i.code {
            if extended.insert((selector, i.explicit)) {
                cases.push(Probe::Extended(i.id));
            }
        }
        if let Code::EndTie { key } = i.code {
            if ties.insert((key.is_some(), i.explicit)) {
                cases.push(Probe::EndTie(i.id));
            }
        }
    }
    let implicit = data
        .instructions
        .iter()
        .find(|i| !i.explicit && matches!(i.code, Code::Note { .. }))
        .unwrap();
    cases.push(Probe::Dispatch(implicit.id, Some(0xbd))); // Live RAM controls interpretation, not source-graph metadata.
    let jump = data
        .instructions
        .iter()
        .find(|i| matches!(i.code, Code::Jump { .. }))
        .unwrap();
    cases.push(Probe::Jump(jump.id));
    let pattern = data
        .instructions
        .iter()
        .find(|i| matches!(i.code, Code::Pattern { .. }))
        .unwrap();
    for depth in [0, 2] {
        cases.push(Probe::Pattern(pattern.id, depth));
    }
    let params: Vec<_> = data
        .instructions
        .iter()
        .filter_map(|i| {
            if let Code::Parameter { value, .. } = i.code {
                Some((value, i.id))
            } else {
                None
            }
        })
        .collect();
    cases.push(Probe::Parameter(params.iter().min().unwrap().1));
    cases.push(Probe::Parameter(params.iter().max().unwrap().1));
    for clock in [0, 24, 48] {
        cases.push(Probe::Wait(clock));
    }
    cases.push(Probe::Copy);
    cases.push(Probe::Raw);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, data, &mut pcs);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&addr| {
            let mut instrs = vec![armv4t::decode_thumb(view.read16(addr), addr)];
            let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                instrs.push(next);
            }
            Block {
                addr,
                thumb: true,
                instrs,
            }
        })
        .collect();
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let object = out.join("audio-sequences.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for m in [&mut decoded, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 5]));
    let mut dispatches = 0;
    let raw = (CLOCK_OFFSET + 49) as usize;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
            if matches!(p, Probe::Raw) {
                m.bus.rom[raw] ^= 0x5a;
            }
        }
        original(&mut legacy, p, data, &mut BTreeSet::new());
        let db = owner.audio_sequences_stats().unwrap().source_bytes;
        let nb = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_sequences_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        original(&mut decoded, p, data, &mut BTreeSet::new());
        p.setup(&mut native, data);
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 2000, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("observed original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        let d = owner.audio_sequences_stats().unwrap().source_bytes - db;
        let n = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_sequences_stats()
            .unwrap()
            .source_bytes
            - nb;
        assert_eq!(d, n, "{p:?}");
        if matches!(p, Probe::Raw) {
            assert_eq!(d, 0);
        } else {
            assert!(d > 0, "{p:?}");
        }
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.bus.rom[raw] = rom[raw];
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            assert!(
                m.save_state().unwrap() == expected,
                "full state/clock {p:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|v| *v > 0), "{compiled:?}");
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans().unwrap() {
            let end = at + bytes.len() as u32;
            for offset in (at - 3..at + 4).chain(end - 3..end + 4) {
                for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                    assert_eq!(decoded.bus.read8(mirror + offset), rom[offset as usize]);
                    assert_eq!(
                        decoded.bus.read16(mirror + offset),
                        u16::from_le_bytes(
                            rom[offset as usize..offset as usize + 2]
                                .try_into()
                                .unwrap()
                        )
                    );
                    assert_eq!(
                        (RT_API.read32)(
                            &mut decoded as *mut Machine as *mut std::ffi::c_void,
                            mirror + offset
                        ),
                        u32::from_le_bytes(
                            rom[offset as usize..offset as usize + 4]
                                .try_into()
                                .unwrap()
                        )
                    );
                }
            }
        }
    }
    decoded.bus.real_bios = false;
    // A word crosses the e42 final PCM endpoint into the first sequence: two owners, no raw fallback.
    let first = data.instructions[0].source_offset as usize;
    for v in &mut decoded.bus.rom[first - 2..first] {
        *v ^= 0x5a;
    }
    let s = owner.audio_sequences_stats().unwrap();
    let t = owner.audio_tones_stats().unwrap();
    assert_eq!(
        decoded.bus.read32(0x08000000 + first as u32 - 2),
        u32::from_le_bytes(rom[first - 2..first + 2].try_into().unwrap())
    );
    assert_eq!(
        owner.audio_sequences_stats().unwrap().source_bytes - s.source_bytes,
        2
    );
    assert_eq!(
        owner.audio_tones_stats().unwrap().source_bytes - t.source_bytes,
        2
    );
    let s = owner.audio_sequences_stats().unwrap();
    let t = owner.audio_tones_stats().unwrap();
    let view = MemView {
        rom: &decoded.bus.rom,
        ewram: &decoded.bus.ewram,
        iwram: &decoded.bus.iwram,
        resources: Some(&owner),
    };
    assert_eq!(
        view.bytes::<4>(0x08000000 + first as u32 - 2).unwrap(),
        rom[first - 2..first + 2]
    );
    assert_eq!(
        owner.audio_sequences_stats().unwrap().host_source_bytes - s.host_source_bytes,
        2
    );
    assert_eq!(
        owner.audio_tones_stats().unwrap().host_source_bytes - t.host_source_bytes,
        2
    );
    decoded.bus.rom[first - 2..first].copy_from_slice(&rom[first - 2..first]);
    let before = owner.audio_sequences_stats().unwrap();
    decoded.bus.rom[raw] ^= 0x5a;
    assert_eq!(decoded.bus.read8(0x08000000 + raw as u32), rom[raw] ^ 0x5a);
    let view = MemView {
        rom: &decoded.bus.rom,
        ewram: &decoded.bus.ewram,
        iwram: &decoded.bus.iwram,
        resources: Some(&owner),
    };
    assert_eq!(view.u8(0x08000000 + raw as u32), Some(rom[raw] ^ 0x5a));
    assert_eq!(owner.audio_sequences_stats().unwrap(), before);
    decoded.bus.rom[raw] = rom[raw];
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, first as u32 - 2, 4),
        (0xdf0c, CLOCK_OFFSET, 8),
        (0xdf0b, CLOCK_OFFSET + 47, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        for v in &mut decoded.bus.rom[first - 2..first] {
            *v ^= 0x5a;
        }
        let before = owner.audio_sequences_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.audio_sequences_stats().unwrap().source_bytes > before);
        range(&mut decoded, data, Some(&rom));
        decoded.bus.rom[first - 2..first].copy_from_slice(&rom[first - 2..first]);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + EXTENDED_OFFSET + 6);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.audio_sequences_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.audio_sequences_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,43,42],"original_interpreter_native_paths":cases.len(),"compiled_opcode_operand_clock_command_extended_bytes":compiled,"poisoned_sequence_source_bytes":177248,"full_states_clocks_equal_at_declared_checkpoints":true,"live_running_override_preserved":true,"unowned_raw_operand_native_and_bus_host_fallback":true,"all808spans_edges_widths_mirrors_realbios":true,"cpu_cpuset_fastset_dma_and_two_tone_two_sequence_guest_host_boundary":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"source_checkpoints":"dispatch/note/wait internal checkpoints; narrow branch/operand/CD/CE/copy returns, not whole sequencer or mixer ABI","host_reads":"generic typed view only, no host sequencer","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
