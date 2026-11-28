//! Verify the original screen-effect readers against altered ROM backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    SCREEN_EFFECT_MOTION_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const STATE: u32 = 0x0201_8000;
const CHILD: u32 = 0x0201_9000;
const MESSAGE: u32 = 0x0201_6000;
const ACTOR: u32 = 0x0201_a000;
const RETURN: u32 = 0x0812_3db0;
const PROOF_SHA: &str = "f1e6119778cfbf88ded38d68ce88434a2fe1f791fad0b9e4761031473712eff9";

#[derive(Deserialize)]
struct Counts {
    first_sprite_rows: u32,
    first_motion_rows: u32,
    second_physical_sprite_rows: u32,
    second_motion_rows: u32,
    offset_samples: u32,
    timed_rows: u32,
    scale_samples: u32,
    scanlines: u32,
    background_modes: u32,
    pending_release_alias: u32,
}

#[derive(Deserialize)]
struct Proof {
    counts: Counts,
    reader_literals: u32,
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], offset: u32) -> u16 {
    let index = offset as usize;
    u16::from_le_bytes(rom[index..index + 2].try_into().unwrap())
}

fn signed(value: u16) -> u32 {
    i32::from(value as i16) as u32
}

fn check_motion(machine: &mut Machine, at: u32, start: u16, end: u16, duration: u16) {
    let delta = end.wrapping_sub(start) as i16;
    assert_eq!(machine.bus.read16(at), start);
    assert_eq!(machine.bus.read16(at + 2), duration);
    assert_eq!(
        machine.bus.read16(at + 6),
        if delta < 0 { 0xffff } else { 1 }
    );
    assert_eq!(machine.bus.read16(at + 10), delta.unsigned_abs());
}

struct SpriteBank {
    rows: u32,
    motion_rows: u32,
    selectors: [u32; 3],
    kinds: Option<u32>,
    motion: u32,
    select: (u32, u32),
    initialize_motion: (u32, u32),
}

fn original_sprite_schedules(machine: &mut Machine, rom: &[u8], counts: &Counts) {
    for index in 0..counts.first_sprite_rows {
        initialize(machine, 0x0812_2c46);
        machine.cpu.regs[7] = STATE;
        let tick = half(rom, 0x3ac4dc + 2 * index);
        machine.bus.write8(STATE + 8, index as u8);
        machine.bus.write16(STATE + 2, tick);
        execute_until(machine, 0x0812_2bc0, 64);
        assert_eq!(
            (machine.cpu.regs[0], machine.cpu.regs[2]),
            (u32::from(tick), index)
        );
    }
    initialize(machine, 0x0812_2c46);
    machine.cpu.regs[7] = STATE;
    machine
        .bus
        .write8(STATE + 8, counts.first_sprite_rows as u8);
    execute_until(machine, 0x0812_2c78, 32);

    for bank in [
        SpriteBank {
            rows: counts.first_sprite_rows,
            motion_rows: counts.first_motion_rows,
            selectors: [0x3ac51e, 0x3ac534, 0x3ac54a],
            kinds: Some(0x3ac508),
            motion: 0x3ac560,
            select: (0x0812_287e, 0x0812_28b2),
            initialize_motion: (0x0812_28ca, 0x0812_2900),
        },
        SpriteBank {
            rows: counts.second_physical_sprite_rows,
            motion_rows: counts.second_motion_rows,
            selectors: [0x3ac580, 0x3ac5c0, 0x3ac600],
            kinds: None,
            motion: 0x3ac640,
            select: (0x0812_2986, 0x0812_29b4),
            initialize_motion: (0x0812_29de, 0x0812_2a14),
        },
    ] {
        let mut seen = BTreeSet::new();
        for index in 0..bank.rows {
            let [motion, position_x, position_y] =
                bank.selectors.map(|base| rom[(base + index) as usize]);
            let x = 60 + u16::from(position_x);
            let y = 40 + u16::from(position_y);
            seen.insert(motion);

            machine.bus.write16(MESSAGE + 0x14, 60);
            machine.bus.write16(MESSAGE + 0x16, 40);
            machine.bus.write16(MESSAGE + 0x18, index as u16);
            initialize(machine, bank.select.0);
            machine.cpu.regs[5] = MESSAGE;
            machine.cpu.regs[7] = STATE;
            execute_until(machine, bank.select.1, 64);
            assert_eq!(machine.bus.read8(STATE + 0xc), index as u8);
            assert_eq!(machine.bus.read8(STATE + 0xd), motion);
            assert_eq!(
                (machine.cpu.regs[4], machine.cpu.regs[5]),
                (signed(x), signed(y))
            );
            if let Some(base) = bank.kinds {
                assert_eq!(machine.cpu.regs[1], u32::from(rom[(base + index) as usize]));
            }

            // The motion block uses the original constructor's selected coordinates.
            machine.cpu.regs[15] = bank.initialize_motion.0;
            execute_until(machine, bank.initialize_motion.1, 512);
            let source = bank.motion + 8 * u32::from(motion);
            let duration = u16::from(rom[source as usize]);
            check_motion(
                machine,
                STATE + 0x10,
                x,
                x.wrapping_add(half(rom, source + 2)),
                duration,
            );
            check_motion(
                machine,
                STATE + 0x1c,
                y,
                y.wrapping_add(half(rom, source + 4)),
                duration,
            );
        }
        assert_eq!(seen.len() as u32, bank.motion_rows);
    }
}

fn original_offset_and_timed_children(machine: &mut Machine, rom: &[u8], counts: &Counts) {
    initialize(machine, 0x0812_31b8);
    execute_until(machine, 0x0812_31d0, 64);
    let first = signed(half(rom, 0x3ac680));
    let stack = machine.cpu.regs[13];
    assert_eq!(machine.bus.read32(stack), first);
    assert_eq!(machine.bus.read32(stack + 4), first);
    for index in 0..counts.offset_samples {
        initialize(machine, 0x0812_3208);
        machine.cpu.regs[5] = STATE;
        machine.bus.write16(STATE + 2, index as u16);
        execute_until(machine, 0x0812_3232, 64);
        let value = signed(half(rom, 0x3ac680 + 2 * index));
        assert_eq!(machine.bus.read32(stack), value);
        assert_eq!(machine.bus.read32(stack + 4), value);
    }

    for index in 0..counts.timed_rows {
        let source = 0x3ac6a8 + 20 * index;
        machine.bus.write16(STATE + 2, half(rom, source));
        machine.bus.write8(STATE + 8, index as u8);
        machine.bus.write32(STATE + 0xc, 0);
        initialize(machine, 0x0812_3380);
        machine.cpu.regs[5] = STATE;
        execute_until(machine, 0x0812_33c4, 64);
        let message = machine.cpu.regs[2];
        assert_eq!(machine.bus.read32(message + 4), BASE + source + 4);
        assert_eq!(machine.bus.read32(message + 8), STATE + 0x1c);
        assert_eq!(
            machine.bus.read16(message + 0x14),
            u16::from(rom[source as usize + 2])
        );

        machine.bus.write16(message, 0);
        machine.bus.write16(CHILD, 0);
        initialize(machine, 0x0812_2d9c);
        machine.cpu.regs[1] = message;
        machine.cpu.regs[2] = CHILD;
        execute_until(machine, 0x0812_2e28, 512);
        let duration = half(rom, source + 16) - 1;
        for axis in 0..3 {
            let from = source + 4 + 4 * axis;
            check_motion(
                machine,
                CHILD + 0xc + 12 * axis,
                half(rom, from),
                half(rom, from + 2),
                duration,
            );
        }
    }

    // An outstanding release leaves the reader on the first blend halfword.
    machine.bus.write8(STATE + 8, counts.timed_rows as u8);
    machine.bus.write16(STATE + 2, 120);
    initialize(machine, 0x0812_3380);
    machine.cpu.regs[5] = STATE;
    execute_until(machine, 0x0812_33d6, 64);
    assert_eq!(
        machine.cpu.regs[1],
        u32::from(half(rom, counts.pending_release_alias))
    );
}

fn original_scale_and_scanlines(machine: &mut Machine, rom: &[u8], counts: &Counts) {
    machine.bus.write32(STATE + 0x1c, ACTOR);
    for index in 0..counts.scale_samples {
        initialize(machine, 0x0812_37c4);
        machine.cpu.regs[5] = STATE;
        machine.bus.write16(STATE + 2, index as u16);
        execute_until(machine, 0x0812_37e6, 128);
        let sample = half(rom, 0x3ac7b8 + 2 * index);
        assert_eq!(machine.bus.read16(ACTOR + 0x32), sample);
        assert_eq!(machine.bus.read16(ACTOR + 0x34), sample);
        assert_eq!(machine.bus.read16(ACTOR + 0x36), 0);
        assert_eq!(machine.bus.read16(STATE + 2), index as u16 + 1);
    }

    let start = STATE + 0x18;
    let end = start + counts.scanlines * 2;
    for direction in [0, 1] {
        machine.bus.write8(STATE + 9, direction);
        machine.bus.write16(STATE + 0xc, 0);
        for at in (start - 2..=end).step_by(2) {
            machine.bus.write16(at, 0x5a5a);
        }
        initialize(machine, 0x0812_3946);
        machine.cpu.regs[5] = STATE;
        execute_until(machine, 0x0812_39dc, 20_000);
        for line in 0..counts.scanlines {
            let value = u16::from(rom[(0x3ac860 + line) as usize]);
            let expected = if direction == 0 {
                0u16.wrapping_sub(value)
            } else {
                240 + value
            };
            assert_eq!(machine.bus.read16(start + 2 * line), expected);
        }
        assert_eq!(machine.bus.read16(start - 2), 0x5a5a);
        assert_eq!(machine.bus.read16(end), 0x5a5a);
    }
}

fn original_background_modes(machine: &mut Machine, rom: &[u8], counts: &Counts) {
    for mode in 0..counts.background_modes {
        let x = half(rom, 0x3ac900 + 4 * mode);
        let y = half(rom, 0x3ac902 + 4 * mode);
        let end_x = half(rom, 0x3ac914 + 4 * mode);
        let end_y = half(rom, 0x3ac916 + 4 * mode);
        let duration = 260;
        let width = u16::from(rom[(0x3ac92d + mode) as usize]);
        let delay = half(rom, 0x3ac938 + 2 * mode);

        machine.bus.write8(STATE + 8, mode as u8);
        initialize(machine, 0x0812_3b64);
        machine.cpu.regs[7] = STATE;
        execute_until(machine, 0x0812_3b84, 64);
        let stack = machine.cpu.regs[13];
        assert_eq!(
            machine.cpu.regs[1],
            u32::from(rom[(0x3ac928 + mode) as usize])
        );
        assert_eq!(machine.cpu.regs[3], signed(0u16.wrapping_sub(x)));
        assert_eq!(machine.bus.read32(stack), signed(0u16.wrapping_sub(y)));

        machine.cpu.regs[15] = 0x0812_3ba4;
        execute_until(machine, 0x0812_3c0a, 512);
        check_motion(machine, STATE + 0xc, x, end_x, duration);
        check_motion(machine, STATE + 0x18, y, end_y, duration);
        assert_eq!(machine.bus.read16(stack + 0x22), width);
        assert_eq!(machine.bus.read32(stack + 0x14), signed(delay));

        let tick = duration - u16::from(rom[(0x3ac932 + mode) as usize]);
        machine.bus.write16(STATE + 2, tick - 1);
        initialize(machine, 0x0812_3d6e);
        machine.cpu.regs[7] = STATE;
        execute_until(machine, RETURN, 64);
        assert_eq!(machine.bus.read16(STATE + 2), tick);
        assert_eq!(machine.bus.read16(stack + 0x3e), width);
        assert_eq!(machine.bus.read32(stack + 0x30), signed(delay));
    }
}

#[test]
#[ignore = "Exact EUR ROM, E647 source proof and e375/e376 datasets"]
fn european_screen_effect_motion_feeds_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_SCREEN_EFFECT_MOTION_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.reader_literals, 24);

    let mut previous = read_dataset(&input("FFTA_SCREEN_EFFECT_MOTION_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_SCREEN_EFFECT_MOTION_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (375, 374)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (376, 375)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .screen_effect_motion_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 376;
    previous.manifest.decoder_revision = 375;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == SCREEN_EFFECT_MOTION_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.screen_effect_motion_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 1089);
    packed[source_bytes - 1] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let counts = &proof.counts;
    let altered = || {
        proof
            .new_ranges
            .iter()
            .map(|span| (span.offset, span.length))
            .chain(std::iter::once((counts.pending_release_alias, 2)))
    };
    for (offset, length) in altered() {
        flip(&mut managed, offset, length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.screen_effect_motion_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        original_sprite_schedules(machine, &rom, counts);
        original_offset_and_timed_children(machine, &rom, counts);
        original_scale_and_scanlines(machine, &rom, counts);
        original_background_modes(machine, &rom, counts);
    }
    let original_reads = owner.screen_effect_motion_bus_bytes().unwrap() - before;
    assert!(original_reads > 0);
    for (offset, length) in altered() {
        flip(&mut managed, offset, length);
    }
    same_state(&raw, &managed, RETURN);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "first_sprite_rows_and_ticks": counts.first_sprite_rows,
        "second_physical_sprite_rows": counts.second_physical_sprite_rows,
        "sprite_motion_rows": counts.first_motion_rows + counts.second_motion_rows,
        "offset_samples": counts.offset_samples,
        "timed_motion_rows": counts.timed_rows,
        "scale_samples": counts.scale_samples,
        "scanline_cases": 2 * counts.scanlines,
        "background_modes": counts.background_modes,
        "pending_release_alias": true,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_SCREEN_EFFECT_MOTION_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
