//! Check the complete original VRAM LZ77 bank with its ROM backing altered.
use super::{
    Extraction, MANIFEST_FILE, Manifest, OBSERVED_LZ_FILE, OBSERVED_LZ_ID, attach_from_directory,
    cmd, encode_observed_lz, parse, parse_observed_lz,
};
use gba_core::{Bus, Machine, ffta_rom::digest};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const DESTINATIONS: [u32; 3] = [VRAM_BASE, VRAM_BASE + 0x6000, VRAM_BASE + 0x8000];
const INITIALIZE: u32 = BASE + 0x039d4e;
const FINISH: u32 = BASE + 0x039d9a;
const DESTINATION_POINTER: u32 = 0x0300_2810;
const VIDEO_STATE: u32 = 0x0201_0000;
const PADDING: usize = 0xb5300e;

#[derive(Deserialize)]
struct Plan {
    streams: BTreeMap<String, StreamWitness>,
    vram_literal_rows: Vec<OriginalCall>,
}

#[derive(Deserialize)]
struct StreamWitness {
    encoded_length: u32,
    decoded_length: usize,
    source_sha256: String,
    decoded_sha256: String,
}

#[derive(Deserialize)]
struct OriginalCall {
    source: u32,
    literal: u32,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_original_calls(machine: &mut Machine) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.write32(DESTINATION_POINTER, VIDEO_STATE);
    machine.bus.write32(VIDEO_STATE + 0x8c, DESTINATIONS[1]);
    machine.bus.write32(VIDEO_STATE + 0x88, DESTINATIONS[2]);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[15] = INITIALIZE;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    for _ in 0..160 {
        if machine.cpu.regs[15] == FINISH {
            return;
        }
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!(
        "VRAM initialization escaped at {:08x}",
        machine.cpu.regs[15]
    );
}

#[test]
#[ignore = "exact EUR ROM, E509 source plan and E507 manifest"]
fn european_vram_lz_bank_publication_and_original_transfers() {
    let work = input("FFTA_VRAM_LZ_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_VRAM_LZ_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_VRAM_LZ_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E507 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (316, 315)
    );

    let resources = tempfile::tempdir_in(&work).unwrap();
    let rom_arg = rom_path.to_str().unwrap().to_owned();
    let prepare = [
        "prepare".into(),
        rom_arg.clone(),
        resources.path().to_str().unwrap().into(),
    ];
    cmd(&prepare).unwrap();
    let revision = resources
        .path()
        .join(&previous.source_rom_sha256)
        .join("v1/e317-d316");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    cmd(&prepare).unwrap();
    assert_eq!(
        fs::read_to_string(revision.join("current")).unwrap(),
        current
    );
    cmd(&["verify".into(), rom_arg, dataset.to_str().unwrap().into()]).unwrap();

    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E510 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (317, 316)
    );
    assert_eq!(manifest.coverage, previous.coverage);
    assert_eq!(manifest.resources.len(), previous.resources.len());
    let index = manifest
        .resources
        .iter()
        .position(|row| row.id == OBSERVED_LZ_ID)
        .unwrap();
    for (i, (current, prior)) in manifest
        .resources
        .iter()
        .zip(&previous.resources)
        .enumerate()
    {
        if i != index {
            assert_eq!(current, prior);
        }
    }
    let old = &previous.resources[index];
    let new = &manifest.resources[index];
    assert_eq!(
        (&new.path, &new.format, &new.dependencies),
        (&old.path, &old.format, &old.dependencies)
    );
    assert!(new.sources.starts_with(&old.sources));
    let added = &new.sources[old.sources.len()..];
    assert_eq!(added.len(), 6);
    assert_eq!(added.iter().map(|row| row.length).sum::<u32>(), 15_476);
    for span in added {
        let start = span.offset as usize;
        let end = start + span.length as usize;
        assert_eq!(digest(&rom[start..end]), span.sha256);
        assert!(
            previous
                .resources
                .iter()
                .flat_map(|r| &r.sources)
                .all(|prior| {
                    prior.offset + prior.length <= span.offset
                        || span.offset + span.length <= prior.offset
                })
        );
    }

    let packet_path = dataset.join(OBSERVED_LZ_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    assert_eq!(packet_bytes.len(), 1_199_367);
    let mut packet = parse_observed_lz(&packet_bytes).unwrap();
    let bank = packet.vram_bank.take().unwrap();
    let previous_catalog = Extraction::from_rom_for_manifest(&rom, &previous).unwrap();
    assert_eq!(packet, previous_catalog.observed_lz.unwrap().decoded);
    let previous_packet = encode_observed_lz(&packet).unwrap();
    assert_eq!(digest(&previous_packet), old.sha256);
    assert_eq!(plan.vram_literal_rows.len(), bank.blocks.len());
    for (i, call) in plan.vram_literal_rows.iter().enumerate() {
        let stream = &plan.streams[&call.source.to_string()];
        let block = &bank.blocks[i];
        assert_eq!(block.source_offset, call.source);
        assert_eq!(block.encoded_source.len(), stream.encoded_length as usize);
        assert_eq!(block.decoded.len(), stream.decoded_length);
        assert_eq!(digest(&block.encoded_source), stream.source_sha256);
        assert_eq!(digest(&block.decoded), stream.decoded_sha256);
        assert_eq!(
            u32::from_le_bytes(bank.source_literals[i]),
            BASE + call.source
        );
        assert!(
            added
                .iter()
                .any(|span| span.offset == call.literal && span.length == 4)
        );
        assert!(
            added
                .iter()
                .any(|span| span.offset == call.source && span.length == stream.encoded_length)
        );
    }

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, previous_bytes).unwrap();
    fs::write(&packet_path, &previous_packet).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let first = added[0].offset as usize;
    managed.bus.rom[first] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + first as u32), !rom[first]);
    managed.bus.rom[first] ^= 0xff;
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let mut missing_bank = manifest.clone();
    missing_bank.resources[index] = old.clone();
    fs::write(
        &manifest_path,
        toml::to_string_pretty(&missing_bank).unwrap(),
    )
    .unwrap();
    fs::write(&packet_path, previous_packet).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();

    let poison = |machine: &mut Machine| {
        for span in added {
            let start = span.offset as usize;
            for byte in &mut machine.bus.rom[start..start + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    let mut raw = Machine::new(rom);
    poison(&mut managed);
    let before = owner.observed_lz_stats().unwrap();
    for span in added {
        for at in span.offset..span.offset + span.length {
            assert_eq!(
                managed.bus.read8(BASE + at),
                raw.bus.rom[at as usize],
                "{at:06x}"
            );
        }
    }
    let after = owner.observed_lz_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 15_476);
    let expected_mixed = [
        raw.bus.rom[PADDING - 2],
        raw.bus.rom[PADDING - 1],
        !raw.bus.rom[PADDING],
        !raw.bus.rom[PADDING + 1],
    ];
    managed.bus.rom[PADDING] ^= 0xff;
    managed.bus.rom[PADDING + 1] ^= 0xff;
    for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
        assert_eq!(
            managed
                .bus
                .read32(mirror + PADDING as u32 - 2)
                .to_le_bytes(),
            expected_mixed
        );
    }
    managed.bus.rom[PADDING] ^= 0xff;
    managed.bus.rom[PADDING + 1] ^= 0xff;
    let before_transfers = owner.observed_lz_stats().unwrap();

    // Start both guest paths from the same state after probing the source views.
    poison(&mut managed);
    let initial = raw.save_state().unwrap();
    managed.load_state(&initial).unwrap();
    poison(&mut managed);
    run_original_calls(&mut raw);
    run_original_calls(&mut managed);
    let after_transfers = owner.observed_lz_stats().unwrap();
    assert_eq!(after_transfers.vram_hle_calls_by_block, [1; 3]);
    assert_eq!(after_transfers.hle_calls - before_transfers.hle_calls, 3);
    assert_eq!(
        after_transfers.source_bytes - before_transfers.source_bytes,
        12
    );
    for (block, address) in bank.blocks.iter().zip(DESTINATIONS) {
        let start = (address - VRAM_BASE) as usize;
        let end = start + block.decoded.len();
        assert_eq!(&raw.bus.vram[start..end], block.decoded);
        assert_eq!(&managed.bus.vram[start..end], block.decoded);
    }
    poison(&mut managed);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let result = serde_json::json!({
        "status": "PASS", "revision": [317, 316], "spans": added.len(), "source_bytes": 15476,
        "decoded_bytes": 28672, "original_vram_calls": 3, "source_literal_bytes": 12,
        "predecessor_attach": true, "missing_bank_rejected": true, "mixed_gaps": true,
        "full_state_equal": true, "manifest_sha256": digest(&manifest_bytes),
        "packet_sha256": new.sha256, "packet_bytes": 1199367,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
