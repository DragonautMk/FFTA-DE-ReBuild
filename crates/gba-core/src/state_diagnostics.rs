//! Read-only named fields in the CURRENT snapshot schema. No byte-offset guesses.
//! Values use postcard exactly like save states (including floating-point bits).
use serde::{
    ser::{Impossible, SerializeStruct},
    Serialize, Serializer,
};
use std::collections::BTreeMap;
type Fields = BTreeMap<String, Vec<u8>>;
#[derive(Debug)]
struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}
fn unsupported() -> Error {
    Error("Diagnostic root must serialize as a named struct".into())
}
struct Collector {
    prefix: &'static str,
    fields: Fields,
    bytes: usize,
}
macro_rules! reject_scalar {
    ($($name:ident($ty:ty)),* $(,)?) => {$(fn $name(self,_value:$ty)->Result<Fields,Error>{Err(unsupported())})*};
}
impl Serializer for Collector {
    type Ok = Fields;
    type Error = Error;
    type SerializeSeq = Impossible<Fields, Error>;
    type SerializeTuple = Impossible<Fields, Error>;
    type SerializeTupleStruct = Impossible<Fields, Error>;
    type SerializeTupleVariant = Impossible<Fields, Error>;
    type SerializeMap = Impossible<Fields, Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<Fields, Error>;
    reject_scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8])
    );
    fn serialize_none(self) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_unit(self) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Fields, Error> {
        Err(unsupported())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(unsupported())
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(unsupported())
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(unsupported())
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(unsupported())
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(unsupported())
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(unsupported())
    }
}
impl SerializeStruct for Collector {
    type Ok = Fields;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        let bytes = postcard::to_allocvec(value).map_err(|e| Error(e.to_string()))?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| Error("Diagnostic field size overflow".into()))?;
        if self.bytes > crate::save_state::MAX_STATE_BYTES || self.fields.len() >= 512 {
            return Err(Error("Diagnostic field bounds exceeded".into()));
        }
        if self
            .fields
            .insert(format!("{}.{}", self.prefix, key), bytes)
            .is_some()
        {
            return Err(Error("Duplicate diagnostic field".into()));
        }
        Ok(())
    }
    fn end(self) -> Result<Fields, Error> {
        Ok(self.fields)
    }
}
/// Exact serialized field values, without ROM/BIOS/host pointers or mutation.
/// CPU/Bus Serialize determines names and skipped fields; never parse varint offsets.
pub fn fields(machine: &crate::Machine) -> Result<Fields, String> {
    let mut cpu = machine
        .cpu
        .serialize(Collector {
            prefix: "cpu",
            fields: Fields::new(),
            bytes: 0,
        })
        .map_err(|e| e.to_string())?;
    let bus = machine
        .bus
        .serialize(Collector {
            prefix: "bus",
            fields: Fields::new(),
            bytes: 0,
        })
        .map_err(|e| e.to_string())?;
    if cpu
        .values()
        .chain(bus.values())
        .map(Vec::len)
        .sum::<usize>()
        > crate::save_state::MAX_STATE_BYTES
    {
        return Err("Diagnostic state exceeds snapshot bound".into());
    }
    cpu.extend(bus);
    Ok(cpu)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_follow_snapshot_serde_and_observation_preserves_full_state() {
        let mut m = crate::Machine::new(vec![0; 512]);
        let before = m.save_state().unwrap();
        let a = fields(&m).unwrap();
        assert_eq!(m.save_state().unwrap(), before);
        assert!(!a.contains_key("bus.rom"));
        assert!(!a.contains_key("bus.rom_resources"));
        assert!(a.contains_key("cpu.regs"));
        assert!(a.contains_key("bus.clock"));
        m.cpu.regs[0] = 7;
        let b = fields(&m).unwrap();
        assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
        assert_eq!(
            a.iter()
                .filter(|(k, v)| b[*k] != **v)
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>(),
            vec!["cpu.regs"]
        );
    }
    #[test]
    fn private_members_and_float_payload_bits_do_not_collapse_to_json_null() {
        let mut m = crate::Machine::new(vec![0; 512]);
        m.bus.hle_tap = vec![f32::from_bits(0x7fc00001)];
        let a = fields(&m).unwrap();
        m.bus.hle_tap[0] = f32::from_bits(0x7fc00002);
        let b = fields(&m).unwrap();
        assert_ne!(a["bus.hle_tap"], b["bus.hle_tap"]);
        m.bus.clock = 17;
        let c = fields(&m).unwrap();
        assert_ne!(b["bus.clock"], c["bus.clock"]);
    }
}
