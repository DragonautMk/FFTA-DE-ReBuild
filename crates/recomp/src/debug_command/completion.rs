//! Pure, bounded command completion. Suggestions never dispatch an action.
use std::ops::Range;

const COMMANDS:&[&str]=&["help","status","pause","resume","step","inspect","break","watch","cancel","graphics","ui-start","ui-report","audio-start","audio-report","code-start","code-report","gameplay-start","gameplay-report","recent-start","recent-freeze","recent-report","issue-report","profile","catalog","cheats","content","replay","jobs","retry","recipe"];

#[derive(Debug)]
struct Token {range:Range<usize>,value:String,prefix:String}
#[derive(Debug)]
pub(crate) struct Completion {pub alternatives:Vec<String>,range:Range<usize>,at_end:bool}

struct CycleState {
    expected:String,cursor:usize,range:Range<usize>,at_end:bool,index:usize,
    alternatives:Vec<String>,profiles:Vec<String>,catalogs:Vec<String>,cheats:Vec<String>,requests:Vec<String>,operations:Vec<String>,
}
#[derive(Default)]
pub(crate) struct Cycle {state:Option<CycleState>}
pub(crate) enum Applied {None,One,Choice{index:usize,total:usize,value:String}}
fn encoded(value:&str,at_end:bool)->String{let mut out=quoted(value);if at_end{out.push(' ');}out}
impl Cycle {
    /// Repeated Tab cycles only the exact unchanged draft and candidate set.
    /// It changes an editable draft, never submits it or dispatches an action.
    pub fn apply(&mut self,editor:&mut super::editor::Editor,profiles:&[String],catalogs:&[String],cheats:&[String],requests:&[String],operations:&[String])->Result<Applied,&'static str>{
        if editor.selection().is_some()||!editor.preedit().is_empty(){self.state=None;return Err("Complete without an active selection or composition");}
        if let Some(state)=self.state.as_mut(){
            if state.expected==editor.text()&&state.cursor==editor.cursor()&&state.profiles==profiles&&state.catalogs==catalogs&&state.cheats==cheats&&state.requests==requests&&state.operations==operations {
                let index=(state.index+1)%state.alternatives.len();let value=encoded(&state.alternatives[index],state.at_end);
                editor.replace_range(state.range.clone(),&value)?;state.index=index;state.range.end=state.range.start+value.len();state.expected=editor.text().to_owned();state.cursor=editor.cursor();
                return Ok(Applied::Choice{index:index+1,total:state.alternatives.len(),value:state.alternatives[index].clone()});
            }
        }
        self.state=None;
        let result=complete(editor.text(),editor.cursor(),profiles,catalogs,cheats,requests,operations)?;
        if result.alternatives.is_empty(){return Ok(Applied::None);}
        let value=encoded(&result.alternatives[0],result.at_end);editor.replace_range(result.range.clone(),&value)?;
        if result.alternatives.len()==1{return Ok(Applied::One);}
        let total=result.alternatives.len();let selected=result.alternatives[0].clone();
        self.state=Some(CycleState{expected:editor.text().to_owned(),cursor:editor.cursor(),range:result.range.start..result.range.start+value.len(),at_end:result.at_end,index:0,
            alternatives:result.alternatives,profiles:profiles.to_vec(),catalogs:catalogs.to_vec(),cheats:cheats.to_vec(),requests:requests.to_vec(),operations:operations.to_vec()});
        Ok(Applied::Choice{index:1,total,value:selected})
    }
}

/// Uses the same quote/escape rules as the command parser, but permits a final
/// unclosed quote while editing. The cursor must be at a Unicode boundary.
fn lex(text:&str,cursor:usize)->Result<Vec<Token>,&'static str>{
    if text.len()>4096||cursor>text.len()||!text.is_char_boundary(cursor){return Err("Invalid draft boundary");}
    if text.chars().any(|c|c.is_control()&&c!='\t'){return Err("Correct multiline/control text before completing");}
    let mut tokens=Vec::new();let mut current:Option<Token>=None;let mut quote=None;let mut escaped=false;
    for (i,ch) in text.char_indices(){
        if quote.is_none() && ch.is_whitespace(){
            if let Some(mut token)=current.take(){token.range.end=i;tokens.push(token);}
            continue;
        }
        let token=current.get_or_insert_with(||Token{range:i..i,value:String::new(),prefix:String::new()});
        if escaped {
            if Some(ch)!=quote && ch!='\\'{return Err("Invalid quoted escape");}
            if i==cursor{return Err("Finish the quoted escape first");}
            token.value.push(ch);if i<cursor{token.prefix.push(ch);}escaped=false;
        } else if let Some(q)=quote {
            if ch==q{quote=None;}else if ch=='\\'{escaped=true;}else{token.value.push(ch);if i<cursor{token.prefix.push(ch);}}
        } else if ch=='\''||ch=='"'{quote=Some(ch);}else{token.value.push(ch);if i<cursor{token.prefix.push(ch);}}
    }
    if escaped{return Err("Finish the quoted escape first");}
    if let Some(mut token)=current{token.range.end=text.len();tokens.push(token);}
    if tokens.len()>64{return Err("Too many command tokens");}Ok(tokens)
}

fn quoted(value:&str)->String{
    if value.chars().any(|c|c.is_whitespace()||matches!(c,'\''|'"'|'\\')){
        let mut out=String::from("\"");for c in value.chars(){if matches!(c,'"'|'\\'){out.push('\\');}out.push(c);}out.push('"');out
    }else{value.to_owned()}
}
fn values(flag:&str,command:&str)->Vec<String>{
    let choices:&[&str]=match (command,flag){
        ("step","--keys")=>&["0x3ff"],
        ("watch","--width")=>&["1","2","4"],
        ("break"|"watch","--steps")=>&["100000"],
        ("recent-start","--reads"|"--on-error")=>&["on","off"],
        ("code-start","--mode")=>&["all","arm","thumb"],
        ("code-start","--kind")=>&["all","native","missing","system","hook","interpreter"],
        ("gameplay-start","--family")=>&["all","task","script","unit"],
        ("audio-start","--streams")=>&["mixed","psg","hle","fifo0","fifo1"],
        (_,"--start"|"--value")=>&["0x0"],
        (_,"--end"|"--context")=>&["0x02000000"],
        (_,"--frames")=>&["1"],(_,"--events")=>&["512"],
        _=>&[],
    };choices.iter().map(|s|(*s).to_owned()).collect()
}
fn flags(command:&str)->&'static [&'static str]{match command{
    "step"=>&["--keys"],"break"=>&["--steps"],"watch"=>&["--steps","--width","--value"],
    "graphics"=>&["--compare-last"],"audio-start"=>&["--frames","--streams"],
    "code-start"=>&["--frames","--events","--start","--end","--kind","--mode"],
    "gameplay-start"=>&["--frames","--events","--family","--context"],
    "recent-start"=>&["--events","--reads","--on-error","--start","--end"],
    "issue-report"=>&["--symptom"],"cancel"=>&["--request"],_=>&[],
}}
pub(crate) fn hint(text:&str,language:input_config::Language)->&'static str {
    let tokens=lex(text,text.len()).unwrap_or_default();
    let command=tokens.first().map_or("",|t|t.value.as_str());
    language.text(match command {
        "help"=>"help [operation-id] · list or details from the registry",
        "profile"|"catalog"=>"profile|catalog select ID · select without executing",
        "cheats"=>match tokens.get(1).map(|t|t.value.as_str()) {
            Some("prepare")=>"cheats prepare CH-ID SOURCE MASTER GENERATION · review the preview",
            Some("apply-once")=>"cheats apply-once CH-ID SOURCE MASTER GENERATION TOKEN · apply once",
            Some("arm")=>"cheats arm CH-ID SOURCE MASTER GENERATION TOKEN · arm after preview",
            Some("disarm")=>"cheats disarm CH-ID|all · stop future writes, without restoring memory",
            Some("preset-save")=>"cheats preset-save NAME · save the armed set without applying it",
            Some("preset-prepare")=>"cheats preset-prepare NAME · inspect the complete group preview",
            Some("preset-arm")=>"cheats preset-arm NAME TOKEN · arm the reviewed group",
            _=>"cheats select ID | refresh | list | show | capabilities | prepare | apply-once | arm | disarm",
        },
        "content"=>"content refresh | content list [--offset N --limit 1..16 --generation N]",
        "replay"=>"replay run [plain|audit|capture|compare] · selected profile",
        "jobs"=>"jobs status | jobs cancel REQUEST-ID",
        "recipe"=>"recipe save NAME 'COMMAND'... | preview/prepare NAME | run/resume/cancel/checkpoint RUN",
        "retry"=>"retry RUN REQUEST-ID DEADLINE-UNIX-MS -- COMMAND · same request, no duplicates",
        "cancel"=>"cancel --request REQUEST-ID · cancel only the exact request",
        "step"=>"step N [--keys 0xMASK] · N 1..600; active-low mask",
        "inspect"=>"inspect 0xADDRESS LENGTH · 1..256 bytes, read-only",
        "break"=>"break 0xPC arm|thumb [--steps N] · interpreter",
        "watch"=>"watch read|write 0xSTART 0xEND [--steps N --width 1|2|4 --value 0xHEX]",
        "graphics"=>"graphics [--compare-last] · explicit capture",
        "audio-start"=>"audio-start [--frames N --streams LIST]",
        "code-start"=>"code-start [--frames N --events N --start 0xHEX --end 0xHEX --kind KIND --mode MODE]",
        "gameplay-start"=>"gameplay-start [--frames N --events N --family FAMILY --context 0xHEX]",
        "recent-start"=>"recent-start [--events N --reads on|off --on-error on|off --start 0xHEX --end 0xHEX]",
        "issue-report"=>"issue-report [--symptom TEXT] · uses selected profile",
        "status"|"pause"|"resume"|"ui-start"|"ui-report"|"audio-report"|"code-report"|"gameplay-report"|"recent-freeze"|"recent-report"=>"No arguments · Tab shows available completions",
        _=>"Tab completes names, options, values and IDs · help OPERATION-ID shows details",
    })
}

pub(crate) fn complete(text:&str,cursor:usize,profiles:&[String],catalogs:&[String],cheats:&[String],requests:&[String],operations:&[String])->Result<Completion,&'static str>{
    let tokens=lex(text,cursor)?;
    let active=tokens.iter().position(|t|t.range.start<=cursor&&cursor<=t.range.end);
    let range=active.map_or(cursor..cursor,|i|tokens[i].range.clone());
    let prefix=active.map_or("",|i|tokens[i].prefix.as_str());
    let full=active.map_or("",|i|tokens[i].value.as_str());
    let before:Vec<&str>=tokens.iter().enumerate().filter(|(i,t)|Some(*i)!=active&&t.range.end<=range.start).map(|(_,t)|t.value.as_str()).collect();
    let used:Vec<&str>=tokens.iter().enumerate().filter(|(i,_)|Some(*i)!=active).map(|(_,t)|t.value.as_str()).collect();
    let c=before.first().copied().unwrap_or("");
    let mut options:Vec<String>=if before.is_empty(){COMMANDS.iter().map(|s|(*s).into()).collect()}else{match (c,before.len()){
        ("help",1)=>operations.to_vec(),("profile"|"catalog",1)=>vec!["select".into()],
        ("content",1)=>vec!["list".into(),"refresh".into()],
        ("recipe",1)=>vec!["save".into(),"preview".into(),"prepare".into(),"checkpoint".into(),"run".into(),"resume".into(),"cancel".into()],
        ("cheats",1)=>vec!["select".into(),"refresh".into(),"list".into(),"show".into(),"capabilities".into(),"prepare".into(),"apply-once".into(),"arm".into(),"disarm".into(),"preset-save".into(),"preset-prepare".into(),"preset-arm".into()],
        ("cheats",2) if before[1]=="select"=>cheats.to_vec(),
        ("cheats",2) if before[1]=="disarm"=>vec!["all".into()],
        ("cheats",2) if before[1]=="show" || before[1]=="capabilities"=>Vec::new(),
        ("cheats",_) if ["list","show","capabilities"].contains(&before.get(1).copied().unwrap_or(""))=>{
            if before.last()==Some(&"--offset"){vec!["0".into()]}else if before.last()==Some(&"--limit"){vec!["16".into()]}else if before.last()==Some(&"--generation"){Vec::new()}else{
                let flags:Vec<&str>=if before[1]=="list"{vec!["--offset","--limit","--generation"]}else{vec!["--offset","--generation"]};flags.into_iter().filter(|flag|!used.contains(flag)).map(str::to_owned).collect()
            }
        },
        ("replay",1)=>vec!["run".into()],
        ("replay",2) if before[1]=="run"=>vec!["plain".into(),"audit".into(),"capture".into(),"compare".into()],
        ("content",_) if before.get(1)==Some(&"list")=>{
            if matches!(before.last(),Some(&"--offset")|Some(&"--generation")){vec!["0".into()]}
            else if before.last()==Some(&"--limit"){vec!["16".into()]}
            else{["--offset","--limit","--generation"].iter().filter(|flag|!used.contains(flag)).map(|s|(*s).to_owned()).collect()}
        },
        ("profile",2) if before[1]=="select"=>profiles.to_vec(),("catalog",2) if before[1]=="select"=>catalogs.to_vec(),
        ("jobs",1)=>vec!["status".into(),"cancel".into()],("jobs",2) if before[1]=="cancel"=>requests.to_vec(),
        ("cancel",2) if before[1]=="--request"=>requests.to_vec(),
        ("step",1)=>vec!["1".into(),"10".into()],("inspect"|"break",1)=>vec!["0x08000000".into()],
        ("inspect",2)=>vec!["16".into(),"32".into(),"64".into()],("break",2)=>vec!["arm".into(),"thumb".into()],
        ("watch",1)=>vec!["read".into(),"write".into()],("watch",2)=>vec!["0x02000000".into()],("watch",3)=>vec!["0x02000004".into()],
        _=>{
            let positional=match c{"step"=>2,"break"=>3,"watch"=>4,"inspect"=>usize::MAX,_=>1};
            if before.len()<positional{Vec::new()}else if before.last().is_some_and(|last|flags(c).contains(last)&&*last!="--compare-last"){
                values(before.last().unwrap(),c)
            }else{flags(c).iter().filter(|flag|!used.contains(flag)).map(|s|(*s).to_owned()).collect()}
        }
    }};
    if c=="audio-start"&&before.last()==Some(&"--streams")&&prefix.contains(','){
        let (head,tail)=prefix.rsplit_once(',').unwrap();let existing:Vec<_>=head.split(',').collect();
        options=values("--streams",c).into_iter().filter(|v|!existing.contains(&v.as_str())&&v.starts_with(tail)).map(|v|format!("{head},{v}")).collect();
    }
    options.retain(|v|v.starts_with(prefix)&&(cursor==range.end||v.starts_with(full)));
    options.sort();options.dedup();options.truncate(20);
    let at_end=range.end==text.len();
    Ok(Completion{alternatives:options,range,at_end})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn choices(text:&str,cursor:usize)->Completion{complete(text,cursor,&["content-profile".into()],&["content-catalog".into()],&["content-cheat".into()],&["req-42".into()],&["control.step".into()]).unwrap()}
    #[test]
    fn flags_values_and_pending_ids_follow_the_actual_grammar(){
        for (text,expected) in [
            ("step 1 --k",vec!["--keys"]),
            ("watch read 0x02000000 0x02000010 --width ",vec!["1","2","4"]),
            ("jobs cancel req",vec!["req-42"]),
            ("cheats select content-c",vec!["content-cheat"]),
            ("cheats a",vec!["apply-once","arm"]),
            ("cheats dis",vec!["disarm"]),
            ("cheats disarm a",vec!["all"]),
            ("cheats cap",vec!["capabilities"]),
            ("cheats capabilities CH-0001 --o",vec!["--offset"]),
            ("replay run c",vec!["capture","compare"]),
            ("code-start --mode ",vec!["all","arm","thumb"]),
            ("audio-start --streams mixed,ps",vec!["mixed,psg"]),
        ] {assert_eq!(choices(text,text.len()).alternatives,expected,"{text}");}
        let text="step 1 --keys 0x3ff ";assert!(!choices(text,text.len()).alternatives.contains(&"--keys".into()));
    }
    #[test]
    fn hints_follow_language_without_changing_command_syntax(){
        let en=input_config::Language::default();let it=input_config::Language::parse("it");
        assert!(hint("step 1",en).contains("active-low mask"));
        assert!(hint("step 1",it).contains("maschera active-low"));
        assert!(hint("cheats arm",en).contains("arm after preview"));
        assert!(hint("cheats arm",it).contains("anteprima"));
        assert!(hint("cheats disarm",en).contains("without restoring memory"));
        assert!(hint("cheats disarm",it).contains("senza ripristinare la memoria"));
        for language in [en,it] {assert!(hint("step 1",language).starts_with("step N [--keys 0xMASK]"));}
    }
    #[test]
    fn quoted_unicode_cursor_and_multiline_are_non_destructive(){
        let text="catalog select 'content-cat";let c=choices(text,text.len());assert_eq!(c.alternatives,vec!["content-catalog"]);
        assert_eq!(&text[c.range.clone()],"'content-cat");assert_eq!(encoded(&c.alternatives[0],c.at_end),"content-catalog ");
        let text="step 1 --keys 0x3ff";let c=choices(text,10);assert_eq!(c.alternatives,vec!["--keys"]);assert_eq!(&text[c.range],"--keys");
        let text="profile select 'é 🐱'";let c=choices(text,text.len());assert!(c.alternatives.is_empty());
        assert!(complete("status\npause",6,&[],&[],&[],&[],&[]).is_err());
        assert!(complete("help é",6,&[],&[],&[],&[],&[]).is_err());
        assert_eq!(quoted("a \"猫\""),"\"a \\\"猫\\\"\"");
        assert_eq!(hint("'step' 1",input_config::Language::default()),hint("step 1",input_config::Language::default()));
        let text="status\u{2003}pause";let c=choices(text,6);assert_eq!(c.alternatives,vec!["status"]);
        let text="profile select \"bad\\n";assert!(complete(text,text.len(),&[],&[],&[],&[],&[]).is_err());
    }
    #[test]
    fn cycling_is_explicit_bounded_and_invalidated_when_choices_change(){
        let mut e=super::super::editor::Editor::default();let mut cycle=Cycle::default();
        let ids=vec!["content-b".into(),"content-a".into()];e.insert("catalog select content-").unwrap();
        assert!(matches!(cycle.apply(&mut e,&[],&ids,&[],&[],&[]),Ok(Applied::Choice{index:1,total:2,..})));assert_eq!(e.text(),"catalog select content-a ");
        assert!(matches!(cycle.apply(&mut e,&[],&ids,&[],&[],&[]),Ok(Applied::Choice{index:2,total:2,..})));assert_eq!(e.text(),"catalog select content-b ");
        let changed=vec!["content-a".into()];assert!(matches!(cycle.apply(&mut e,&[],&changed,&[],&[],&[]),Ok(Applied::None)));assert_eq!(e.text(),"catalog select content-b ");
    }
}
