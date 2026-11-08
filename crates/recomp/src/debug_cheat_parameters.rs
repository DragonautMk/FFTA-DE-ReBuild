//! Explicit, locally declared parameter domains for source-code placeholders.
//! A declaration is not proof of a device format or a game's valid values.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Choice {
    pub name: String,
    pub value: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Specification {
    pub name: String,
    pub allowed: Domain,
    pub slots: Vec<Slot>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Domain {
    List { values: Vec<u16> },
    Range { min: u16, max: u16, step: u16 },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Slot {
    pub line: u16,
    pub field: Field,
    pub offset: u8,
    pub digits: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Field { Word, Value }

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 48 && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
impl Choice {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !valid_name(&self.name) { return Err("Parameter name must contain 1..48 ASCII letters, digits, '-' or '_'".into()); }
        Ok(())
    }
}

impl Specification {
    /// Check the declared shape without choosing a value for execution.
    pub(crate) fn validate_source(&self, lines: &[String]) -> Result<(), String> {
        let value = match &self.allowed {
            Domain::List { values } => *values.first().ok_or("Empty parameter value list")?,
            Domain::Range { min, .. } => *min,
        };
        expand(lines, self, &Choice { name: self.name.clone(), value }).map(|_| ())
    }

    fn accepts(&self, choice: &Choice) -> Result<(), String> {
        if !valid_name(&self.name) || choice.name != self.name || self.slots.is_empty() || self.slots.len() > 16 {
            return Err("Invalid parameter identity or slots".into());
        }
        let allowed = match &self.allowed {
            Domain::List { values } => {
                if values.is_empty() || values.len() > 256 || values.iter().copied().collect::<BTreeSet<_>>().len() != values.len() {
                    return Err("Invalid parameter value list".into());
                }
                values.contains(&choice.value)
            }
            Domain::Range { min, max, step } => {
                if min > max || *step == 0 { return Err("Invalid parameter range".into()); }
                choice.value >= *min && choice.value <= *max && (choice.value - min) % step == 0
            }
        };
        if !allowed { return Err("Parameter value is outside its declared domain".into()); }
        Ok(())
    }
}

/// Substitute only explicitly declared placeholder nibbles. All remaining
/// input must be complete hexadecimal code before the normal decoder runs.
pub(crate) fn expand(lines: &[String], spec: &Specification, choice: &Choice) -> Result<Vec<String>, String> {
    spec.accepts(choice)?;
    if lines.is_empty() || lines.len() > 1024 { return Err("A block needs 1..1024 lines".into()); }
    let mut fields = Vec::with_capacity(lines.len());
    for line in lines {
        let parts: Vec<_> = line.split_ascii_whitespace().collect();
        if parts.len() != 2 || parts[0].len() != 8 || parts[1].len() != 4 || !line.is_ascii() {
            return Err("Invalid parameterized CodeBreaker line".into());
        }
        fields.push([parts[0].as_bytes().to_vec(), parts[1].as_bytes().to_vec()]);
    }
    let mut occupied = BTreeSet::new();
    for slot in &spec.slots {
        let index = usize::from(slot.line);
        let field = match slot.field { Field::Word => 0, Field::Value => 1 };
        let offset = usize::from(slot.offset);
        let digits = usize::from(slot.digits);
        if digits == 0 || digits > 4 || index >= fields.len() || offset + digits > fields[index][field].len()
            || (digits < 4 && u32::from(choice.value) >= (1u32 << (digits * 4))) {
            return Err("Parameter slot or value width is invalid".into());
        }
        let replacement = format!("{:0digits$X}", choice.value);
        for (i, byte) in replacement.bytes().enumerate() {
            let position = offset + i;
            if !occupied.insert((index, field, position)) || !matches!(fields[index][field][position], b'x' | b'X') {
                return Err("Parameter slots overlap or do not match placeholders".into());
            }
            fields[index][field][position] = byte;
        }
    }
    let mut expanded = Vec::with_capacity(fields.len());
    for [word, value] in fields {
        if !word.iter().chain(value.iter()).all(u8::is_ascii_hexdigit) {
            return Err("Code contains undeclared or invalid placeholders".into());
        }
        expanded.push(format!("{} {}", String::from_utf8(word).unwrap(), String::from_utf8(value).unwrap()));
    }
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> Specification {
        Specification { name: "amount".into(), allowed: Domain::Range { min: 2, max: 10, step: 2 },
            slots: vec![Slot { line: 0, field: Field::Value, offset: 2, digits: 2 }] }
    }

    #[test]
    fn expands_only_declared_allowed_values_and_preserves_input() {
        let source = vec!["82000000 00XX".to_string(), "32000002 0001".to_string()];
        let expanded = expand(&source, &spec(), &Choice { name: "amount".into(), value: 10 }).unwrap();
        assert_eq!(expanded, ["82000000 000A", "32000002 0001"]);
        assert_eq!(source[0], "82000000 00XX");
        for value in [0, 3, 12] { assert!(expand(&source, &spec(), &Choice { name: "amount".into(), value }).is_err()); }
        assert!(expand(&source, &spec(), &Choice { name: "other".into(), value: 2 }).is_err());
    }

    #[test]
    fn refuses_uncovered_overlapping_or_oversized_substitutions() {
        let source = vec!["820000XX 00XX".to_string()];
        let choice = Choice { name: "amount".into(), value: 2 };
        assert!(expand(&source, &spec(), &choice).is_err());
        let mut both = spec(); both.slots.push(Slot { line: 0, field: Field::Word, offset: 6, digits: 2 });
        assert_eq!(expand(&source, &both, &choice).unwrap()[0], "82000002 0002");
        both.slots.push(both.slots[0].clone()); assert!(expand(&source, &both, &choice).is_err());
        let mut list = spec(); list.allowed = Domain::List { values: vec![2, 256] };
        assert!(expand(&["82000000 00XX".into()], &list, &Choice { name: "amount".into(), value: 256 }).is_err());
        list.allowed = Domain::List { values: vec![2, 2] }; assert!(expand(&source, &list, &choice).is_err());
    }
}
