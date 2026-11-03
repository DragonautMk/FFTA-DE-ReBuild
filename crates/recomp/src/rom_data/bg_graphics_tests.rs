//! Reduced EUR/Wine oracle for descriptor field00. Private outputs are ignored.
use super::*;
use gba_core::{
    rom_data::bg_graphics::{self, Catalog, Graphic, Graphics},
    Bus,
};

#[test]
fn graphics_json_preserves_words_and_refuses_unknown_or_incoherent_fields() {
    let mut source = (32u32 << 8 | 0x20).to_le_bytes().to_vec();
    source.extend_from_slice(&32u32.to_be_bytes());
    source.extend_from_slice(&[1, 29]);
    let value = Graphics {
        descriptor_graphics: vec![0],
        graphics: vec![Graphic::decode(&source, 0x1000).unwrap().0],
    };
    let encoded = serde_json::to_vec_pretty(&value).unwrap();
    let parsed: Graphics = serde_json::from_slice(&encoded).unwrap();
    parsed.validate().unwrap();
    assert_eq!(parsed, value);
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), encoded);
    let mut unknown = serde_json::to_value(&value).unwrap();
    unknown["graphics"][0]["extra"] = true.into();
    assert!(serde_json::from_value::<Graphics>(unknown).is_err());
    let mut inconsistent = value.clone();
    inconsistent.graphics[0].words[0] ^= 0x8000_0000;
    assert!(inconsistent.validate().is_err());
}

#[test]
#[ignore = "requires exact EUR ROM and absent FFTA_STATE_TEST_MAIN_GRAPHICS_ROOT; Wine-only new checks"]
fn european_main_graphics_match_original_staged_decoder_and_dma_layout() {
    const RETURN: u32 = 0x0800_0300;
    const STACK: u32 = 0x0300_7c00;
    const TASK: u32 = 0x0300_2000;
    const DEST: usize = 0x91a0;
    const ENTRY: u32 = 0x0801_b688;
    const CONSUMER: u32 = 0x0800_54a0;
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_MAIN_GRAPHICS_ROOT").unwrap());
    assert!(!root.exists(), "preserve previous oracle evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let catalog = Catalog::from_rom(&rom).unwrap();
    let descriptors = gba_core::rom_data::bg_descriptors::Catalog::from_rom(&rom).unwrap();
    assert_eq!(catalog.decoded.descriptor_graphics.len(), 162);
    assert_eq!(catalog.decoded.graphics.len(), 50);
    let encoded = serde_json::to_vec_pretty(&catalog.decoded).unwrap();
    let parsed: Graphics = serde_json::from_slice(&encoded).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), encoded);
    let mut forged = parsed.clone();
    let original = &parsed.graphics[0];
    forged.graphics[0] = (8..original.encoded_source.len())
        .find_map(|index| {
            let mut source = original.encoded_source.clone();
            source[index] ^= 0x80;
            let (candidate, consumed) = Graphic::decode(&source, original.relative_source).ok()?;
            (consumed == source.len() && candidate.words != original.words).then_some(candidate)
        })
        .expect("coherent same-extent literal edit");
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bg-main-graphics.json"), &encoded).unwrap();
    let mut machine = crate::make_machine(rom, None);
    let mut calls = 0u64;
    let mut instructions = 0u64;
    let mut reports = Vec::new();
    let mut tags = [0usize; 2];
    for (id, (graphic, source)) in catalog
        .decoded
        .graphics
        .iter()
        .zip(&catalog.sources)
        .enumerate()
    {
        let descriptor = catalog
            .decoded
            .descriptor_graphics
            .iter()
            .position(|&g| g as usize == id)
            .unwrap();
        assert_eq!(
            descriptors.decoded.entries[descriptor].graphics_source,
            graphic.relative_source
        );
        assert_eq!(
            source.offset,
            descriptors.source.offset + graphic.relative_source
        );
        assert_eq!(source.length as usize, graphic.encoded_source.len());
        assert_eq!(
            source.sha256,
            gba_core::ffta_rom::digest(&graphic.encoded_source)
        );
        let body = if graphic.encoded_source[0] == 0x22 {
            tags[1] += 1;
            8
        } else {
            tags[0] += 1;
            4
        };
        let expected = graphic.source_bytes();
        machine.bus.ewram.fill(0x5a);
        machine.bus.iwram.fill(0xcd);
        machine.bus.write16(0x0200_7f10, descriptor as u16);
        machine.bus.write32(0x0200_7f50, 0x0600_0000);
        machine.bus.write32(0x0200_7f54, 0x0600_6000);
        machine.bus.write32(0x0200_7f58, 0x0600_8000);
        machine.bus.write8(TASK, 1);
        let mut cursor = None;
        let mut entered = 0;
        let mut invoke = |machine: &mut gba_core::Machine| {
            machine.cpu.regs[1] = TASK;
            machine.cpu.regs[13] = STACK;
            machine.cpu.regs[14] = RETURN | 1;
            machine.cpu.regs[15] = ENTRY;
            machine.cpu.cpsr = 0xbf;
            for _ in 0..2_000_000 {
                machine.bus.frame_ready = false;
                if machine.cpu.regs[15] == CONSUMER {
                    entered += 1;
                    assert_eq!(machine.cpu.regs[0], 0x0200_0000 + DEST as u32);
                    assert_eq!(machine.cpu.regs[1], 0x0800_0000 + source.offset + body);
                }
                if machine.cpu.regs[15] == CONSUMER + 0x43e {
                    cursor = Some(machine.cpu.regs[7]);
                }
                machine.step();
                instructions += 1;
                if machine.cpu.regs[15] == RETURN {
                    assert_eq!(machine.cpu.regs[13], STACK);
                    calls += 1;
                    return;
                }
            }
            panic!("staged original failed to return for graphic {id}");
        };
        invoke(&mut machine);
        assert_eq!(machine.bus.read8(TASK), 2);
        assert_eq!(&machine.bus.ewram[DEST..DEST + expected.len()], &expected);
        assert_eq!(&machine.bus.ewram[DEST - 16..DEST], &[0x5a; 16]);
        assert!(machine.bus.ewram
            [DEST + expected.len()..DEST + bg_graphics::DESTINATION_BYTES + 16]
            .iter()
            .all(|&b| b == 0x5a));
        // Two representative staged transfers cover tag20 and tag22 without
        // rerunning their decoder: each phase queues one DMA32 half-buffer.
        if descriptor == 0 || descriptor == 19 {
            machine.bus.write32(0x0300_2bcc, 0);
            invoke(&mut machine);
            assert_eq!(machine.bus.read8(TASK), 3);
            assert_eq!(machine.bus.read32(0x0200_7eb4), 0x0200_0000 + DEST as u32);
            assert_eq!(
                machine.bus.read32(0x0200_7eb8),
                0x0600_0000 + graphic.destination_offset
            );
            assert_eq!(
                machine.bus.read32(0x0200_7ebc),
                0x8400_0000 | (expected.len() as u32 / 8)
            );
            machine.bus.write32(0x0300_2bcc, 0);
            invoke(&mut machine);
            assert_eq!(machine.bus.read8(TASK), 4);
            assert_eq!(
                machine.bus.read32(0x0200_7ed0),
                0x0200_0000 + DEST as u32 + expected.len() as u32 / 2
            );
            assert_eq!(
                machine.bus.read32(0x0200_7ed4),
                0x0600_0000 + graphic.destination_offset + expected.len() as u32 / 2
            );
            assert_eq!(
                machine.bus.read32(0x0200_7ed8),
                0x8400_0000 | (expected.len() as u32 / 8)
            );
        }
        drop(invoke);
        assert_eq!(entered, 1);
        assert_eq!(cursor, Some(0x0800_0000 + source.offset + source.length));
        let file = format!("graphic-{id}.bin");
        fs::write(root.join(&file), &expected).unwrap();
        reports.push(
            serde_json::json!({"id":id,"descriptor":descriptor,"tag":graphic.encoded_source[0],
            "offset":source.offset,"encoded_bytes":source.length,"source_sha256":source.sha256,
            "destination_offset":graphic.destination_offset,"decoded_bytes":expected.len(),
            "decoded_sha256":gba_core::ffta_rom::digest(&expected),"file":file}),
        );
    }
    assert_eq!(tags, [23, 27]);
    assert_eq!(calls, 54);
    let report = serde_json::json!({"region":"europe","descriptors":162,"unique_sources":50,
        "tag20":tags[0],"tag22":tags[1],"original_calls":calls,"original_instructions":instructions,
        "complete_outputs":true,"exact_cursors":true,"guards":true,"dma_queue_layout":true,
        "json_sha256":gba_core::ffta_rom::digest(&encoded),"sources":reports,
        "linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_MAIN_GRAPHICS descriptors=162 sources=50 calls={calls} instructions={instructions} outputs/cursors/guards/DMA-layout=pass");
}
