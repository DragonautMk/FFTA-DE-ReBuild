use super::*;
use crate::rom_data::layered_obj_animations::Catalog;

#[test]
#[ignore = "private EUR layered OBJ runtime projection; Wine only"]
fn european_layered_obj_staged_runtime_projects_owned_bytes() {
    let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let spans = catalog.decoded.source_spans().unwrap();
    let owner = LayeredObjResources::new(catalog.decoded.clone(), &catalog.sources).unwrap();
    let mut poisoned = rom.clone();
    let mut owned = vec![false; rom.len()];
    for (at, bytes, _) in &spans {
        let start = *at as usize;
        poisoned[start..start + bytes.len()].fill(0);
        owned[start..start + bytes.len()].fill(true);
    }
    for (at, bytes, _) in &spans {
        for (i, &expected) in bytes.iter().enumerate() {
            assert_eq!(
                owner.guest_read::<1>(*at as usize + i, &poisoned, None),
                Some([expected])
            );
        }
        for start in [*at as usize - 1, *at as usize + bytes.len() - 1] {
            let prior = [0x5a; 4];
            let expected = std::array::from_fn(|i| {
                if owned[start + i] {
                    rom[start + i]
                } else {
                    prior[i]
                }
            });
            assert_eq!(
                owner
                    .guest_read::<4>(start, &poisoned, Some(prior))
                    .unwrap(),
                expected
            );
        }
    }
    for bank in &catalog.decoded.banks {
        for list in &bank.lists {
            assert_eq!(owner.host_list(list.source_offset), Some(list));
        }
    }
    assert!(owner.host_list(0).is_none());
    let stats = owner.stats();
    assert!(stats.descriptor_reads > 0 && stats.descriptor_bytes >= 448);
    assert!(stats.fields.source_bytes > 0 && stats.fields.host_reads >= 566);
    assert!(owner.resident_bytes() > 0);
    let mut bad_sources = catalog.sources.clone();
    let replacement = if bad_sources[0].sha256.starts_with('0') {
        "1"
    } else {
        "0"
    };
    bad_sources[0].sha256.replace_range(..1, replacement);
    assert!(LayeredObjResources::new(catalog.decoded, &bad_sources).is_err());
}
