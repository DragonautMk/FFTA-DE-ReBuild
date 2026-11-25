//! Verify the complete E514 CPUSet source range through the original callers.
use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Extraction,
    Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{ffta_rom::digest, Bus, Machine, StepEvent};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const CPU_SET: u32 = 0x0814_5230;
const CONTEXT: u32 = 0x0300_09e0;
const DESTINATIONS: [u32; 3] = [0x0600_6500, 0x0600_6a00, 0x0600_6e00];

#[derive(Deserialize)]
struct Plan {
    copy_calls: [CopyCall; 3],
    spans: [SourceWitness; 6],
    complete_range: [u32; 2],
}

#[derive(Deserialize)]
struct CopyCall {
    site: u32,
    load: u32,
    source: u32,
    halfwords: u32,
}

#[derive(Deserialize)]
struct SourceWitness {
    offset: u32,
    length: u32,
    role: String,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_original(machine: &mut Machine, calls: &[CopyCall; 3]) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.write32(CONTEXT, DESTINATIONS[0]);
    machine.bus.write32(CONTEXT + 0xb0, DESTINATIONS[1]);
    machine.bus.write32(CONTEXT + 4, DESTINATIONS[2]);
    machine.cpu.regs[7] = CONTEXT;
    machine.cpu.regs[13] = 0x0300_7c00;
    let mut observed = 0;
    // The first two copies run consecutively. The third uses a different
    // live context field and enters at its own original literal load.
    for (entry, last, argument) in [(0, 1, CONTEXT + 0x17), (2, 2, CONTEXT + 4 + 0x19)] {
        machine.cpu.regs[15] = ROM_BASE + calls[entry].load;
        machine.cpu.regs[1] = argument;
        machine.cpu.cpsr = 0xbf;
        machine.cpu.branch = None;
        let end = ROM_BASE + calls[last].site + 4;
        for _ in 0..64 {
            let pc = machine.cpu.regs[15];
            if pc == end {
                break;
            }
            let args = [
                machine.cpu.regs[0],
                machine.cpu.regs[1],
                machine.cpu.regs[2],
            ];
            let link = machine.cpu.regs[14];
            if let StepEvent::Instr(instr) = machine.step() {
                if matches!(instr.op, armv4t::Op::Swi { imm: 0x0b }) {
                    let call = &calls[observed];
                    assert!(instr.thumb);
                    assert_eq!(pc, CPU_SET);
                    assert_eq!(link, ROM_BASE + call.site + 5);
                    assert_eq!(
                        args,
                        [
                            ROM_BASE + call.source,
                            DESTINATIONS[observed],
                            call.halfwords
                        ]
                    );
                    observed += 1;
                }
            }
        }
        assert_eq!(machine.cpu.regs[15], end);
        assert_eq!(observed, last + 1);
    }
}

#[test]
#[ignore = "exact EUR ROM, E514 source plan and E513 predecessor manifest"]
fn european_halfword_preloads_publish_and_execute_original_copies() {
    let work = input("FFTA_HALFWORD_PRELOAD_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_HALFWORD_PRELOAD_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_HALFWORD_PRELOAD_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E513 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (318, 317)
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
        .join(&prior.source_rom_sha256)
        .join("v1/e319-d318");
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
    let manifest: Manifest = parse(&manifest_bytes, "E515 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (319, 318)
    );
    let index = manifest
        .resources
        .iter()
        .position(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let old = &prior.resources[index];
    let new = &manifest.resources[index];
    let mut without_preloads = manifest.clone();
    without_preloads.resources[index] = old.clone();
    let mut previous_layout = without_preloads.clone();
    previous_layout.extractor_revision = prior.extractor_revision;
    previous_layout.decoder_revision = prior.decoder_revision;
    assert_eq!(previous_layout, prior);
    assert_eq!(
        (&new.id, &new.path, &new.format, &new.dependencies),
        (&old.id, &old.path, &old.format, &old.dependencies)
    );
    assert!(old.sources.iter().all(|span| new.sources.contains(span)));
    let added: Vec<_> = new
        .sources
        .iter()
        .filter(|span| !old.sources.contains(span))
        .collect();
    assert_eq!(added.len(), plan.spans.len());
    for (source, witness) in added.iter().zip(&plan.spans) {
        assert_eq!(
            (source.offset, source.length, &source.role, &source.sha256),
            (
                witness.offset,
                witness.length,
                &witness.role,
                &witness.sha256
            )
        );
        assert!(prior.resources.iter().flat_map(|r| &r.sources).all(|span| {
            span.offset + span.length <= source.offset
                || source.offset + source.length <= span.offset
        }));
    }
    let [start, end] = plan.complete_range;
    let mut cursor = start;
    for span in new
        .sources
        .iter()
        .filter(|span| (start..end).contains(&span.offset))
    {
        assert_eq!(span.offset, cursor);
        cursor += span.length;
    }
    assert_eq!(cursor, end);
    assert_eq!(end - start, 3164);

    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    let preloads = packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .preloads
        .take()
        .unwrap();
    assert_eq!(
        preloads.banks.each_ref().map(|b| b.halfwords.len()),
        [126, 126, 288]
    );
    let prior_packet = Extraction::from_rom_for_manifest(&rom, &prior)
        .unwrap()
        .ui_raw_transfers
        .unwrap()
        .decoded;
    assert_eq!(packet, prior_packet);
    let prior_packet_bytes = encode_ui_raw_transfers(&prior_packet).unwrap();
    assert_eq!(digest(&prior_packet_bytes), old.sha256);

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    fs::write(&packet_path, &prior_packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let first_payload = plan.copy_calls[0].source as usize;
    managed.bus.rom[first_payload] ^= 0xff;
    assert_eq!(
        managed.bus.read8(ROM_BASE + first_payload as u32),
        !rom[first_payload]
    );
    managed.bus.rom[first_payload] ^= 0xff;
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(
        &manifest_path,
        toml::to_string_pretty(&without_preloads).unwrap(),
    )
    .unwrap();
    fs::write(&packet_path, prior_packet_bytes).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();

    let poison = |machine: &mut Machine| {
        for span in &added {
            let at = span.offset as usize;
            for byte in &mut machine.bus.rom[at..at + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    let mut raw = Machine::new(rom);
    let initial = raw.save_state().unwrap();
    managed.load_state(&initial).unwrap();
    poison(&mut managed);
    run_original(&mut raw, &plan.copy_calls);
    run_original(&mut managed, &plan.copy_calls);
    for (call, address) in plan.copy_calls.iter().zip(DESTINATIONS) {
        let at = call.source as usize;
        let length = (2 * call.halfwords) as usize;
        let destination = (address - VRAM_BASE) as usize;
        let expected = &raw.bus.rom[at..at + length];
        assert_eq!(&raw.bus.vram[destination..destination + length], expected);
        assert_eq!(
            &managed.bus.vram[destination..destination + length],
            expected
        );
    }
    let stats = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.payload_bytes,
            stats.selector_bytes
        ),
        (1092, 1080, 12)
    );
    assert_eq!(stats.banks_used, 3);
    poison(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    // The halfwords touching both ends of the new payload still come from
    // the E513 banks. The first byte past the new literals is still raw.
    for at in [first_payload - 2, 0xb4d0f8] {
        managed.bus.rom[at..at + 2]
            .iter_mut()
            .for_each(|byte| *byte ^= 0xff);
        assert_eq!(
            managed.bus.read16(ROM_BASE + at as u32).to_le_bytes(),
            raw.bus.rom[at..at + 2]
        );
    }
    let gap = 0x075c3c;
    managed.bus.rom[gap] ^= 0xff;
    assert_eq!(managed.bus.read8(ROM_BASE + gap as u32), !raw.bus.rom[gap]);

    let result = serde_json::json!({
        "status": "PASS", "revision": [319, 318], "spans": added.len(), "source_bytes": stats.source_bytes,
        "copied_bytes": stats.payload_bytes, "source_literal_bytes": stats.selector_bytes,
        "original_calls": plan.copy_calls.len(), "complete_payload_range": [start, end],
        "complete_payload_bytes": end - start, "predecessor_attach": true, "missing_preloads_rejected": true,
        "adjacent_owners_preserved": true, "full_state_equal": true, "manifest_sha256": digest(&manifest_bytes),
        "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
