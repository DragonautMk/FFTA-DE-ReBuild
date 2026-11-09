//! Bounded, session-local observations. Never perform guest bus reads or writes.
use gba_core::{debug_memory, Machine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MAX_PINS: usize = 16;
const MAX_WINDOW: u32 = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag="kind", rename_all="kebab-case", deny_unknown_fields)]
pub(crate) enum Target {
    Register { index: u8 },
    Memory { address: u32, length: u32 },
    Field { family: String, field: String, context: u32 },
}

/// These are the same raw context offsets described by the existing gameplay
/// inspector, not a claim that a context still owns an actor or task.
fn field_layout(family:&str,field:&str)->Option<(u32,u32,&'static str)> {
    match (family,field) {
        ("task","callback")=>Some((0,4,"docs/rom_data_tasks.md")),
        ("task","context-size")=>Some((4,2,"docs/rom_data_tasks.md")),
        ("task","metadata")=>Some((6,2,"docs/rom_data_tasks.md")),
        ("script","cursor")=>Some((0,2,"docs/rom_data_scripts.md")),
        ("script","stream")=>Some((12,4,"docs/rom_data_scripts.md")),
        ("aux-script","action")=>Some((0,1,"docs/rom_data_scripts.md")),
        ("aux-script","slot")=>Some((1,1,"docs/rom_data_scripts.md")),
        ("aux-script","cursor")=>Some((2,2,"docs/rom_data_scripts.md")),
        ("aux-script","stream")=>Some((4,4,"docs/rom_data_scripts.md")),
        ("unit","flags")=>Some((0,4,"docs/rom_data_unit_obj.md")),
        ("unit","sprite")=>Some((6,2,"docs/rom_data_unit_obj.md")),
        ("unit","countdown")=>Some((10,2,"docs/rom_data_unit_obj.md")),
        ("unit","sequence-cursor")=>Some((12,2,"docs/rom_data_unit_obj.md")),
        ("unit","frame-count")=>Some((14,2,"docs/rom_data_unit_obj.md")),
        ("unit","marker")=>Some((0x20,4,"docs/rom_data_unit_obj.md")),
        ("unit","sequence-frames")=>Some((0x34,4,"docs/rom_data_unit_obj.md")),
        ("unit","current-frame")=>Some((0x38,4,"docs/rom_data_unit_obj.md")),
        _=>None,
    }
}
fn name_valid(s:&str)->bool { !s.is_empty() && s.len()<=48 && !s.chars().any(char::is_control) }
fn observed_context(m:&Machine,family:&str,context:u32)->bool {
    if !m.cpu.thumb(){return false;}
    let register=match (family,m.cpu.regs[15]) {
        ("task",0x080d7062|0x080d7064|0x080d70a2|0x080d7346|0x080d7348)=>0,
        ("task",0x080d707a|0x080d707c|0x080d76b6)=>1,
        ("script",0x081265d4|0x081265fa|0x08126610)=>5,
        ("aux-script",0x0800a248|0x0800a274|0x0800a286)=>5,
        ("unit",0x08022400|0x08022e20)=>0,
        ("unit",0x08022488|0x080224e2)=>4,
        _=>return false,
    };
    m.cpu.regs[register]==context
}
impl Target {
    pub fn validate(&self)->Result<(),String> {
        match self {
            Self::Register{index} if *index<16=>Ok(()),
            Self::Register{..}=>Err("Register must be r0..r15".into()),
            Self::Memory{address,length} if (1..=MAX_WINDOW).contains(length) && address.checked_add(length-1).is_some()=>Ok(()),
            Self::Memory{..}=>Err("Memory pin needs 1..32 bytes without address wrap".into()),
            Self::Field{family,field,context}=>{
                let (offset,width,_)=field_layout(family,field).ok_or("Unknown decoded field")?;
                let end=context.checked_add(offset).and_then(|a|a.checked_add(width-1)).ok_or("Context address wraps")?;
                let ram=if family=="task" { (0x08000000..=0x09ffffff).contains(context) && end<=0x09ffffff }
                    else { (0x02000000..=0x0203ffff).contains(context) && end<=0x0203ffff || (0x03000000..=0x03007fff).contains(context) && end<=0x03007fff };
                if !ram{return Err("Field context must be canonical RAM, or ROM for a task descriptor".into());}Ok(())
            }
        }
    }
    fn observe(&self,m:&Machine)->Result<(Value,Value),String>{
        match self {
            Self::Register{index}=>Ok((json!(m.cpu.regs[*index as usize]),json!({"kind":"register","register":index,"pc":m.cpu.regs[15]}))),
            Self::Memory{address,length}=>{
                let bytes=debug_memory::peek(m,*address,*length)?;
                let backing=bytes.iter().map(|b|json!([b.region,b.offset])).collect::<Vec<_>>();
                Ok((json!(bytes.iter().map(|b|b.value).collect::<Vec<_>>()),json!({"kind":"readonly-backing","address":address,"length":length,"backing":backing,"pc":m.cpu.regs[15],"scope":"Per-byte backing region and offset; no bus or I/O register reads"})))
            },
            Self::Field{family,field,context}=>{
                let (offset,width,guide)=field_layout(family,field).ok_or("Unknown decoded field")?;
                let address=context.checked_add(offset).ok_or("Context address wraps")?;
                let bytes=debug_memory::peek(m,address,width)?;
                let value=bytes.iter().enumerate().fold(0u32,|n,(i,b)|n|(u32::from(b.value)<<(8*i)));
                Ok((json!(value),json!({"kind":"decoded-context-field","family":family,"field":field,"context":context,"offset":offset,"address":address,"width":width,"bytes":bytes,"pc":m.cpu.regs[15],"guide":guide,"scope":"Observed backing at a supplied context; lifetime and identity are not inferred"})))
            }
        }
    }
}

struct Pin { label:String, owner:String, target:Target, current:Value, previous:Option<Value>, previous_source:Option<Value>, previous_frame:Option<u64>, previous_clock:Option<u64>, origin:Value, source:Value, frame:u64, clock:u64, stale:bool, error:Option<String> }
#[derive(Default)]
pub(crate) struct Pins { items:Vec<Pin>, filter:String, offset:usize }
impl Pins {
    pub fn add(&mut self,m:&Machine,label:String,owner:String,target:Target)->Result<(),String>{
        if !name_valid(&label)||!name_valid(&owner){return Err("Label and owner need 1..48 bytes without control characters".into());}
        if self.items.len()>=MAX_PINS{return Err("At most 16 pins per session".into());}
        if self.items.iter().any(|p|p.label==label){return Err("Pin label already exists".into());}
        target.validate()?;
        if matches!(target,Target::Field{..}) && gba_core::ffta_rom::digest(&m.bus.rom)!=gba_core::ffta_rom::EUROPE_SHA256{return Err("Decoded fields require the authenticated EUR ROM".into());}
        if let Target::Field{family,context,..}=&target {if !observed_context(m,family,*context){return Err("Pause at the matching gameplay-inspector site with the observed context register".into());}}
        let (current,source)=target.observe(m)?;
        self.items.push(Pin{label,owner,target,current,previous:None,previous_source:None,previous_frame:None,previous_clock:None,origin:source.clone(),source,frame:m.bus.frames,clock:m.bus.clock,stale:false,error:None});Ok(())
    }
    pub fn remove(&mut self,label:&str)->Result<(),String>{let i=self.items.iter().position(|p|p.label==label).ok_or("Unknown pin")?;self.items.remove(i);Ok(())}
    pub fn set_view(&mut self,filter:&str,offset:usize)->Result<(),String>{
        if !["all","changed","stale"].contains(&filter){return Err("Filter must be all, changed or stale".into());}
        if offset>=MAX_PINS{return Err("Pin offset must be 0..15".into());}
        self.filter=filter.into();self.offset=offset;Ok(())
    }
    pub fn refresh(&mut self,m:&Machine){
        for pin in &mut self.items {if pin.stale{continue;}match pin.target.observe(m){
            Ok((current,source))=>{pin.previous=Some(std::mem::replace(&mut pin.current,current));pin.previous_source=Some(std::mem::replace(&mut pin.source,source));pin.previous_frame=Some(pin.frame);pin.previous_clock=Some(pin.clock);pin.frame=m.bus.frames;pin.clock=m.bus.clock;pin.error=None;},
            Err(error)=>{pin.stale=true;pin.error=Some(error);}
        }}
    }
    pub fn invalidate(&mut self,reason:&str){for pin in &mut self.items {pin.stale=true;pin.error=Some(reason.into());}}
    pub fn status(&self)->Value {
        let pins:Vec<Value>=self.items.iter().map(|p|json!({"label":p.label,"owner":p.owner,"target":p.target,"value":p.current,"previous":p.previous,"previous_source":p.previous_source,"previous_frame":p.previous_frame,"previous_clock":p.previous_clock,"changed":p.previous.as_ref().is_some_and(|v|v!=&p.current),"origin":p.origin,"source":p.source,"frame":p.frame,"clock":p.clock,"stale":p.stale,"reason":p.error})).collect();
        json!({"schema":1,"pins":pins,"filter":if self.filter.is_empty(){"all"}else{&self.filter},"offset":self.offset,"scope":"Session-local observations; addresses and context are not persistent identities"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inspection_tracks_change_without_guest_mutation_and_invalidates_context(){
        let mut m=Machine::new(vec![0;16]);m.cpu.regs[3]=5;m.bus.ewram[4]=7;
        let mut pins=Pins::default();let before=m.save_state().unwrap();
        pins.add(&m,"register".into(),"test".into(),Target::Register{index:3}).unwrap();
        pins.add(&m,"window".into(),"test".into(),Target::Memory{address:0x02000004,length:2}).unwrap();
        assert_eq!(before,m.save_state().unwrap());
        m.cpu.regs[3]=9;m.bus.ewram[4]=8;m.bus.frames=2;m.bus.clock=123;let before=m.save_state().unwrap();pins.refresh(&m);assert_eq!(before,m.save_state().unwrap());
        let status=pins.status();assert_eq!(status["pins"][0]["previous"],5);assert_eq!(status["pins"][0]["value"],9);assert_eq!(status["pins"][1]["previous"][0],7);assert_eq!(status["pins"][1]["value"][0],8);assert_eq!(status["pins"][1]["source"]["address"],0x02000004u32);assert_eq!(status["pins"][1]["previous_source"]["address"],0x02000004u32);assert_eq!(status["pins"][1]["frame"],2);assert_eq!(status["pins"][1]["clock"],123);
        pins.invalidate("state-restored");m.bus.ewram[4]=10;pins.refresh(&m);let status=pins.status();assert_eq!(status["pins"][1]["stale"],true);assert_eq!(status["pins"][1]["value"][0],8);assert_eq!(status["pins"][1]["reason"],"state-restored");
    }
    #[test]
    fn invalid_or_unsafe_pins_are_refused_atomically(){
        let m=Machine::new(vec![0;16]);let before=m.save_state().unwrap();let mut p=Pins::default();
        for target in [Target::Register{index:16},Target::Memory{address:0x04000000,length:2},Target::Memory{address:u32::MAX,length:2},Target::Field{family:"unit".into(),field:"countdown".into(),context:0x02000000}] {assert!(p.add(&m,"x".into(),"owner".into(),target).is_err());}
        assert!(p.status()["pins"].as_array().unwrap().is_empty());assert_eq!(before,m.save_state().unwrap());
    }
    #[test]
    fn maximum_pin_status_is_bounded_and_removal_is_explicit(){
        let m=Machine::new(vec![0;16]);let mut pins=Pins::default();
        for i in 0..16{pins.add(&m,format!("pin-{i}"),"owner".into(),Target::Memory{address:0x02000000,length:32}).unwrap();}
        assert!(pins.add(&m,"overflow".into(),"owner".into(),Target::Register{index:0}).is_err());pins.refresh(&m);
        assert!(serde_json::to_vec(&pins.status()).unwrap().len()<48*1024);pins.remove("pin-4").unwrap();assert_eq!(pins.status()["pins"].as_array().unwrap().len(),15);
    }
    #[test]
    #[ignore = "requires an explicitly protected EUR ROM fixture"]
    fn decoded_fields_require_the_authenticated_observed_context(){
        let rom=std::fs::read(std::env::var_os("FFTA_PIN_TEST_ROM").expect("protected EUR ROM")).unwrap();
        let mut m=Machine::new(rom);m.cpu.cpsr|=1<<5;m.cpu.regs[15]=0x08022488;m.cpu.regs[4]=0x02000000;m.bus.ewram[10..12].copy_from_slice(&7u16.to_le_bytes());
        let mut p=Pins::default();let before=m.save_state().unwrap();
        p.add(&m,"countdown".into(),"test".into(),Target::Field{family:"unit".into(),field:"countdown".into(),context:0x02000000}).unwrap();assert_eq!(before,m.save_state().unwrap());
        m.bus.ewram[10..12].copy_from_slice(&6u16.to_le_bytes());let before=m.save_state().unwrap();p.refresh(&m);assert_eq!(before,m.save_state().unwrap());
        let status=p.status();assert_eq!(status["pins"][0]["previous"],7);assert_eq!(status["pins"][0]["value"],6);assert_eq!(status["pins"][0]["origin"]["pc"],0x08022488u32);assert_eq!(status["pins"][0]["source"]["address"],0x0200000au32);
        assert!(p.add(&m,"wrong".into(),"test".into(),Target::Field{family:"unit".into(),field:"countdown".into(),context:0x02000010}).is_err());
        for (family,field,pc,register,context) in [
            ("task","metadata",0x080d7062,0,0x08000000),
            ("script","cursor",0x081265d4,5,0x02000000),
            ("aux-script","cursor",0x0800a248,5,0x02000000),
        ] {
            m.cpu.regs[15]=pc;m.cpu.regs[register]=context;let before=m.save_state().unwrap();
            p.add(&m,family.into(),"test".into(),Target::Field{family:family.into(),field:field.into(),context}).unwrap();assert_eq!(before,m.save_state().unwrap());
            let pin=p.status()["pins"].as_array().unwrap().last().unwrap().clone();assert_eq!(pin["origin"]["family"],family);assert_eq!(pin["origin"]["pc"],pc);
        }
    }
}
