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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<ValueLabel>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValueLabel { pub value: u16, pub label: String }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Domain {
    List { values: Vec<u16> },
    Range { min: u16, max: u16, step: u16 },
    DependentList { on: String, cases: Vec<DependentCase> },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DependentCase { pub when: u16, pub values: Vec<u16> }

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
    #[cfg(test)]
    pub(crate) fn validate_source(&self, lines: &[String]) -> Result<(), String> {
        let value = match &self.allowed {
            Domain::List { values } => *values.first().ok_or("Empty parameter value list")?,
            Domain::Range { min, .. } => *min,
            Domain::DependentList { .. } => return Err("Dependent parameter needs its earlier choice".into()),
        };
        expand(lines, self, &Choice { name: self.name.clone(), value }).map(|_| ())
    }

    fn accepts(&self, choice: &Choice, prior: &[Specification], choices: &[Choice]) -> Result<(), String> {
        if !valid_name(&self.name) || choice.name != self.name || self.slots.is_empty() || self.slots.len() > 16 {
            return Err("Invalid parameter identity or slots".into());
        }
        let mut labeled_values = BTreeSet::new();
        if self.labels.len() > 256 || self.labels.iter().any(|item| {
            let in_domain = match &self.allowed {
                Domain::List { values } => values.contains(&item.value),
                Domain::Range { min, max, step } => *step > 0 && item.value >= *min && item.value <= *max && (item.value - min) % step == 0,
                Domain::DependentList { cases, .. } => cases.iter().any(|case| case.values.contains(&item.value)),
            };
            !in_domain || !labeled_values.insert(item.value) || item.label.trim().is_empty()
                || item.label.len() > 256 || item.label.chars().count() > 80 || item.label.chars().any(char::is_control)
        }) { return Err("Invalid parameter value labels".into()); }
        let max_value = match &self.allowed {
            Domain::List { values } => *values.iter().max().ok_or("Empty parameter value list")?,
            Domain::Range { min, max, step } => {
                if min > max || *step == 0 { return Err("Invalid parameter range".into()); }
                min + ((max - min) / step) * step
            }
            Domain::DependentList { on, cases } => {
                if !valid_name(on) || cases.is_empty() || cases.len()>256 || cases.iter().map(|c|c.values.len()).sum::<usize>()>4096 {
                    return Err("Invalid dependent parameter cases".into());
                }
                let parent=prior.iter().find(|s|s.name==*on).ok_or("Dependent parameter must reference an earlier parameter")?;
                let expected=parent.allowed.possible_values()?;
                let keys=cases.iter().map(|case|case.when).collect::<BTreeSet<_>>();
                if keys.len()!=cases.len() || keys!=expected {return Err("Dependent cases must cover each earlier value exactly once".into());}
                let mut maximum=0;
                for case in cases {
                    if case.values.is_empty() || case.values.len()>256 || case.values.iter().copied().collect::<BTreeSet<_>>().len()!=case.values.len() {return Err("Invalid dependent parameter values".into());}
                    maximum=maximum.max(*case.values.iter().max().unwrap());
                }
                maximum
            }
        };
        for slot in &self.slots {
            let digits = usize::from(slot.digits);
            if digits == 0 || digits > 4 || (digits < 4 && u32::from(max_value) >= (1u32 << (digits * 4))) {
                return Err("Declared parameter domain exceeds slot width".into());
            }
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
            Domain::DependentList { on, cases } => {
                let parent=choices.iter().find(|c|c.name==*on).ok_or("Missing earlier parameter choice")?;
                cases.iter().find(|case|case.when==parent.value).is_some_and(|case|case.values.contains(&choice.value))
            }
        };
        if !allowed { return Err("Parameter value is outside its declared domain".into()); }
        Ok(())
    }
}

impl Domain {
    fn possible_values(&self)->Result<BTreeSet<u16>,String>{
        let values:BTreeSet<u16>=match self {
            Self::List{values}=>values.iter().copied().collect(),
            Self::Range{min,max,step} if *step>0 && min<=max=>{
                let count=(u32::from(*max)-u32::from(*min))/u32::from(*step)+1;
                if count>256{return Err("Dependent parent has more than 256 values".into());}
                (0..count).map(|i|(u32::from(*min)+i*u32::from(*step)) as u16).collect()
            }
            Self::DependentList{cases,..}=>cases.iter().flat_map(|case|case.values.iter().copied()).collect(),
            _=>return Err("Invalid dependent parent domain".into()),
        };
        if values.is_empty() || values.len()>256{return Err("Dependent parent needs 1..256 values".into());}
        Ok(values)
    }
    fn first_for(&self,choices:&[Choice])->Result<u16,String>{
        match self {
            Self::List{values}=>values.first().copied(),
            Self::Range{min,..}=>Some(*min),
            Self::DependentList{on,cases}=>choices.iter().find(|c|c.name==*on).and_then(|choice|cases.iter().find(|case|case.when==choice.value)).and_then(|case|case.values.first()).copied(),
        }.ok_or_else(||"No value available for declared parameter".into())
    }
}

/// Substitute only explicitly declared placeholder nibbles. All remaining
/// input must be complete hexadecimal code before the normal decoder runs.
#[cfg(test)]
pub(crate) fn expand(lines: &[String], spec: &Specification, choice: &Choice) -> Result<Vec<String>, String> {
    expand_many(lines,std::slice::from_ref(spec),std::slice::from_ref(choice))
}

/// All declared choices are required, matched by unique name and applied as
/// one transaction. Slots may cover X, Y or Z placeholders but never overlap.
pub(crate) fn expand_many(lines: &[String], specs: &[Specification], choices: &[Choice]) -> Result<Vec<String>, String> {
    if specs.is_empty() || specs.len()>8 || choices.len()!=specs.len(){return Err("Choose each declared parameter exactly once".into());}
    let mut names=BTreeSet::new();let mut choice_names=BTreeSet::new();
    for choice in choices {choice.validate()?;if !choice_names.insert(&choice.name){return Err("Duplicate parameter choice".into());}}
    for (index,spec) in specs.iter().enumerate() {
        if !names.insert(&spec.name){return Err("Duplicate parameter declaration".into());}
        let choice=choices.iter().find(|choice|choice.name==spec.name).ok_or("Missing declared parameter choice")?;
        spec.accepts(choice,&specs[..index],choices)?;
    }
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
    for spec in specs {let choice=choices.iter().find(|choice|choice.name==spec.name).unwrap();for slot in &spec.slots {
        let index = usize::from(slot.line);
        let field = match slot.field { Field::Word => 0, Field::Value => 1 };
        let offset = usize::from(slot.offset);
        let digits = usize::from(slot.digits);
        if digits == 0 || digits > 4 || index >= fields.len() || offset + digits > fields[index][field].len()
            || (digits < 4 && u32::from(choice.value) >= (1u32 << (digits * 4))) {
            return Err("Parameter slot or value width is invalid".into());
        }
        let replacement = format!("{:0digits$X}", choice.value);
        let placeholder=fields[index][field][offset].to_ascii_lowercase();
        if !matches!(placeholder,b'x'|b'y'|b'z'){return Err("Parameter slot does not match placeholders".into());}
        for (i, byte) in replacement.bytes().enumerate() {
            let position = offset + i;
            if !occupied.insert((index, field, position)) || fields[index][field][position].to_ascii_lowercase()!=placeholder {
                return Err("Parameter slots overlap or do not match placeholders".into());
            }
            fields[index][field][position] = byte;
        }
    }}
    let mut expanded = Vec::with_capacity(fields.len());
    for [word, value] in fields {
        if !word.iter().chain(value.iter()).all(u8::is_ascii_hexdigit) {
            return Err("Code contains undeclared or invalid placeholders".into());
        }
        expanded.push(format!("{} {}", String::from_utf8(word).unwrap(), String::from_utf8(value).unwrap()));
    }
    Ok(expanded)
}

pub(crate) fn validate_many_source(lines:&[String],specs:&[Specification])->Result<(),String>{
    let mut choices=Vec::new();for spec in specs{choices.push(Choice{name:spec.name.clone(),value:spec.allowed.first_for(&choices)?});}
    expand_many(lines,specs,&choices).map(|_|())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> Specification {
        Specification { name: "amount".into(), allowed: Domain::Range { min: 2, max: 10, step: 2 }, labels: vec![],
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
    fn optional_labels_are_bounded_and_do_not_change_selected_values() {
        let lines = vec!["82000000 00XX".to_string()];
        let mut parameter = spec();
        let choice = Choice { name: "amount".into(), value: 4 };
        let plain = expand(&lines, &parameter, &choice).unwrap();
        assert!(serde_json::to_value(&parameter).unwrap().get("labels").is_none());
        parameter.labels = vec![ValueLabel { value: 4, label: "Four".into() }];
        assert_eq!(expand(&lines, &parameter, &choice).unwrap(), plain);
        let parsed: Specification = serde_json::from_value(serde_json::to_value(&parameter).unwrap()).unwrap();
        assert_eq!(parsed, parameter);
        parameter.labels.push(ValueLabel { value: 4, label: "Duplicate".into() });
        assert!(expand(&lines, &parameter, &choice).is_err());
        parameter.labels = vec![ValueLabel { value: 3, label: "Outside stepped range".into() }];
        assert!(expand(&lines, &parameter, &choice).is_err());
        parameter.labels = vec![ValueLabel { value: 4, label: "Line\nbreak".into() }];
        assert!(expand(&lines, &parameter, &choice).is_err());
        parameter.labels = vec![ValueLabel { value: 4, label: "x".repeat(81) }];
        assert!(expand(&lines, &parameter, &choice).is_err());
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
        assert!(list.validate_source(&["82000000 00XX".into()]).is_err());
        let mut range = spec(); range.allowed = Domain::Range { min: 2, max: 256, step: 2 };
        assert!(range.validate_source(&["82000000 00XX".into()]).is_err());
        range.allowed = Domain::Range { min: 2, max: 256, step: 3 };
        assert!(range.validate_source(&["82000000 00XX".into()]).is_ok());
        list.allowed = Domain::List { values: vec![2, 2] }; assert!(expand(&source, &list, &choice).is_err());
    }
    #[test]
    fn multiple_parameters_expand_independently_and_refuse_ambiguous_inputs(){
        let source=vec!["820000XX YYZZ".to_string()];
        let mk=|name:&str,field,offset|Specification{name:name.into(),allowed:Domain::List{values:vec![1,2]},labels:vec![],slots:vec![Slot{line:0,field,offset,digits:2}]};
        let specs=vec![mk("race",Field::Word,6),mk("job",Field::Value,0),mk("look",Field::Value,2)];
        let choices=vec![Choice{name:"look".into(),value:2},Choice{name:"race".into(),value:1},Choice{name:"job".into(),value:2}];
        validate_many_source(&source,&specs).unwrap();assert_eq!(expand_many(&source,&specs,&choices).unwrap()[0],"82000001 0202");assert_eq!(source[0],"820000XX YYZZ");
        assert!(expand_many(&source,&specs,&choices[..2]).is_err());let mut duplicate=choices.clone();duplicate[0]=duplicate[1].clone();assert!(expand_many(&source,&specs,&duplicate).is_err());
        let mut overlapping=specs.clone();overlapping[2].slots[0]=overlapping[1].slots[0].clone();assert!(validate_many_source(&source,&overlapping).is_err());
        let mut bad=choices.clone();bad[0].value=3;assert!(expand_many(&source,&specs,&bad).is_err());
    }
    #[test]
    fn dependent_domains_require_complete_earlier_cases_and_refuse_cross_choices(){
        let source=vec!["820000XX 00YY".into()];
        let parent=Specification{name:"race".into(),allowed:Domain::List{values:vec![1,2]},labels:vec![],slots:vec![Slot{line:0,field:Field::Word,offset:6,digits:2}]};
        let child=Specification{name:"job".into(),allowed:Domain::DependentList{on:"race".into(),cases:vec![DependentCase{when:1,values:vec![10,11]},DependentCase{when:2,values:vec![20]}]},labels:vec![],slots:vec![Slot{line:0,field:Field::Value,offset:2,digits:2}]};
        let specs=vec![parent,child];validate_many_source(&source,&specs).unwrap();
        let choice=|name:&str,value|Choice{name:name.into(),value};
        assert_eq!(expand_many(&source,&specs,&[choice("job",20),choice("race",2)]).unwrap(),["82000002 0014"]);
        assert!(expand_many(&source,&specs,&[choice("race",1),choice("job",20)]).is_err());
        assert!(expand_many(&source,&specs,&[choice("race",2),choice("job",11)]).is_err());
        let mut missing=specs.clone();if let Domain::DependentList{cases,..}=&mut missing[1].allowed{cases.pop();}assert!(validate_many_source(&source,&missing).is_err());
        let mut duplicate=specs.clone();if let Domain::DependentList{cases,..}=&mut duplicate[1].allowed{cases[1].when=1;}assert!(validate_many_source(&source,&duplicate).is_err());
        let mut too_wide=specs.clone();if let Domain::DependentList{cases,..}=&mut too_wide[1].allowed{cases[1].values.push(256);}assert!(validate_many_source(&source,&too_wide).is_err());
        let mut later=specs.clone();later.swap(0,1);assert!(validate_many_source(&source,&later).is_err());
        let encoded=serde_json::to_value(&specs[1].allowed).unwrap();
        #[derive(Deserialize)] #[serde(tag="kind",rename_all="kebab-case")] enum OldDomain { List, Range }
        assert!(serde_json::from_value::<OldDomain>(encoded).is_err());
    }
}
