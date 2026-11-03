//! One combined Wine source oracle for the new localized text sources.
use super::*;
use gba_core::{
    rom_data::{script_text::*, text_catalogs::ProgramUnit},
    Bus, Machine,
};
use serde_json::Value;
use std::collections::BTreeSet;

const RETURN: u32 = 0x08000300;
fn reset(m: &mut Machine, pc: u32) {
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
    m.bus.write32(0x03007c00, 0);
    m.bus.write32(0x03007c04, 0);
}
fn until(m: &mut Machine, end: u32, limit: usize, pcs: &mut BTreeSet<u32>) -> usize {
    for n in 0..limit {
        if m.cpu.regs[15] == end {
            return n;
        }
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
    }
    panic!(
        "original text pc{:08x} end{end:08x} regs{:08x?}",
        m.cpu.regs[15], m.cpu.regs
    );
}
fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn raw_target(m: &mut Machine, rom: &[u8], language: u8, key: u16, message: u16, live: u8) -> u32 {
    let root = ROOTS[if language < 5 {
        usize::from(language)
    } else {
        0
    }];
    let key = if message > 239 { 0xaa } else { key };
    let bank = rom[0xd68c2c + usize::from(key) * 4 + 2];
    let mut target = 0x08000000u32
        .wrapping_add(root)
        .wrapping_add(word(rom, root + u32::from(bank) * 4));
    if message <= 239 && bank == 61 {
        let at = target.wrapping_add(u32::from(live.saturating_sub(1) / 10) * 4);
        target = (0x08000000 + root).wrapping_add(m.bus.read32(at));
    }
    target
}
fn remap(rom: &[u8], key: u16, message: u16, live: u8) -> u16 {
    if message as u8 > 239 {
        return u16::from((message as u8).wrapping_sub(240));
    }
    if rom[0xd68c2c + usize::from(key) * 4 + 2] == 61 {
        message.wrapping_add(u16::from(live.saturating_sub(1) % 10) * 24)
    } else {
        message
    }
}

#[test]
#[ignore = "private EUR localized text model and one combined original reader oracle; Wine only, Linux/USA pending"]
fn european_script_text_matches_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_PRIOR_DATASET").unwrap());
    let keys_bytes = fs::read(prior.join("script-archive.json")).unwrap();
    let font_bytes = fs::read(prior.join("font-bank.json")).unwrap();
    for (id, bytes) in [
        ("engine.script-archive", &keys_bytes),
        ("font.complete-bank", &font_bytes),
    ] {
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            evidence["dependency_file_sha256"][id].as_str().unwrap()
        );
    }
    let keys: gba_core::rom_data::script_archive::ScriptArchive =
        serde_json::from_slice(&keys_bytes).unwrap();
    let font: gba_core::rom_data::fonts::FontBank = serde_json::from_slice(&font_bytes).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_dependencies(&keys, &font).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 5);
    for ((at, bytes, role), expected) in spans
        .iter()
        .zip(evidence["source_ranges"].as_array().unwrap())
    {
        assert_eq!(u64::from(*at), expected["offset"].as_u64().unwrap());
        assert_eq!(bytes.len() as u64, expected["length"].as_u64().unwrap());
        assert_eq!(role, expected["role"].as_str().unwrap());
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            expected["sha256"].as_str().unwrap()
        );
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }
    assert_eq!(spans.iter().map(|v| v.1.len()).sum::<usize>(), SOURCE_BYTES);
    let mut program_count = 0;
    let mut flags = [0; 4];
    let mut cases = BTreeSet::new();
    let mut forms = BTreeSet::new();
    let mut headers = BTreeSet::new();
    let mut glyphs = BTreeSet::new();
    let mut opcodes = BTreeSet::new();
    for (li, (language, expected)) in data
        .languages
        .iter()
        .zip(evidence["languages"].as_array().unwrap())
        .enumerate()
    {
        assert_eq!(
            u64::from(language.id),
            expected["language"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(language.source_offset),
            expected["source_offset"].as_u64().unwrap()
        );
        for (target, offset) in language
            .top
            .iter()
            .zip(expected["top_offsets"].as_array().unwrap())
        {
            assert_eq!(
                u64::from(language.target_offset(target).unwrap() - language.source_offset),
                offset.as_u64().unwrap()
            );
        }
        for (&index, offset) in language
            .variants
            .bank_indices
            .iter()
            .zip(expected["variant_offsets"].as_array().unwrap())
        {
            assert_eq!(
                u64::from(
                    language.banks[usize::from(index)].source_offset - language.source_offset
                ),
                offset.as_u64().unwrap()
            );
        }
        for (bi, (bank, expected)) in language
            .banks
            .iter()
            .zip(expected["banks"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(
                u64::from(bank.source_offset),
                expected["source_offset"].as_u64().unwrap()
            );
            let expected_indices: Vec<_> = expected["offsets"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|x| (x.as_u64().unwrap() as u16).to_le_bytes())
                .collect();
            assert_eq!(bank.index_bytes().unwrap(), expected_indices);
            assert_eq!(
                bank.programs.len(),
                expected["programs"].as_array().unwrap().len()
            );
            cases.insert((li, bi, 0));
            cases.insert((li, bi, bank.programs.len() - 1));
            for (pi, (program, expected)) in bank
                .programs
                .iter()
                .zip(expected["programs"].as_array().unwrap())
                .enumerate()
            {
                program_count += 1;
                flags[(program.header & 3) as usize] += 1;
                assert_eq!(
                    u64::from(program.source_offset),
                    expected["source_offset"].as_u64().unwrap()
                );
                assert_eq!(
                    u64::from(program.header),
                    expected["header"].as_u64().unwrap()
                );
                let source = program.source_bytes().unwrap();
                assert_eq!(source.len() as u64, expected["length"].as_u64().unwrap());
                assert_eq!(
                    gba_core::ffta_rom::digest(&source),
                    expected["sha256"].as_str().unwrap()
                );
                assert_eq!(
                    source,
                    rom[program.source_offset as usize
                        ..program.source_offset as usize + source.len()]
                );
                assert_eq!(
                    serde_json::to_value(&program.decoded_padding).unwrap(),
                    expected["decoded_padding"]
                );
                assert_eq!(
                    serde_json::to_value(&program.encoded).unwrap(),
                    expected["encoded"]
                );
                assert_eq!(
                    program.units.len(),
                    expected["units"].as_array().unwrap().len()
                );
                if forms.insert(program.header & 3) || headers.insert(program.header) {
                    cases.insert((li, bi, pi));
                }
                for (unit, expected) in program
                    .units
                    .iter()
                    .zip(expected["units"].as_array().unwrap())
                {
                    match unit {
                        ProgramUnit::Glyph { index } => {
                            assert_eq!(u64::from(*index), expected["glyph"].as_u64().unwrap());
                            if glyphs.insert(*index) {
                                cases.insert((li, bi, pi));
                            }
                        }
                        ProgramUnit::Command { opcode, operands } => {
                            assert_eq!(u64::from(*opcode), expected["command"].as_u64().unwrap());
                            assert_eq!(
                                serde_json::to_value(operands).unwrap(),
                                expected["operands"]
                            );
                            if opcodes.insert((program.header & 1, *opcode)) {
                                cases.insert((li, bi, pi));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        (program_count, flags, glyphs.len(), opcodes.len()),
        (41010, [5709, 25488, 9813, 0], 147, 11)
    );
    for (gap, expected) in data
        .alignment
        .iter()
        .zip(evidence["alignment"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(gap.source_offset),
            expected["source_offset"].as_u64().unwrap()
        );
        assert_eq!(serde_json::to_value(&gap.bytes).unwrap(), expected["bytes"]);
    }
    assert_eq!(
        data.alignment.iter().map(|p| p.bytes.len()).sum::<usize>(),
        34767
    );
    let json = serde_json::to_vec_pretty(data).unwrap();
    let decoded: ScriptText = serde_json::from_slice(&json).unwrap();
    assert_eq!(&decoded, data);
    assert_eq!(serde_json::to_vec_pretty(&decoded).unwrap(), json);
    fs::write(root.join("script-text.json"), &json).unwrap();
    for n in 0..22 {
        let mut bad = data.clone();
        match n {
            0 => {
                bad.languages.pop();
            }
            1 => bad.languages[0].id = 1,
            2 => bad.languages[0].source_offset += 2,
            3 => bad.languages[0].end += 2,
            4 => {
                bad.languages[0].top.pop();
            }
            5 => bad.languages[0].top[61] = TopTarget::Bank { index: 0 },
            6 => bad.languages[0].top[0] = TopTarget::Variants {},
            7 => bad.languages[0].top[0] = TopTarget::Bank { index: 255 },
            8 => bad.languages[0].variants.bank_indices[0] = 255,
            9 => {
                bad.languages[0].variants.bank_indices.pop();
            }
            10 => bad.languages[0].banks.swap(0, 1),
            11 => {
                bad.languages[0].banks[0].programs.pop();
            }
            12 => bad.languages[0].banks[0].sentinel = 0,
            13 => bad.languages[0].banks[0].programs[0].source_offset += 1,
            14 => {
                bad.alignment.pop();
            }
            15 => bad.alignment[0].bytes.clear(),
            16 => bad.alignment[0].bytes = vec![0; 4],
            17 => bad.alignment.swap(0, 1),
            18 => bad.alignment[0].source_offset = u32::MAX,
            19 => bad.languages[0].banks[0].programs[0].header ^= 1,
            20 => bad.languages[0].banks[0].programs[0]
                .encoded
                .as_mut()
                .unwrap()
                .clear(),
            _ => {
                let unit = bad
                    .languages
                    .iter_mut()
                    .flat_map(|l| &mut l.banks)
                    .flat_map(|b| &mut b.programs)
                    .flat_map(|p| &mut p.units)
                    .find(|u| matches!(u, ProgramUnit::Glyph { .. }))
                    .unwrap();
                if let ProgramUnit::Glyph { index } = unit {
                    *index = u16::MAX;
                }
            }
        }
        assert!(bad.validate().is_err(), "shape guard{n}");
    }
    for n in 0..6 {
        let mut bad = data.clone();
        match n {
            0 => bad.languages[0].top[0] = bad.languages[0].top[1].clone(),
            1 => bad.languages[0].variants.bank_indices.swap(0, 1),
            2 => bad.languages[0].banks[0].programs[0].header ^= 0x10,
            3 => {
                let program = bad
                    .languages
                    .iter_mut()
                    .flat_map(|l| &mut l.banks)
                    .flat_map(|b| &mut b.programs)
                    .find(|p| {
                        p.header & 3 == 0
                            && p.units
                                .iter()
                                .any(|u| matches!(u, ProgramUnit::Glyph { .. }))
                    })
                    .unwrap();
                if let ProgramUnit::Glyph { index } = program
                    .units
                    .iter_mut()
                    .find(|u| matches!(u, ProgramUnit::Glyph { .. }))
                    .unwrap()
                {
                    *index ^= 1;
                }
            }
            4 => {
                let program = bad.languages.iter_mut().flat_map(|l| &mut l.banks).flat_map(|b| &mut b.programs).find(|p| p.header&2==0 && p.units.iter().any(|u| matches!(u, ProgramUnit::Command {operands,..} if !operands.is_empty()))).unwrap();
                if let ProgramUnit::Command { operands, .. } = program
                    .units
                    .iter_mut()
                    .find(
                        |u| matches!(u, ProgramUnit::Command {operands,..} if !operands.is_empty()),
                    )
                    .unwrap()
                {
                    operands[0] ^= 1;
                }
            }
            _ => bad.alignment[0].bytes[0] ^= 1,
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err(), "coherent source guard{n}");
    }
    let mut badkeys = keys.clone();
    badkeys.keys[0].text_bank = 255;
    assert!(data.validate_dependencies(&badkeys, &font).is_err());
    let mut badfont = font.clone();
    badfont.glyphs.pop();
    assert!(data.validate_dependencies(&keys, &badfont).is_err());
    let mut bad = data.clone();
    let p = bad
        .languages
        .iter_mut()
        .flat_map(|l| &mut l.banks)
        .flat_map(|b| &mut b.programs)
        .find(|p| {
            p.header & 3 == 0
                && p.units
                    .iter()
                    .any(|u| matches!(u, ProgramUnit::Glyph { .. }))
        })
        .unwrap();
    if let ProgramUnit::Glyph { index } = p
        .units
        .iter_mut()
        .find(|u| matches!(u, ProgramUnit::Glyph { .. }))
        .unwrap()
    {
        *index = 3175;
    }
    bad.validate().unwrap();
    assert!(bad.validate_dependencies(&keys, &font).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    for s in [
        r#"{"languages":[],"alignment":[],"extra":0}"#,
        r#"{"languages":[],"languages":[],"alignment":[]}"#,
        r#"{"languages":[],"alignment":[{"source_offset":0,"bytes":[],"extra":0}]}"#,
        r#"{"languages":[],"alignment":[{"source_offset":-1,"bytes":[]}]}"#,
        r#"{"languages":[],"alignment":[{"source_offset":0,"bytes":[256]}]}"#,
        r#"{"languages":null,"alignment":[]}"#,
        r#"{"languages":[],"alignment":[]}"#,
    ] {
        let parsed = serde_json::from_str::<ScriptText>(s);
        assert!(parsed.is_err() || parsed.unwrap().validate().is_err());
    }
    assert!(serde_json::from_slice::<ScriptText>(&json[..json.len() - 1]).is_err());
    for s in [
        r#"{"kind":"bank","index":256}"#,
        r#"{"kind":"bank","index":0,"extra":0}"#,
        r#"{"kind":"unknown"}"#,
        r#"{"kind":"variants","index":0}"#,
    ] {
        assert!(serde_json::from_str::<TopTarget>(s).is_err());
    }
    let mut m = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut m,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = m.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut steps = 0;
    let mut language_cases = 0;
    let mut bank_cases = 0;
    let mut live_cases = 0;
    let mut remap_cases = 0;
    let mut raw_cases = 0;
    let mut index_cases = 0;
    for language in [0u8, 1, 2, 3, 4, 5, 255] {
        reset(&mut m, 0x08009ab0);
        m.bus.write8(0x02001f6e, language);
        steps += until(&mut m, RETURN, 80, &mut pcs);
        assert_eq!(
            m.cpu.regs[0],
            0x08000000 + data.source_language(language).unwrap().source_offset
        );
        language_cases += 1;
    }
    for language in &data.languages {
        m.bus.write8(0x02001f6e, language.id);
        m.bus.write8(0x02003c2a, 0);
        for key in &keys.keys {
            reset(&mut m, 0x08009b08);
            m.bus.write16(0x02002192, key.id);
            m.cpu.regs[0] = 0;
            steps += until(&mut m, RETURN, 400, &mut pcs);
            assert_eq!(
                m.cpu.regs[0],
                0x08000000
                    + language
                        .source_bank(key.text_bank, 0)
                        .unwrap()
                        .source_offset
            );
            bank_cases += 1;
        }
        for live in 0u8..=255 {
            reset(&mut m, 0x08009b08);
            m.bus.write16(0x02002192, 115);
            m.bus.write8(0x02003c2a, live);
            m.cpu.regs[0] = 23;
            steps += until(&mut m, RETURN, 400, &mut pcs);
            assert_eq!(
                m.cpu.regs[0],
                0x08000000 + language.source_bank(61, live).unwrap().source_offset
            );
            live_cases += 1;
        }
        for bank in &language.banks {
            for index in [0, bank.programs.len() - 1, bank.programs.len()] {
                reset(&mut m, 0x08013f96);
                m.cpu.regs[1] = 0x08000000 + bank.source_offset;
                m.cpu.regs[3] = index as u32;
                steps += until(&mut m, 0x08013fa0, 8, &mut pcs);
                assert_eq!(
                    m.cpu.regs[2],
                    0x08000000
                        + bank.source_offset
                        + u32::from(half(&rom, bank.source_offset + index as u32 * 2))
                );
                index_cases += 1;
            }
        }
        for index in [65535u32, 65536, 0x1ffff] {
            let bank = &language.banks[0];
            reset(&mut m, 0x08013f96);
            m.cpu.regs[1] = 0x08000000 + bank.source_offset;
            m.cpu.regs[3] = index;
            steps += until(&mut m, 0x08013fa0, 8, &mut pcs);
            assert_eq!(
                m.cpu.regs[2],
                0x08000000
                    + bank.source_offset
                    + u32::from(half(&rom, bank.source_offset + (index as u16 as u32) * 2))
            );
            index_cases += 1;
        }
    }
    for live in 0u8..=255 {
        reset(&mut m, 0x08009b7c);
        m.bus.write16(0x02002192, 115);
        m.bus.write8(0x02003c2a, live);
        m.cpu.regs[0] = 23;
        steps += until(&mut m, RETURN, 400, &mut pcs);
        assert_eq!(m.cpu.regs[0], u32::from(remap(&rom, 115, 23, live)));
        remap_cases += 1;
    }
    for language in [0u8, 4, 255] {
        for key in [0u16, 115, 234, 239, 65535] {
            for message in [239u16, 240, 255, 256, 0x1f0, 0xffff] {
                m.bus.write8(0x02001f6e, language);
                m.bus.write16(0x02002192, key);
                m.bus.write8(0x02003c2a, 255);
                let target = raw_target(&mut m, &rom, language, key, message, 255);
                reset(&mut m, 0x08009b08);
                m.cpu.regs[0] = u32::from(message);
                steps += until(&mut m, RETURN, 400, &mut pcs);
                assert_eq!(m.cpu.regs[0], target, "raw root key{key} msg{message}");
                reset(&mut m, 0x08009b7c);
                m.cpu.regs[0] = u32::from(message);
                steps += until(&mut m, RETURN, 400, &mut pcs);
                assert_eq!(
                    m.cpu.regs[0],
                    u32::from(remap(&rom, key, message, 255)),
                    "raw remap key{key} msg{message}"
                );
                raw_cases += 1;
            }
        }
    }
    const DEST: u32 = 0x02030010;
    const SCRATCH: u32 = 0x02032000;
    const REBUILT: u32 = 0x02034000;
    let mut header_cases = 0;
    let mut copy_cases = 0;
    let mut decompressed_tail_cases = 0;
    for &(li, bi, pi) in &cases {
        let bank = &data.languages[li].banks[bi];
        let program = &bank.programs[pi];
        reset(&mut m, 0x08013fa0);
        m.cpu.regs[2] = 0x08000000 + program.source_offset;
        m.cpu.regs[4] = REBUILT;
        m.cpu.regs[5] = REBUILT + 4;
        steps += until(&mut m, 0x08013fc6, 24, &mut pcs);
        assert_eq!(m.bus.read8(REBUILT), (program.header >> 4) as u8 & 15);
        assert_eq!(
            (m.cpu.regs[4], m.cpu.regs[5]),
            (
                u32::from(program.header & 1),
                u32::from((program.header >> 1) & 1)
            )
        );
        header_cases += 1;
        let mut outputs = Vec::new();
        for rebuilt in [false, true] {
            m.load_state(&start).unwrap();
            reset(&mut m, 0x08013f8c);
            m.bus.write8(0x02001f6e, li as u8);
            // Existing nullable-context fallback in a controlled scratch reader case; retained checkpoint is untouched.
            m.bus.write32(0x020020d0, 0);
            m.bus.write16(0x020020dc, 1);
            for i in 0..4096 {
                m.bus.write8(DEST - 16 + i, 0x5a);
            }
            let (source, index) = if rebuilt {
                let mut bytes = vec![4, 0, 255, 255];
                bytes.extend_from_slice(&(program.header & !2).to_le_bytes());
                bytes.extend(program.program_bytes().unwrap());
                for (i, &v) in bytes.iter().enumerate() {
                    m.bus.write8(REBUILT + i as u32, v);
                }
                (REBUILT, 0)
            } else {
                (0x08000000 + bank.source_offset, pi as u32)
            };
            m.cpu.regs[..4].copy_from_slice(&[DEST, source, SCRATCH, index]);
            steps += until(&mut m, RETURN, 100000, &mut pcs);
            assert_eq!(m.cpu.regs[13], 0x03007c00);
            if !rebuilt && program.header & 2 != 0 {
                let mut expected = program.program_bytes().unwrap();
                expected.extend_from_slice(&program.decoded_padding);
                assert!(
                    (0..expected.len()).all(|i| m.bus.read8(SCRATCH + i as u32) == expected[i]),
                    "decompressed li{li} bi{bi} pi{pi}"
                );
                decompressed_tail_cases += 1;
            }
            outputs.push((
                m.cpu.regs[0],
                (0..4096)
                    .map(|i| m.bus.read8(DEST - 16 + i))
                    .collect::<Vec<_>>(),
            ));
        }
        let expected = if program.header & 2 != 0 && outputs[0].0 > 511 {
            outputs[0].0 - 512
        } else {
            outputs[0].0
        };
        assert_eq!(outputs[1].0, expected);
        assert_eq!(outputs[0].1, outputs[1].1, "copy li{li} bi{bi} pi{pi}");
        copy_cases += 1;
    }
    let result = serde_json::json!({"status":"PASS","source_bytes":SOURCE_BYTES,"source_ranges":5,"physical_banks":385,"programs":41010,"alignment_bytes":34767,"glyphs":147,"command_forms":11,"shape_guards":22,"coherent_source_guards":6,"dependency_guards":3,"json_guards":12,"rom_identity_guards":2,"original_language_cases":language_cases,"original_key_bank_cases":bank_cases,"original_live_quotient_cases":live_cases,"original_live_remainder_cases":remap_cases,"original_raw_special_message_cases":raw_cases,"original_u16_index_cases":index_cases,"original_header_cases":header_cases,"original_program_copy_cases":copy_cases,"original_compressed_body_and_tail_cases":decompressed_tail_cases,"instructions_coverage":steps,"pcs_coverage":pcs.len(),"scope":"new source model and original selector/index/header/expansion outputs; not native full-state/clock/ordinary reachability or whole text callback ABI","source_runtime_revision_unchanged":[1,51,50],"linux":"pending","usa":"pending","native_windows":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_SCRIPT_TEXT_MODEL {result}");
}
