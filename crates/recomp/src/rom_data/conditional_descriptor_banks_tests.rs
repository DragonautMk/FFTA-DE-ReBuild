//! EUR conditional descriptor banks and original count/address producers.
use super::*;
use gba_core::rom_data::conditional_descriptor_banks::{Catalog,DescriptorBanks,BANK_COUNTS,BANK_OFFSETS,SOURCE_SPANS,TABLE_ROOT};
use gba_core::{Bus,Machine};

#[test]
#[ignore = "private EUR source and original reader check; Wine only, Linux/USA pending"]
fn european_conditional_descriptor_banks_original_producers() {
    const STOP:u32=0x03007f00;const UNIT:u32=0x02010000;
    let root=PathBuf::from(std::env::var_os("FFTA_DESCRIPTOR_BANKS_ROOT").unwrap());assert!(!root.exists());fs::create_dir_all(&root).unwrap();
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();let catalog=Catalog::from_rom(&rom).unwrap();let model=&catalog.decoded;model.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(),3);assert_eq!(model.banks.len(),23);assert_eq!(model.banks.iter().map(|b|b.records.len()).sum::<usize>(),684);
    let reconstructed=model.source_bytes().unwrap();
    for (i,(offset,length,_,_)) in SOURCE_SPANS.into_iter().enumerate() {
        assert_eq!(reconstructed[i],rom[offset as usize..(offset+length) as usize]);
        for at in offset..offset+length {assert_eq!(model.byte_at(at),Some(rom[at as usize]),"byte {at:06x}");}
    }
    for at in [TABLE_ROOT,0x6df534,0x6e0acc,0x6e0afb,0x6e0b44] {assert_eq!(model.byte_at(at),None,"unowned {at:06x}");}
    let bytes=serde_json::to_vec(model).unwrap();assert_eq!(serde_json::from_slice::<DescriptorBanks>(&bytes).unwrap(),*model);
    let mut bad=model.clone();bad.banks[0].records.pop();assert!(bad.validate().is_err());
    bad=model.clone();bad.banks[0].source_offset+=8;assert!(bad.validate().is_err());
    bad=model.clone();bad.banks[0].index=0;assert!(bad.validate().is_err());
    bad=model.clone();bad.banks[0].records[0].field0^=1;assert!(bad.validate().is_ok());assert!(bad.verify_source(&rom).is_err());
    assert!(serde_json::from_str::<DescriptorBanks>("{\"pointer_offset\":0,\"banks\":[],\"unexpected\":true}").is_err());
    fn reset(m:&mut Machine,pc:u32) {m.bus.ffta_sprites.enabled=false;m.cpu.regs=[0;16];m.cpu.regs[13]=0x03007c00;m.cpu.regs[14]=STOP|1;m.cpu.regs[15]=pc;m.cpu.cpsr=0xbf;}
    fn run_to(m:&mut Machine,end:u32,max:usize) {let mut n=0;while m.cpu.regs[15]!=end {assert!(n<max,"pc {:08x}, end {:08x}",m.cpu.regs[15],end);m.bus.frame_ready=false;m.step();n+=1;}}
    let mut m=crate::make_machine(rom.clone(),None);
    // Actual original count dispatcher covers every declared bank; zero/24/255
    // return zero in this producer, without placing any guard in the guest getter.
    for bank in 0u32..=24 {
        reset(&mut m,0x080cd938);m.cpu.regs[0]=bank;run_to(&mut m,STOP,32);
        let expected=if (1..=23).contains(&bank) {BANK_COUNTS[bank as usize-1] as u32} else {0};
        assert_eq!(m.cpu.regs[0],expected,"count bank {bank}");
    }
    for bank in [0u32,1,22,23,24,255,256] {
        // Original initializer stores the count at unit+34 after its getters.
        reset(&mut m,0x080cd9e8);m.bus.write8(UNIT+6,bank as u8);m.bus.write8(UNIT+0x34,0xa5);m.cpu.regs[0]=UNIT;m.cpu.regs[1]=0;
        run_to(&mut m,0x080cda10,200);
        let expected=if (1..=23).contains(&(bank as u8)) {BANK_COUNTS[bank as u8 as usize-1] as u8} else {0};
        assert_eq!(m.bus.read8(UNIT+0x34),expected,"stored bank {bank}");
    }
    for (bank,row) in [(0u32,0u32),(1,0),(1,141),(22,10),(22,11),(22,16),(23,8),(23,9),(24,0),(255,255),(257,256)] {
        reset(&mut m,0x080d0ed4);m.cpu.regs[0]=bank;m.cpu.regs[1]=row;run_to(&mut m,STOP,16);
        let pointer_offset=DescriptorBanks::encoded_pointer_view(bank);
        let po=(pointer_offset-0x08000000) as usize;let ptr=u32::from_le_bytes(rom[po..po+4].try_into().unwrap());
        let expected=DescriptorBanks::encoded_record_view(ptr,row);
        assert_eq!(m.cpu.regs[0],expected,"getter bank {bank} row {row}");
        if let Some(record)=model.select(bank as u8,row as u8) {let off=(expected-0x08000000) as usize;assert_eq!(record.bytes(),rom[off..off+8]);}
    }
    assert!(model.select(0,0).is_none());assert!(model.select(22,11).is_none());assert!(model.select(23,9).is_none());
    assert_eq!(BANK_OFFSETS[21]+11*8,0x6e0acc);assert_eq!(BANK_OFFSETS[22],0x6e0afc);
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","banks":23,"records":684,"source_bytes":5564,"count_banks":25,"initializer_cases":7,"getter_cases":11,"unowned_gap_bytes":48,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
