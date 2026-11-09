//! Host-only selection state for a source-bound cheat. The guest is never
//! accessed here; actions are requests to the existing shared handlers.
use super::MenuInput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub index: u16,
    pub name: String,
    pub master: String,
    pub assessment: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter { pub name: String, pub domain: Domain }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Domain { List(Vec<u16>), Range { min:u16, max:u16, step:u16 }, DependentList { on:String, cases:Vec<(u16,Vec<u16>)> } }
impl Domain {
    fn dependent_values<'a>(on:&str,cases:&'a [(u16,Vec<u16>)],parameters:&[Parameter],selected:&[u16])->Option<&'a [u16]>{
        let parent=parameters.iter().position(|p|p.name==on)?;let selected=*selected.get(parent)?;
        cases.iter().find(|(when,_)|*when==selected).map(|(_,values)|values.as_slice())
    }
    fn count(&self,parameters:&[Parameter],selected:&[u16])->u32 { match self { Self::List(values)=>values.len() as u32, Self::Range{min,max,step} if *step>0 && min<=max=>u32::from((max-min)/step)+1, Self::DependentList{on,cases}=>Self::dependent_values(on,cases,parameters,selected).map_or(0,|v|v.len() as u32), _=>0 } }
    fn value(&self,index:u32,parameters:&[Parameter],selected:&[u16])->Option<u16> { match self { Self::List(values)=>values.get(index as usize).copied(), Self::Range{min,step,..} if index<self.count(parameters,selected)=>u16::try_from(u32::from(*min)+index*u32::from(*step)).ok(), Self::DependentList{on,cases}=>Self::dependent_values(on,cases,parameters,selected)?.get(index as usize).copied(), _=>None } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request { Prepare, ApplyOnce, Arm, Disarm, DisarmAll, PresetSave, PresetPrepare, PresetArm }

#[derive(Default)]
pub(super) struct State {
    pub id: String,
    pub title: String,
    pub sources: Vec<Source>,
    pub cursor: usize,
    pub selected: Option<usize>,
    pub focus: usize,
    pub feedback: String,
    pub preview: String,
    pub preview_token: String,
    pub presets: Vec<String>,
    pub preset_cursor: usize,
    pub preset_selected: Option<String>,
    pub preset_name: String,
    pub preset_mode: bool,
    pub parameter_mode: bool,
    pub parameter_cursor: u32,
    pub parameter_selected: Vec<u16>,
}

impl State {
    pub fn new(id: String, title: String, armed: bool, sources: Vec<Source>, presets: Vec<String>) -> Self {
        Self { id, title, sources, presets, focus: if armed { 4 } else { 0 }, ..Self::default() }
    }
    pub fn selected_source(&self) -> Option<&Source> { self.selected.and_then(|index| self.sources.get(index)) }
    pub fn selected_parameters(&self)->Option<Vec<(&str,u16)>>{let source=self.selected_source()?;if source.parameters.len()!=self.parameter_selected.len(){return None;}Some(source.parameters.iter().zip(&self.parameter_selected).map(|(p,v)|(p.name.as_str(),*v)).collect())}
    pub fn current_parameter(&self)->Option<(&str,u16)>{let source=self.selected_source()?;let p=source.parameters.get(self.parameter_selected.len())?;Some((&p.name,p.domain.value(self.parameter_cursor,&source.parameters,&self.parameter_selected)?))}
    pub fn handle(&mut self, input: MenuInput) -> Option<Request> {
        if self.parameter_mode {
            self.focus=0;
            let count=self.selected_source().and_then(|s|s.parameters.get(self.parameter_selected.len()).map(|p|p.domain.count(&s.parameters,&self.parameter_selected))).unwrap_or(0);
            match input {
                MenuInput::Up => self.parameter_cursor=self.parameter_cursor.saturating_sub(1),
                MenuInput::Down => self.parameter_cursor=(self.parameter_cursor+1).min(count.saturating_sub(1)),
                MenuInput::Confirm if count>0 => {let value=self.current_parameter().unwrap().1;self.parameter_selected.push(value);self.parameter_cursor=0;if self.selected_source().is_some_and(|s|s.parameters.len()==self.parameter_selected.len()){self.parameter_mode=false;self.focus=1;}},
                MenuInput::Left => {self.parameter_mode=false;self.selected=None;self.parameter_selected.clear();},
                _=>{}
            }
            self.preview.clear();self.preview_token.clear();self.feedback.clear();return None;
        }
        if self.focus == 0 {
            if self.preset_mode {
                match input {
                    MenuInput::Up => self.preset_cursor = self.preset_cursor.saturating_sub(1),
                    MenuInput::Down => self.preset_cursor = (self.preset_cursor + 1).min(self.presets.len().saturating_sub(1)),
                    MenuInput::Right => self.focus = 1,
                    MenuInput::Confirm => {
                        self.preset_selected = self.presets.get(self.preset_cursor).cloned();
                        self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                    }
                    _ => {}
                }
                return None;
            }
            match input {
                MenuInput::Up => self.cursor = self.cursor.saturating_sub(1),
                MenuInput::Down => self.cursor = (self.cursor + 1).min(self.sources.len().saturating_sub(1)),
                MenuInput::Right => self.focus = 1,
                MenuInput::Confirm if !self.sources.is_empty() => {
                    self.selected = Some(self.cursor); self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                    self.parameter_selected.clear();self.parameter_cursor=0;
                    self.parameter_mode=self.selected_source().is_some_and(|s|!s.parameters.is_empty());
                    self.focus = if self.parameter_mode {0}else{1};
                }
                _ => {}
            }
        } else {
            match input {
                MenuInput::Left => self.focus = 0,
                MenuInput::Up => self.focus = self.focus.saturating_sub(1).max(1),
                MenuInput::Down => self.focus = (self.focus + 1).min(6),
                MenuInput::Confirm if self.focus == 6 => {
                    self.preset_mode = !self.preset_mode; self.focus = 0;
                    self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                }
                MenuInput::Confirm if self.preset_mode => return match self.focus {
                    1 => Some(Request::PresetSave), 2 => Some(Request::PresetPrepare),
                    3 => Some(Request::PresetArm), 4 => Some(Request::DisarmAll), _ => None,
                },
                MenuInput::Confirm => return Some(match self.focus {
                    1 => Request::Prepare, 2 => Request::ApplyOnce, 3 => Request::Arm,
                    4 => Request::Disarm, _ => Request::DisarmAll,
                }),
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_explicit_and_changing_source_clears_preview() {
        let mut state = State::new("CH-0001".into(), "Example".into(), false, vec![Source { index: 2, name: "one".into(), master: "CH-0002".into(), assessment: "candidate".into(),parameters:vec![] }, Source { index: 5, name: "two".into(), master: "CH-0003".into(), assessment: "unresolved".into(),parameters:vec![] }], vec![]);
        assert!(state.selected_source().is_none());
        state.handle(MenuInput::Right); assert_eq!(state.handle(MenuInput::Confirm), Some(Request::Prepare));
        assert!(state.selected_source().is_none());
        state.handle(MenuInput::Left); state.handle(MenuInput::Down); state.handle(MenuInput::Confirm);
        assert_eq!(state.selected_source().unwrap().index, 5);
        state.preview = "old preview".into(); state.handle(MenuInput::Left); state.handle(MenuInput::Up); state.handle(MenuInput::Confirm);
        assert_eq!(state.selected_source().unwrap().index, 2); assert!(state.preview.is_empty());
    }
    #[test]
    fn parameter_requires_explicit_selection_and_source_change_clears_review(){
        let source=Source{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![Parameter{name:"amount".into(),domain:Domain::Range{min:2,max:8,step:2}}]};
        let mut state=State::new("CH-0001".into(),"Example".into(),false,vec![source],vec![]);
        state.handle(MenuInput::Confirm);assert!(state.parameter_mode);assert!(state.selected_parameters().is_none());
        state.handle(MenuInput::Down);assert_eq!(state.current_parameter(),Some(("amount",4)));state.handle(MenuInput::Confirm);assert_eq!(state.selected_parameters(),Some(vec![("amount",4)]));
        state.preview="old".into();state.preview_token="token".into();state.handle(MenuInput::Left);state.handle(MenuInput::Confirm);assert!(state.parameter_mode);assert!(state.selected_parameters().is_none());assert!(state.preview.is_empty());assert!(state.preview_token.is_empty());
    }
    #[test]
    fn multiple_parameters_are_confirmed_in_order_before_prepare(){
        let parameter=|name:&str|Parameter{name:name.into(),domain:Domain::List(vec![1,2])};
        let source=Source{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![parameter("race"),parameter("job"),parameter("look")]};
        let mut state=State::new("CH-0001".into(),"Example".into(),false,vec![source],vec![]);
        state.handle(MenuInput::Confirm);assert_eq!(state.current_parameter(),Some(("race",1)));state.handle(MenuInput::Down);state.handle(MenuInput::Confirm);
        assert_eq!(state.current_parameter(),Some(("job",1)));assert!(state.selected_parameters().is_none());state.handle(MenuInput::Confirm);
        assert_eq!(state.current_parameter(),Some(("look",1)));state.handle(MenuInput::Down);state.handle(MenuInput::Confirm);
        assert_eq!(state.selected_parameters(),Some(vec![("race",2),("job",1),("look",2)]));assert_eq!(state.handle(MenuInput::Confirm),Some(Request::Prepare));
        state.preview="old".into();state.preview_token="old".into();state.handle(MenuInput::Left);state.handle(MenuInput::Confirm);
        assert!(state.selected_parameters().is_none());assert_eq!(state.current_parameter(),Some(("race",1)));assert!(state.preview.is_empty());assert!(state.preview_token.is_empty());
    }
    #[test]
    fn dependent_parameter_choices_follow_the_explicit_previous_value(){
        let source=Source{index:0,name:"synthetic".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![
            Parameter{name:"race".into(),domain:Domain::List(vec![1,2])},
            Parameter{name:"job".into(),domain:Domain::DependentList{on:"race".into(),cases:vec![(1,vec![10,11]),(2,vec![20])]}},
        ]};
        let mut state=State::new("CH-0001".into(),"Example".into(),false,vec![source],vec![]);
        state.handle(MenuInput::Confirm);state.handle(MenuInput::Down);state.handle(MenuInput::Confirm);
        assert_eq!(state.current_parameter(),Some(("job",20)));state.handle(MenuInput::Down);assert_eq!(state.current_parameter(),Some(("job",20)));
        state.handle(MenuInput::Confirm);assert_eq!(state.selected_parameters(),Some(vec![("race",2),("job",20)]));
        state.preview="old".into();state.preview_token="old".into();state.handle(MenuInput::Left);state.handle(MenuInput::Confirm);state.handle(MenuInput::Confirm);
        assert_eq!(state.current_parameter(),Some(("job",10)));state.handle(MenuInput::Down);assert_eq!(state.current_parameter(),Some(("job",11)));
        assert!(state.preview.is_empty());assert!(state.preview_token.is_empty());
    }
}
