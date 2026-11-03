//! EUR conditional bank-zero source model and original mode-3 path.
use super::*;
use gba_core::rom_data::conditional_descriptor_bank_zero::{BankZero,Catalog,SOURCE_SPANS};
use gba_core::{Bus,Machine};

#[test]
#[ignore = "private EUR source and original reader check; Wine only, Linux/USA pending"]
fn european_conditional_descriptor_bank_zero_original_path() {
    const STOP:u32=0x03007f00;const UNIT:u32=0x02010000;const OUT:u32=0x02011000;
    let root=PathBuf::from(std::env::var_os("FFTA_DESCRIPTOR_ZERO_ROOT").unwrap());assert!(!root.exists());fs::create_dir_all(&root).unwrap();
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();let catalog=Catalog::from_rom(&rom).unwrap();let model=&catalog.decoded;model.verify_source(&rom).unwrap();assert_eq!(catalog.sources.len(),2);assert_eq!(model.records.len(),15);
    let bytes=model.source_bytes().unwrap();for (i,(offset,length,_,_)) in SOURCE_SPANS.into_iter().enumerate() {assert_eq!(bytes[i],rom[offset as usize..(offset+length) as usize]);for at in offset..offset+length {assert_eq!(model.byte_at(at),Some(rom[at as usize]));}}
    for at in [0x6df4d3,0x6df4d8,0x6df534,0x6df53b,0x6df5b4] {assert_eq!(model.byte_at(at),None,"unowned {at:06x}");}
    assert!(model.select(0).is_none());assert!(model.select(1).is_some());assert!(model.select(15).is_some());assert!(model.select(16).is_none());
    let json=serde_json::to_vec(model).unwrap();assert_eq!(serde_json::from_slice::<BankZero>(&json).unwrap(),*model);
    let mut bad=model.clone();bad.records.pop();assert!(bad.validate().is_err());bad=model.clone();bad.bank_root+=8;assert!(bad.validate().is_err());bad=model.clone();bad.first_row=0;assert!(bad.validate().is_err());bad=model.clone();bad.records[0].field0^=1;assert!(bad.validate().is_ok());assert!(bad.verify_source(&rom).is_err());
    assert!(serde_json::from_str::<BankZero>("{\"pointer_offset\":0,\"bank_root\":0,\"first_row\":1,\"records\":[],\"unexpected\":true}").is_err());
    fn reset(m:&mut Machine,pc:u32) {m.bus.ffta_sprites.enabled=false;m.cpu.regs=[0;16];m.cpu.regs[13]=0x03007c00;m.cpu.regs[14]=STOP|1;m.cpu.regs[15]=pc;m.cpu.cpsr=0xbf;}
    fn run_to(m:&mut Machine,end:u32,max:usize) {let mut n=0;while m.cpu.regs[15]!=end {assert!(n<max,"pc {:08x}, end {:08x}",m.cpu.regs[15],end);m.bus.frame_ready=false;m.step();n+=1;}}
    let mut m=crate::make_machine(rom,None);
    for job in [None,Some(23u8),Some(24),Some(25)] {
        reset(&mut m,0x080cbb80);m.bus.write8(UNIT+7,job.unwrap_or(0));m.cpu.regs[0]=if job.is_some(){UNIT}else{0};run_to(&mut m,STOP,32);assert_eq!(m.cpu.regs[0],u32::from(job==Some(24)),"predicate {job:?}");
    }
    for job in [23u8,24,25] {
        reset(&mut m,0x080cd9e8);m.bus.write8(UNIT+6,1);m.bus.write8(UNIT+7,job);m.bus.write8(UNIT+0x37,0xa5);m.cpu.regs[0]=UNIT;run_to(&mut m,0x080cda48,300);
        assert_eq!(m.bus.read8(UNIT+0x37),if job==24 {1} else {0xa5},"initializer job {job}");
    }
    for slot in [0u8,1] {
        reset(&mut m,0x080d08b4);m.bus.write8(UNIT+0x37,slot);m.bus.write8(OUT,0xa5);m.bus.write8(OUT+1,0xa5);m.cpu.regs[0]=UNIT;m.cpu.regs[1]=3;m.cpu.regs[2]=OUT;m.cpu.regs[3]=OUT+1;run_to(&mut m,STOP,1000);
        assert_eq!(m.cpu.regs[0],0x086df534);assert_eq!(m.bus.read8(OUT),1);assert_eq!(m.bus.read8(OUT+1),15);
        for field in [251u32,265] {
            reset(&mut m,0x081379f0);m.bus.write8(UNIT+6,1);m.bus.write8(UNIT+7,24);m.bus.write8(UNIT+8,0);m.bus.write8(UNIT+0x35,0);m.bus.write8(UNIT+0x36,0);m.bus.write8(UNIT+0x37,slot);m.cpu.regs[0]=UNIT;m.cpu.regs[1]=field;run_to(&mut m,STOP,2000);
            assert_eq!(m.cpu.regs[0],u32::from(slot!=0),"scan slot {slot} field {field}");
        }
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","records":15,"source_bytes":124,"predicate_cases":4,"initializer_cases":3,"mode3_slots":[0,1],"scan_fields":[251,265],"record0_raw":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
