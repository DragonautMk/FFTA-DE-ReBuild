//! Exercise the selected EUR UI text readers with altered ROM copies.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    LOCALIZED_INLINE_UI_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{text_short::Text, SourceRange},
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = BASE + 0x300;
const DESTINATION: u32 = 0x0201_0000;
const EXPANDER: u32 = 0x0801_9d98;
const GLYPH_ENTRY: u32 = 0x0801_aae6;
const GLYPH_STOP: u32 = 0x0801_ab26;
const PROOF_SHA: &str = "15ada2da70db3b98df0b1b9843af2a9aad5cbf2244a8aef18fde68c8bc8e780b";

#[derive(Deserialize)]
struct Reference {
    pc: u32,
    target: u32,
    register: usize,
}

#[derive(Deserialize)]
struct Table {
    offset: u32,
    strings: [(u32, u32); 5],
}

#[derive(Deserialize)]
struct Selector {
    entry: u32,
    stop: u32,
    table: u32,
    output: usize,
    language_register: Option<usize>,
}

#[derive(Deserialize)]
struct Proof {
    references: Vec<Reference>,
    literal_words: usize,
    language: u32,
    tables: Vec<Table>,
    shared: [(u32, u32); 6],
    glyphs: (u32, u32),
    selectors: Vec<Selector>,
    fallback_entry: u32,
    fallback_stop: u32,
    fallback_source: u32,
    new_ranges: Vec<SourceRange>,
}

fn original_selections(machine: &mut Machine, proof: &Proof) {
    for reference in &proof.references {
        initialize(machine, BASE + reference.pc);
        execute_until(machine, BASE + reference.pc + 2, 2);
        assert_eq!(
            machine.cpu.regs[reference.register],
            BASE + reference.target
        );
    }

    for reader in &proof.selectors {
        let table = proof
            .tables
            .iter()
            .find(|table| table.offset == reader.table)
            .unwrap();
        for (language, &(start, _)) in table.strings.iter().enumerate() {
            initialize(machine, BASE + reader.entry);
            machine.bus.write8(proof.language, language as u8);
            if let Some(register) = reader.language_register {
                machine.cpu.regs[register] = proof.language;
            }
            execute_until(machine, BASE + reader.stop, 32);
            assert_eq!(
                machine.cpu.regs[reader.output],
                BASE + start,
                "language {language} at {:06x}",
                reader.entry
            );
        }
    }

    let guarded = proof
        .selectors
        .iter()
        .find(|reader| reader.entry == 0x05a782)
        .unwrap();
    let table = proof
        .tables
        .iter()
        .find(|table| table.offset == guarded.table)
        .unwrap();
    for (language, &(start, _)) in table.strings.iter().enumerate() {
        initialize(machine, BASE + proof.fallback_entry);
        machine.bus.write8(proof.language, language as u8);
        machine.cpu.regs[5] = proof.language;
        let (stop, selected) = if language == 3 {
            (proof.fallback_stop, proof.fallback_source)
        } else {
            (guarded.stop, start)
        };
        execute_until(machine, BASE + stop, 32);
        assert_eq!(machine.cpu.regs[2], BASE + selected, "language {language}");
    }
}

fn original_expansion(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let strings = proof
        .tables
        .iter()
        .flat_map(|table| &table.strings)
        .chain(&proof.shared);
    for &(start, end) in strings {
        let source = &rom[start as usize..end as usize];
        let (text, consumed) = Text::decode(source, start).unwrap();
        assert_eq!(consumed, source.len());
        assert_eq!(text.source_bytes().unwrap(), source);
        let expanded = text.expanded_bytes().unwrap();
        let end = DESTINATION + expanded.len() as u32;
        for at in DESTINATION - 1..=end {
            machine.bus.write8(at, 0x5a);
        }

        initialize(machine, EXPANDER);
        machine.cpu.regs[0] = DESTINATION;
        machine.cpu.regs[1] = BASE + start;
        execute_until(machine, RETURN, 4096);
        assert_eq!(machine.cpu.regs[0], expanded.len() as u32 - 1);
        for (index, byte) in expanded.into_iter().enumerate() {
            assert_eq!(machine.bus.read8(DESTINATION + index as u32), byte);
        }
        assert_eq!(machine.bus.read8(DESTINATION - 1), 0x5a);
        assert_eq!(machine.bus.read8(end), 0x5a);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E649 source proof and e376/e377 datasets"]
fn european_localized_inline_ui_uses_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_LOCALIZED_INLINE_UI_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.references.len(), 29);
    assert_eq!(proof.literal_words, 27);
    assert_eq!(proof.tables.len(), 7);
    assert_eq!(proof.selectors.len(), 8);
    assert_eq!(proof.glyphs.1 - proof.glyphs.0, 12);

    let mut previous = read_dataset(&input("FFTA_LOCALIZED_INLINE_UI_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_LOCALIZED_INLINE_UI_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (376, 375)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (377, 376)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .localized_inline_ui_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 377;
    previous.manifest.decoder_revision = 376;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == LOCALIZED_INLINE_UI_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.localized_inline_ui_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 886);
    packed[source_bytes - 1] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.localized_inline_ui_bus_bytes().unwrap();
    let levels: Vec<_> = (0..=16u32).chain([u32::MAX]).collect();
    for machine in [&mut raw, &mut managed] {
        original_selections(machine, &proof);
        original_expansion(machine, &rom, &proof);
        for &level in &levels {
            let offset = (proof.glyphs.0 + 2 * (level / 3).min(5)) as usize;
            initialize(machine, GLYPH_ENTRY);
            machine.cpu.regs[4] = level;
            let stack = machine.cpu.regs[13];
            machine.bus.write32(stack, 0x5a5a_5a5a);
            execute_until(machine, GLYPH_STOP, 32);
            assert_eq!(machine.bus.read8(stack), rom[offset]);
            assert_eq!(machine.bus.read8(stack + 1), rom[offset + 1]);
            assert_eq!(machine.bus.read8(stack + 2), 0);
            assert_eq!(machine.bus.read8(stack + 3), 0x5a);
        }
    }
    let original_reads = owner.localized_inline_ui_bus_bytes().unwrap() - before;
    assert!(original_reads > 0);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, GLYPH_STOP);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "literal_readers": proof.references.len(),
        "language_reads": proof.selectors.len() * 5,
        "guarded_language_cases": 5,
        "expanded_strings": proof.tables.len() * 5 + proof.shared.len(),
        "level_glyph_inputs": levels.len(),
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_LOCALIZED_INLINE_UI_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
