//! Edge-triggered controller keyboard; only the frontend applies its actions.
const COLS:usize=7;
const KEYS:usize=49;
const PAGES:[&str;4]=[
    "abcdefghijklmnopqrstuvwxyz0123456789",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
    "àèéìòùäëïöüáíóúâêîôûãõñçßøåæœ-_./:@ ",
    ",;=+*!?()[]{}<>%&|\"'`~$^#\\-_/.: ",
];
const ACTIONS:[&str;13]=["Space","Back","Clear","Aa/#","Previous","Next","Cursor -","Cursor +","Complete","Send","Close","status","help"];

#[derive(Clone,Copy,Default)]
pub(crate) struct Input {pub up:bool,pub down:bool,pub left:bool,pub right:bool,pub confirm:bool,pub cancel:bool}
impl Input {fn any(self)->bool{self.up||self.down||self.left||self.right||self.confirm||self.cancel}}
#[derive(Debug,PartialEq,Eq)]
pub(crate) enum Action {Changed,Insert(String),Backspace,Clear,Previous,Next,Left,Right,Complete,Submit,Close,Cancel}
#[derive(Default)]
pub(crate) struct Controller {page:usize,selected:usize,visible:bool,armed:bool,previous:Input}
impl Controller {
    pub fn reset(&mut self){self.armed=false;self.previous=Input::default();self.visible=false;}
    pub fn visible(&self)->bool{self.visible}
    pub fn selected(&self)->usize{self.selected}
    pub fn labels(&self)->Vec<String>{
        let mut keys=PAGES[self.page].chars().take(36).map(|c|c.to_string()).collect::<Vec<_>>();
        while keys.len()<36{keys.push(" ".into());}
        keys.extend(ACTIONS.iter().map(|s|s.to_string()));keys
    }
    pub fn localized_labels(&self,language:input_config::Language)->Vec<String>{
        let mut labels=self.labels();
        for label in &mut labels[36..47] { *label=language.text(label).to_owned(); }
        labels
    }
    /// A neutral sample arms input after focus arrives, so a held button cannot
    /// submit or close the panel. At most one action is produced per poll.
    pub fn poll(&mut self,input:Input)->Option<Action>{
        if !self.armed {if !input.any(){self.armed=true;}self.previous=input;return None;}
        let prev=self.previous;self.previous=input;
        let up=input.up&&!prev.up;let down=input.down&&!prev.down;
        let left=input.left&&!prev.left;let right=input.right&&!prev.right;
        let confirm=input.confirm&&!prev.confirm;let cancel=input.cancel&&!prev.cancel;
        if !(up||down||left||right||confirm||cancel){return None;}
        self.visible=true;
        if cancel{return Some(Action::Cancel);}
        if up{self.selected=(self.selected+KEYS-COLS)%KEYS;}
        else if down{self.selected=(self.selected+COLS)%KEYS;}
        else if left{let row=self.selected/COLS;self.selected=row*COLS+(self.selected%COLS+COLS-1)%COLS;}
        else if right{let row=self.selected/COLS;self.selected=row*COLS+(self.selected%COLS+1)%COLS;}
        else if confirm {
            return match self.selected {
                0..=35=>Some(Action::Insert(self.labels()[self.selected].clone())),
                36=>Some(Action::Insert(" ".into())),37=>Some(Action::Backspace),38=>Some(Action::Clear),
                39=>{self.page=(self.page+1)%PAGES.len();Some(Action::Changed)},40=>Some(Action::Previous),41=>Some(Action::Next),
                42=>Some(Action::Left),43=>Some(Action::Right),44=>Some(Action::Complete),45=>Some(Action::Submit),
                46=>Some(Action::Close),47=>Some(Action::Insert("status".into())),48=>Some(Action::Insert("help".into())),_=>unreachable!(),
            };
        }
        Some(Action::Changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn press(c:&mut Controller,input:Input)->Option<Action>{let action=c.poll(input);c.poll(Input::default());action}
    #[test]
    fn held_input_needs_neutral_then_edges_and_wraps(){
        let mut c=Controller::default();let a=Input{confirm:true,..Default::default()};
        assert_eq!(c.poll(a),None);assert!(!c.visible());c.poll(Input::default());
        assert_eq!(c.poll(a),Some(Action::Insert("a".into())));assert_eq!(c.poll(a),None);
        c.poll(Input::default());press(&mut c,Input{left:true,..Default::default()});assert_eq!(c.selected(),6);
        press(&mut c,Input{up:true,..Default::default()});assert_eq!(c.selected(),48);
        assert_eq!(press(&mut c,a),Some(Action::Insert("help".into())));
        c.reset();assert_eq!(c.poll(a),None);assert!(!c.visible());
    }
    #[test]
    fn supports_punctuation_history_presets_and_explicit_submission(){
        let mut c=Controller::default();c.poll(Input::default());c.selected=39;
        assert_eq!(press(&mut c,Input{confirm:true,..Default::default()}),Some(Action::Changed));assert_eq!(c.page,1);
        press(&mut c,Input{confirm:true,..Default::default()});press(&mut c,Input{confirm:true,..Default::default()});assert_eq!(c.page,3);
        assert!(c.labels().contains(&"-".into()));assert!(c.labels().contains(&"\"".into()));
        c.selected=40;assert_eq!(press(&mut c,Input{confirm:true,..Default::default()}),Some(Action::Previous));
        c.selected=45;assert_eq!(press(&mut c,Input{confirm:true,..Default::default()}),Some(Action::Submit));
        assert_eq!(press(&mut c,Input{cancel:true,..Default::default()}),Some(Action::Cancel));
    }
    #[test]
    fn localized_action_labels_do_not_change_inserted_commands(){
        let mut c=Controller::default();let en=c.localized_labels(input_config::Language::default());
        let it=c.localized_labels(input_config::Language::parse("it"));
        assert_eq!(en[40],"Previous");assert_eq!(it[40],"Prec.");
        assert_eq!(&en[..36],&it[..36]);assert_eq!(&en[47..],&it[47..]);
        c.poll(Input::default());c.selected=47;
        assert_eq!(press(&mut c,Input{confirm:true,..Default::default()}),Some(Action::Insert("status".into())));
    }
}
