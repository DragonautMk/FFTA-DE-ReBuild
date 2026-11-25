//! Verify the complete Color Mode cursor table through ordinary EUR title input.

use super::super::{
    attach_from_directory, cmd, encode_layered_obj_animations, parse, parse_layered_obj_animations,
    Manifest, LAYERED_OBJ_ANIMATIONS_FILE, LAYERED_OBJ_ANIMATIONS_ID, MANIFEST_FILE,
};
use super::{input, reach_guest_pc};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        layered_obj_animations::TITLE_COLOR_MODE_POSITION_ROOT as ROOT,
        read_audit::{Service, TraceCapture, TraceFilter},
        SourceRange,
    },
    Bus, Machine,
};
use std::{collections::BTreeMap, fs, sync::Arc};

const RELEASED: u16 = 0x3ff;
const ACCEPT: u16 = RELEASED & !1;
const DOWN: u16 = RELEASED & !0x80;
const STORED_COLOR_MODE: usize = 0x2fc2;
const CURSOR_READERS: [u32; 2] = [0x0813_a906, 0x0813_a916];

fn reach_color_mode(machine: &mut Machine) {
    reach_guest_pc(machine, 0x0813_a766, 60, None, |_| RELEASED);
    let title_frame = machine.bus.frames;
    let context = machine.bus.read32(0x0200_f3b0);
    let selected = machine.bus.read8(context + 0x8962);
    assert!(selected < 4);
    let down_presses = u64::from((2 + 4 - selected) % 4);
    reach_guest_pc(machine, 0x0813_a830, 64, None, |frame| {
        let elapsed = frame - title_frame;
        let confirm = 4 + 8 * down_presses;
        if (confirm..confirm + 2).contains(&elapsed) {
            ACCEPT
        } else if (4..confirm).contains(&elapsed) && (elapsed - 4) % 8 < 2 {
            DOWN
        } else {
            RELEASED
        }
    });
}

fn confirm_color_mode(machine: &mut Machine, service: Service) -> BTreeMap<(u32, u32), u64> {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(ROOT, ROOT + 12)],
        },
        16,
    )
    .unwrap();
    let initial_mode = machine.bus.ewram[STORED_COLOR_MODE] & 3;
    assert!(initial_mode < 3);
    let start = machine.bus.frames;
    for frame in 0..28 {
        capture.frame(Some(machine.bus.frames));
        machine.bus.keys = match frame {
            6..8 | 14..16 => DOWN,
            22..24 => ACCEPT,
            _ => RELEASED,
        };
        crate::run_frame_interpreter_observed(machine, crate::FRAME_STEP_GUARD, |_| {});
        assert_eq!(machine.bus.frames, start + frame + 1);
    }
    assert_eq!(
        machine.bus.ewram[STORED_COLOR_MODE] & 3,
        (initial_mode + 2) % 3
    );
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut reads = BTreeMap::new();
    for event in report.events {
        let site = event
            .key
            .instruction
            .expect("Original Color Mode cursor instruction");
        let offset = event.key.read.physical_offset;
        assert_eq!(event.key.service, service);
        assert_eq!(event.key.read.length, 2);
        assert!(site.thumb && offset % 2 == 0 && event.requests > 0);
        assert_eq!(site.pc, CURSOR_READERS[((offset - ROOT) % 4 / 2) as usize]);
        assert!(reads.insert((offset, site.pc), event.requests).is_none());
    }
    assert_eq!(reads.len(), 6);
    reads
}

#[test]
#[ignore = "exact EUR ROM, E129 title checkpoint and E532 predecessor manifest"]
fn european_title_color_mode_cursor_uses_extracted_positions() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_TITLE_COLOR_MODE_WORK");
    let previous_bytes = fs::read(input("FFTA_TITLE_COLOR_MODE_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E532 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (327, 326)
    );

    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e328-d327");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let obj_path = dataset.join(LAYERED_OBJ_ANIMATIONS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let obj_bytes = fs::read(&obj_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E533 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == LAYERED_OBJ_ANIMATIONS_ID)
        .unwrap();
    let bank = ROOT as usize..(ROOT + 12) as usize;
    let mut expected = previous.clone();
    expected.extractor_revision = 328;
    expected.decoder_revision = 327;
    let entry = &mut expected.resources[index];
    entry.sha256 = digest(&obj_bytes);
    let next = entry.sources.partition_point(|source| source.offset < ROOT);
    assert_eq!(
        entry.sources[next - 1].offset + entry.sources[next - 1].length,
        ROOT
    );
    assert_eq!(entry.sources[next].offset, ROOT + 12);
    entry.sources.insert(
        next,
        SourceRange {
            offset: ROOT,
            length: 12,
            sha256: digest(&rom[bank.clone()]),
            role: "title-color-mode-positions".into(),
        },
    );
    assert_eq!(manifest, expected);

    let mut layered = parse_layered_obj_animations(&obj_bytes).unwrap();
    assert!(layered.title_color_mode_positions.take().is_some());
    let old_obj = encode_layered_obj_animations(&layered).unwrap();
    assert_eq!(digest(&old_obj), previous.resources[index].sha256);
    let mut managed = crate::make_machine(rom.clone(), None);
    fs::write(&obj_path, old_obj).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let old_owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 328;
    incomplete.decoder_revision = 327;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &old_owner,
        managed.bus.rom_resources().unwrap()
    ));
    fs::write(&obj_path, obj_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();

    let mut raw = crate::make_machine(rom, None);
    let title = fs::read(input("FFTA_TITLE_COLOR_MODE_START")).unwrap();
    let title_sha = digest(&title);
    assert_eq!(
        title_sha,
        "fefd6a26720c4574d0ccb2b93efe1c6ffc2353248e931a64fb9dc10539dd1e32"
    );
    raw.load_state(&title).unwrap();
    reach_color_mode(&mut raw);
    let entry_frame = raw.bus.frames;
    let initial_mode = raw.bus.ewram[STORED_COLOR_MODE] & 3;
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    for byte in &mut managed.bus.rom[bank.clone()] {
        *byte ^= 0xff;
    }
    let raw_reads = confirm_color_mode(&mut raw, Service::Raw);
    assert_eq!(raw_reads, confirm_color_mode(&mut managed, Service::Typed));
    let reads: u64 = raw_reads.values().sum();
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .layered_obj_animations_stats()
        .unwrap();
    assert_eq!(stats.title_color_mode_position_reads, reads);
    assert_eq!(stats.title_color_mode_position_bytes, 2 * reads);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
    managed.bus.rom[bank.clone()].copy_from_slice(&raw.bus.rom[bank]);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "revision": [328, 327],
            "manifest_sha256": digest(&manifest_bytes),
            "source_bytes": 12,
            "position_pairs": 3,
            "title_state_sha256": title_sha,
            "ordinary_cursor_reads": reads,
            "entry_frame": entry_frame,
            "initial_mode": initial_mode,
            "confirmed_mode": raw.bus.ewram[STORED_COLOR_MODE] & 3,
            "predecessor_attach": true,
            "missing_group_rejected": true,
            "full_state_video_and_audio_equal": true
        }))
        .unwrap(),
    )
    .unwrap();
}
