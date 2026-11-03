//! Readonly backing storage inspection and optional thread-bound bus observation.
//! No guest pointers/fields, protocol reads, timing changes or serialized state.
use crate::Machine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Byte {
    pub address: u32,
    pub region: &'static str,
    pub offset: usize,
    pub value: u8,
}
/// Backing bytes, not a simulated bus read. IO/backup protocols are deliberately refused.
pub fn peek(machine: &Machine, address: u32, length: u32) -> Result<Vec<Byte>, String> {
    if length == 0 || length > 256 || address.checked_add(length - 1).is_none() {
        return Err("Inspect needs 1..256 bytes without address wrap".into());
    }
    let b = &machine.bus;
    (address..=address+(length-1)).map(|a| {
        let (region,offset,data)=match a>>24 {
            0 if a<0x4000 => ("bios-backing",a as usize,&b.bios),
            2 => ("ewram",(a&0x3ffff) as usize,&b.ewram),
            3 => ("iwram",(a&0x7fff) as usize,&b.iwram),
            5 => ("palette",(a&0x3ff) as usize,&b.palette),
            6 => {let o=(a&0x1ffff) as usize;("vram",if o>=0x18000{o-0x8000}else{o},&b.vram)},
            7 => ("oam",(a&0x3ff) as usize,&b.oam),
            8..=13 => ("rom-backing",(a&0x1ffffff) as usize,&b.rom),
            _=>return Err(format!("No readonly backing view for {a:08x}; IO/open bus/backup protocols are not read")),
        };
        Ok(Byte{address:a,region,offset,value:*data.get(offset).ok_or("Address outside backing storage")?})
    }).collect()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Watch {
    pub start: u32,
    pub end: u32,
    pub write: bool,
    pub width: Option<u8>,
    pub value: Option<u32>,
}
impl Watch {
    pub fn validate(&self) -> Result<(), String> {
        if self.start >= self.end
            || self.width.is_some_and(|w| !matches!(w, 1 | 2 | 4))
            || self
                .width
                .zip(self.value)
                .is_some_and(|(w, v)| w < 4 && v >= (1u32 << (8 * w)))
        {
            return Err(
                "Watch needs an exclusive nonempty range, width1/2/4 and a fitting value".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Access {
    pub address: u32,
    pub width: u8,
    pub write: bool,
    pub value: u32,
}

#[cfg(feature = "rom-data-read-audit")]
mod observer {
    use super::*;
    use std::{cell::RefCell, marker::PhantomData, rc::Rc};
    struct State {
        watch: Watch,
        hit: Option<Access>,
    }
    thread_local! { static STATE:RefCell<Option<State>>=const{RefCell::new(None)}; }
    pub struct Capture(PhantomData<Rc<()>>);
    impl Capture {
        pub fn start(watch: Watch) -> Result<Self, String> {
            watch.validate()?;
            STATE.with(|c| {
                let mut s = c.borrow_mut();
                if s.is_some() {
                    return Err("Bus watch already active on this thread".into());
                }
                *s = Some(State { watch, hit: None });
                Ok(Self(PhantomData))
            })
        }
        pub fn hit(&self) -> Option<Access> {
            STATE.with(|s| s.borrow().as_ref().and_then(|s| s.hit.clone()))
        }
    }
    impl Drop for Capture {
        fn drop(&mut self) {
            STATE.with(|s| *s.borrow_mut() = None);
        }
    }
    pub(crate) fn record(address: u32, width: u8, write: bool, value: u32) {
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            let Some(s) = state.as_mut() else {
                return;
            };
            if s.hit.is_some() || crate::rom_data::read_audit::fetching() {
                return;
            }
            let w = &s.watch;
            if write == w.write
                && address < u32::MAX
                && u64::from(address) + u64::from(width) > u64::from(w.start)
                && address < w.end
                && w.width.is_none_or(|v| v == width)
                && w.value.is_none_or(|v| v == value)
            {
                s.hit = Some(Access {
                    address,
                    width,
                    write,
                    value,
                });
            }
        });
    }
}
#[cfg(feature = "rom-data-read-audit")]
pub(crate) use observer::record;
#[cfg(feature = "rom-data-read-audit")]
pub use observer::Capture;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bus;
    #[test]
    fn inspection_preserves_all_state_and_refuses_protocols() {
        let mut m = Machine::new(vec![0; 0x100]);
        m.bus.ewram[3] = 91;
        m.bus.io[0] = 17;
        let before = m.save_state().unwrap();
        assert_eq!(peek(&m, 0x02040003, 1).unwrap()[0].value, 91);
        assert_eq!(peek(&m, 0x0a000000, 1).unwrap()[0].offset, 0);
        for (a, n) in [
            (0x04000130, 2),
            (0x0e000000, 1),
            (0x10000000, 1),
            (u32::MAX, 2),
            (0x02000000, 257),
            (0x02000000, 0),
            (0x080000ff, 2),
        ] {
            assert!(peek(&m, a, n).is_err());
        }
        assert_eq!(before, m.save_state().unwrap());
        // The explicit bus method remains separate from inspection.
        assert_eq!(m.bus.read8(0x02000003), 91);
    }
    #[test]
    fn mirrors_vram_fold_and_window_bounds_are_explicit() {
        let mut m = Machine::new(vec![0; 0x100]);
        m.bus.vram[0x10000] = 7;
        let bytes = peek(&m, 0x06018000, 2).unwrap();
        assert_eq!((bytes[0].offset, bytes[0].value), (0x10000, 7));
        assert_eq!(peek(&m, 0x03008000, 1).unwrap()[0].offset, 0);
        assert_eq!(peek(&m, 0x05000400, 1).unwrap()[0].offset, 0);
    }
    #[cfg(feature = "rom-data-read-audit")]
    #[test]
    fn observed_read_write_value_and_fetch_exclusion_preserve_execution() {
        // ARM LDR r0,[r1]; STR r0,[r2]; branch to itself.
        let words = [0xe5910000u32, 0xe5820000, 0xeafffffe];
        let rom = words.iter().flat_map(|v| v.to_le_bytes()).collect();
        let mut m = Machine::new(rom);
        m.cpu.regs[15] = 0x08000000;
        m.cpu.regs[1] = 0x02000000;
        m.cpu.regs[2] = 0x02000004;
        m.bus.ewram[..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        let seed = m.save_state().unwrap();
        let c = Capture::start(Watch {
            start: 0x08000000,
            end: 0x08000004,
            write: false,
            width: None,
            value: None,
        })
        .unwrap();
        m.step();
        assert!(c.hit().is_none());
        drop(c);
        m.load_state(&seed).unwrap();
        let c = Capture::start(Watch {
            start: 0x02000000,
            end: 0x02000004,
            write: false,
            width: Some(4),
            value: Some(0x12345678),
        })
        .unwrap();
        m.step();
        assert_eq!(
            c.hit().unwrap(),
            Access {
                address: 0x02000000,
                width: 4,
                write: false,
                value: 0x12345678
            }
        );
        drop(c);
        let observed = m.save_state().unwrap();
        m.load_state(&seed).unwrap();
        m.step();
        assert_eq!(observed, m.save_state().unwrap());
        let c = Capture::start(Watch {
            start: 0x02000004,
            end: 0x02000008,
            write: true,
            width: Some(4),
            value: Some(0x12345678),
        })
        .unwrap();
        m.step();
        assert!(c.hit().unwrap().write);
        assert_eq!(&m.bus.ewram[4..8], &0x12345678u32.to_le_bytes());
        drop(c);
    }
}
