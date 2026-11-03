//! New custom UI banks: original decoder, returned-length and upload contract.
use super::*;
use gba_core::{
    rom_data::ui_graphics::{Catalog, Graphic, Graphics, REFERENCES},
    Bus, Machine,
};
pub(super) const SCRATCH: u32 = 0x02020010;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
pub(super) fn setup(m: &mut Machine, row: usize) {
    m.cpu.regs.fill(0);
    for i in [4, 5, 6, 7, 10] {
        m.cpu.regs[i] = SCRATCH;
    }
    if REFERENCES[row].call == 0x143f0c {
        // This second upload inherits r5 from the preceding original literal.
        m.cpu.regs[5] = u32::from_le_bytes(m.bus.rom[0x143f6c..0x143f70].try_into().unwrap());
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = 0x08000000 + REFERENCES[row].reader;
    for i in 0..32800 {
        m.bus.write8(SCRATCH - 16 + i, 0x5a);
    }
    m.bus.vram.fill(0x5a);
}
#[test]
#[ignore = "EUR custom UI original source/decode/returned-count/upload; Wine only, Linux/USA pending"]
fn european_ui_custom_graphics_match_original_decode_and_upload() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_GRAPHICS_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let graphics = &catalog.decoded;
    assert_eq!(
        (
            graphics.banks.len(),
            REFERENCES.len(),
            catalog.sources.len()
        ),
        (9, 10, 9)
    );
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 57038);
    assert_eq!(
        graphics
            .banks
            .iter()
            .map(|b| b.words.len() * 4)
            .sum::<usize>(),
        84160
    );
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UI_GRAPHICS_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    for (i, b) in graphics.banks.iter().enumerate() {
        assert_eq!(inv["banks"][i]["source_offset"], b.source_offset);
        assert_eq!(inv["banks"][i]["source_length"], b.encoded_source.len());
        assert_eq!(
            inv["banks"][i]["source_sha256"],
            gba_core::ffta_rom::digest(&b.encoded_source)
        );
        assert_eq!(
            inv["banks"][i]["decoded_sha256"],
            gba_core::ffta_rom::digest(&b.decoded_bytes())
        );
    }
    let bytes = serde_json::to_vec_pretty(graphics).unwrap();
    let parsed: Graphics = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..7 {
        let mut bad = graphics.clone();
        match kind {
            0 => bad.banks.clear(),
            1 => bad.banks[0].words[0] ^= 1,
            2 => bad.banks[0].encoded_source.push(0),
            3 => {
                bad.banks[0].encoded_source.pop();
            }
            4 => bad.banks[0].source_offset = u32::MAX,
            5 => bad.banks.swap(0, 1),
            _ => bad.banks[1].source_offset = bad.banks[0].source_offset,
        };
        assert!(bad.validate().is_err());
    }
    let mut coherent = graphics.clone();
    let b = &graphics.banks[0];
    coherent.banks[0] = (4..b.encoded_source.len())
        .find_map(|at| {
            let mut v = b.encoded_source.clone();
            v[at] ^= 1;
            Graphic::decode(&v, b.source_offset).ok().filter(|v| v != b)
        })
        .unwrap();
    coherent.validate().unwrap();
    assert!(coherent.verify_source(&rom).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    unknown["banks"][0]["unknown"] = 0.into();
    assert!(serde_json::from_value::<Graphics>(unknown).is_err());
    let mut m = crate::make_machine(rom.clone(), None);
    let mut instructions = 0;
    for (row, r) in REFERENCES.iter().enumerate() {
        let bank = graphics
            .banks
            .iter()
            .find(|b| b.source_offset == r.source_offset)
            .unwrap();
        let decoded = bank.decoded_bytes();
        setup(&mut m, row);
        let stop = 0x08000000 + r.transfer_call + 4;
        let mut terminal = false;
        let mut transfer = false;
        for _ in 0..2_000_000 {
            if m.cpu.regs[15] == 0x080058de {
                assert_eq!(
                    m.cpu.regs[7],
                    0x08000000 + r.source_offset + bank.encoded_source.len() as u32
                );
                assert_eq!(m.cpu.regs[9], decoded.len() as u32);
                terminal = true;
            }
            if m.cpu.regs[15] == 0x08000000 + r.call + 4 {
                assert_eq!(
                    m.cpu.regs[0],
                    decoded.len() as u32,
                    "original declared return length"
                );
            }
            if m.cpu.regs[15] == 0x08000000 + r.transfer_call {
                assert_eq!(
                    &m.cpu.regs[..3],
                    &[SCRATCH + r.decoded_offset, r.destination, r.control]
                );
                transfer = true;
            }
            m.bus.frame_ready = false;
            m.step();
            instructions += 1;
            if m.cpu.regs[15] == stop {
                break;
            }
        }
        assert_eq!(m.cpu.regs[15], stop);
        assert!(terminal && transfer);
        assert!((0..decoded.len()).all(|i| m.bus.read8(SCRATCH + i as u32) == decoded[i]));
        assert!((0..16).all(|i| m.bus.read8(SCRATCH - 16 + i) == 0x5a));
        assert!((decoded.len()..32784).all(|i| m.bus.read8(SCRATCH + i as u32) == 0x5a));
        let length = if r.transfer_target == 0x14522c {
            (((r.control & 0x1fffff) + 7) & !7) * 4
        } else {
            (r.control & 0x1fffff) * 2
        } as usize;
        let mut expected = vec![0x5a; 0x18000];
        let dest = (r.destination - 0x06000000) as usize;
        expected[dest..dest + length].copy_from_slice(
            &decoded[r.decoded_offset as usize..r.decoded_offset as usize + length],
        );
        assert_eq!(m.bus.vram.as_slice(), expected);
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ui-graphics.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"banks":9,"references":10,"ranges":9,"source_bytes":57038,"decoded_bytes":84160,"original_decode_upload_paths":10,"instructions":instructions,"exact_source_cursor_returned_size_original_controls_scratch_guards_whole_vram":true,"partial544_of2944_preserved":true,"strict_json_source_coherent_edit_guards":true,"runtime":"offline only; managed/consumption pending","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
