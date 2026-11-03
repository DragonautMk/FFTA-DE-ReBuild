//! Parse debugger commands without dispatching handlers or interpreting a shell.
use crate::debug_control::Action;
pub(crate) mod editor;
pub(crate) mod controller;
pub(crate) mod completion;
pub(crate) mod observation;

const MAX_BYTES: usize = 4096;
const MAX_TOKENS: usize = 64;
const MAX_ID_BYTES: usize = 512;

#[derive(Debug)]
pub(crate) enum Command {
    Help(Option<String>),
    Control(Action),
    Select { slot: menu::ContentSlot, id: String },
    JobsStatus,
    Retry { run: String, id: String, deadline_unix_ms: u64, action: Action },
}

/// Whitespace separates tokens; single/double quotes group text. Backslash
/// escapes only the active quote or another backslash inside quoted text.
/// There is deliberately no expansion, substitution, pipeline or shell.
pub(crate) fn tokenize(input:&str)->Result<Vec<String>,String> {
    if input.len()>MAX_BYTES {return Err("Command exceeds 4096 bytes".into());}
    if input.chars().any(|c|c.is_control() && c!='\t') {return Err("Command must be one line without control characters".into());}
    let mut tokens=Vec::new();let mut token=String::new();let mut quote=None;let mut started=false;
    let mut chars=input.chars().peekable();
    while let Some(ch)=chars.next() {
        if let Some(q)=quote {
            if ch==q {quote=None;} else if ch=='\\' {
                match chars.next() {Some(next) if next==q || next=='\\'=>token.push(next),_=>return Err("Only quoted quote/backslash escapes are supported".into())}
            } else {token.push(ch);}
        } else if ch=='\'' || ch=='"' {quote=Some(ch);started=true;}
        else if ch.is_whitespace() {
            if started {tokens.push(std::mem::take(&mut token));started=false;if tokens.len()>MAX_TOKENS{return Err("Too many command tokens".into());}}
        } else {token.push(ch);started=true;}
    }
    if quote.is_some(){return Err("Unterminated quote".into());}
    if started {tokens.push(token);}
    if tokens.len()>MAX_TOKENS{return Err("Too many command tokens".into());}
    if tokens.is_empty(){return Err("Empty command".into());}
    Ok(tokens)
}

fn id(value:&str,max:usize)->Result<String,String>{
    if value.is_empty() || value.len()>max || value.chars().any(char::is_control) {return Err("Invalid identifier".into());}
    Ok(value.to_owned())
}
fn decimal(v:&str)->Result<(),String>{
    if v.is_empty() || !v.bytes().all(|c|c.is_ascii_digit()) || (v.len()>1 && v.starts_with('0')) {return Err("Use unambiguous decimal digits without leading zeroes".into());}Ok(())
}
fn hex(v:&str)->Result<(),String>{
    let digits=v.strip_prefix("0x").ok_or("Use 0x-prefixed hexadecimal values")?;
    if digits.is_empty() || !digits.bytes().all(|c|c.is_ascii_hexdigit()){return Err("Invalid 0x-prefixed hexadecimal value".into());}Ok(())
}
fn numeric_syntax(t:&[String])->Result<(),String>{
    match t[0].as_str(){
        "step"=>{if let Some(v)=t.get(1){decimal(v)?;}},
        "inspect"=>{if let Some(v)=t.get(1){hex(v)?;}if let Some(v)=t.get(2){decimal(v)?;}},
        "break"=>{if let Some(v)=t.get(1){hex(v)?;}},
        "watch"=>{if let Some(v)=t.get(2){hex(v)?;}if let Some(v)=t.get(3){hex(v)?;}},
        _=>{}
    }
    for pair in t.windows(2){match pair[0].as_str(){
        "--keys"|"--start"|"--end"|"--context"|"--value"=>hex(&pair[1])?,
        "--steps"|"--width"|"--events"|"--frames"|"--offset"|"--limit"|"--generation"=>decimal(&pair[1])?,_=>{}
    }}
    Ok(())
}

/// Parse to existing typed operations. This function performs no filesystem,
/// menu, session or machine action; dispatch is a separate frontend boundary.
pub(crate) fn parse(input:&str)->Result<Command,String>{
    let t=tokenize(input)?;
    parse_tokens(&t,true)
}
fn parse_tokens(t:&[String],allow_retry:bool)->Result<Command,String>{
    if t.first().is_some_and(|v|v=="retry") {
        if !allow_retry || t.len()<6 || t[4]!="--" {return Err("Use retry RUN REQUEST-ID DEADLINE-UNIX-MS -- COMMAND".into());}
        let run=id(&t[1],100)?;let request=id(&t[2],100)?;decimal(&t[3])?;
        let deadline_unix_ms=t[3].parse().map_err(|_|"Invalid original deadline")?;
        let action=match parse_tokens(&t[5..],false)? {
            Command::Control(Action::Status|Action::ContentList{..}|Action::CheatList{..}|Action::CheatShow{..})|Command::JobsStatus=>return Err("Read-only query: request a fresh result instead".into()),
            Command::Control(action)=>action,
            Command::Select{slot,id}=>Action::SelectId{slot:match slot {menu::ContentSlot::Replay=>crate::debug_control::SelectionSlot::Replay,menu::ContentSlot::Catalog=>crate::debug_control::SelectionSlot::Catalog,menu::ContentSlot::CheatCatalog=>crate::debug_control::SelectionSlot::CheatCatalog,_=>return Err("Unsupported selection slot".into())},id},
            _=>return Err("Retry requires an executable command".into()),
        };
        return Ok(Command::Retry{run,id:request,deadline_unix_ms,action});
    }
    match t {
        [family,verb,rest @ ..] if family=="cheats" && ["refresh","list","show","prepare","apply-once","arm","disarm"].contains(&verb.as_str())=>{
            let name=if verb=="apply-once"{"cheat-apply".into()}else{format!("cheat-{verb}")};
            let mut args=vec![name,"<current-session>".into()];for pair in rest.windows(2){if ["--offset","--limit"].contains(&pair[0].as_str()){decimal(&pair[1])?;}}args.extend_from_slice(rest);return crate::debug_control::parse(&args).map(|(_,a)|Command::Control(a));
        },
        [family,verb,rest @ ..] if family=="replay" && verb=="run"=>{
            let mode=match rest {[]=>"plain",[mode]=>mode.as_str(),_=>return Err("Use replay run [plain|audit|capture|compare]".into())};
            let args=vec!["replay-run".into(),"<current-session>".into(),"--mode".into(),mode.into()];return crate::debug_control::parse(&args).map(|(_,a)|Command::Control(a));
        },
        [family,verb,rest @ ..] if family=="content" && (verb=="refresh"||verb=="list")=>{
            let mut args=vec![if verb=="refresh"{"content-refresh".into()}else{"content-list".into()},"<current-session>".into()];
            numeric_syntax(t)?;args.extend_from_slice(rest);return crate::debug_control::parse(&args).map(|(_,a)|Command::Control(a));
        },
        [v] if v=="help"=>return Ok(Command::Help(None)),
        [v,name] if v=="help"=>{
            if crate::debug_operations::get(name).is_none(){return Err("Unknown operation ID".into());}
            return Ok(Command::Help(Some(name.clone())));
        },
        [family,verb,value] if verb=="select" && (family=="profile"||family=="catalog"||family=="cheats")=>{
            return Ok(Command::Select{slot:if family=="profile"{menu::ContentSlot::Replay}else if family=="catalog"{menu::ContentSlot::Catalog}else{menu::ContentSlot::CheatCatalog},id:id(value,MAX_ID_BYTES)?});
        },
        [family,verb] if family=="jobs" && verb=="status"=>return Ok(Command::JobsStatus),
        [family,verb,value] if family=="jobs" && verb=="cancel"=>{
            let args=vec!["cancel".into(),"<current-session>".into(),"--request".into(),id(value,100)?];
            return crate::debug_control::parse(&args).map(|(_,a)|Command::Control(a));
        },
        _=>{}
    }
    if t[0]=="issue-report" && t.iter().any(|v|v=="--profile") {return Err("Select a profile by ID first; paths are not accepted".into());}
    numeric_syntax(&t)?;
    let mut args=Vec::with_capacity(t.len()+1);args.push(t[0].clone());args.push("<current-session>".into());args.extend(t[1..].iter().cloned());
    crate::debug_control::parse(&args).map(|(_,a)|Command::Control(a))
}


pub(crate) fn parse_error(error:&str)->serde_json::Value {
    local_error("parse-error",error)
}
pub(crate) fn local_error(outcome:&str,error:&str)->serde_json::Value {
    serde_json::json!({"schema":1,"kind":"ffta-debug-command","ok":false,"outcome":outcome,"error":error,"delivery":"not-sent"})
}
/// CLI and panel share typed parsing, local failure classification and the
/// unchanged server response, including unacknowledged timeout details.
pub(crate) fn execute(root:&std::path::Path,input:&str)->serde_json::Value {
    let command=match parse(input){Ok(command)=>command,Err(error)=>return parse_error(&error)};
    let result=match command {
        Command::Help(None)=>return serde_json::json!({"schema":1,"kind":"ffta-debug-command-help","operations":crate::debug_operations::registry().operations}),
        Command::Help(Some(id))=>return serde_json::json!({"schema":1,"kind":"ffta-debug-command-help","operation":crate::debug_operations::get(&id)}),
        Command::JobsStatus=>crate::debug_control::execute_action_classified(root,Action::Status,120_000),
        Command::Control(action)=>crate::debug_control::execute_action_classified(root,action,120_000),
        Command::Retry{run,id,deadline_unix_ms,action}=>crate::debug_control::retry_action_classified(root,action,120_000,run,id,deadline_unix_ms),
        Command::Select{slot,id}=>{
            let slot=match slot {menu::ContentSlot::Replay=>crate::debug_control::SelectionSlot::Replay,menu::ContentSlot::Catalog=>crate::debug_control::SelectionSlot::Catalog,menu::ContentSlot::CheatCatalog=>crate::debug_control::SelectionSlot::CheatCatalog,_=>return parse_error("Unsupported selection slot")};
            crate::debug_control::execute_action_classified(root,Action::SelectId{slot,id},120_000)
        },
    };
    result.unwrap_or_else(|error|error.value("ffta-debug-command"))
}
pub(crate) fn cmd(args:&[String])->Result<(),String>{
    let value=if args.len()!=2{parse_error("Use debug-command SESSION 'COMMAND'")}else{execute(std::path::Path::new(&args[0]),&args[1])};
    println!("{value}");if value.get("ok")==Some(&serde_json::json!(false)){Err(value["error"].as_str().unwrap_or("Command failed").into())}else{Ok(())}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_are_bounded_and_never_execute_or_expand() {
        assert_eq!(tokenize("issue-report --symptom \"é and \\\"quoted\\\"\"").unwrap(),vec!["issue-report","--symptom","é and \"quoted\""]);
        assert_eq!(tokenize("'$(not executed)' \\").unwrap(),vec!["$(not executed)","\\"]);
        for input in ["", "'unfinished", "one\ntwo", "\"bad\\n\""] {assert!(tokenize(input).is_err(),"{input:?}");}
        assert!(tokenize(&"x".repeat(MAX_BYTES+1)).is_err());assert!(tokenize(&"a ".repeat(MAX_TOKENS+1)).is_err());
    }
    #[test]
    fn shared_control_parser_and_typed_selection_are_unambiguous() {
        let cases=[("pause",vec!["pause","." ]),("step 1 --keys 0x3ff",vec!["step",".","1","--keys","0x3ff"]),("inspect 0x8000000 16",vec!["inspect",".","0x8000000","16"]),("break 0x8000000 thumb --steps 100",vec!["break",".","0x8000000","thumb","--steps","100"]),("watch read 0x2000000 0x2000010 --width 2",vec!["watch",".","read","0x2000000","0x2000010","--width","2"])];
        for (text,args) in cases {let Command::Control(a)=parse(text).unwrap() else{panic!("{text}")};let (_,b)=crate::debug_control::parse(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).unwrap();assert_eq!(serde_json::to_value(a).unwrap(),serde_json::to_value(b).unwrap());}
        let Command::Select{slot,id}=parse("profile select 'root:opaque id'").unwrap() else{panic!()};assert_eq!(slot,menu::ContentSlot::Replay);assert_eq!(id,"root:opaque id");
        assert!(matches!(parse("jobs status"),Ok(Command::JobsStatus)));assert!(matches!(parse("help control.step"),Ok(Command::Help(Some(_)))));
        for text in ["step 01","step 0","step 601","step 1 --keys 3ff","inspect 8000000 16","inspect 0x8000000 0","break 0x8000000 invalid","watch read 0x2000000 0x2000000","issue-report --profile /tmp/x","jobs cancel ''","profile select ''","unknown","help unknown"] {assert!(parse(text).is_err(),"{text}");}
    }
    #[test]
    fn retry_reuses_exact_typed_command_and_rejects_invalid_or_nested_input(){
        let Command::Retry{run,id,deadline_unix_ms,action}=parse("retry run-1 request-2 12345 -- profile select 'content id'").unwrap() else{panic!()};
        assert_eq!((run.as_str(),id.as_str(),deadline_unix_ms),("run-1","request-2",12345));
        assert!(matches!(action,Action::SelectId{slot:crate::debug_control::SelectionSlot::Replay,id} if id=="content id"));
        let Command::Retry{action,..}=parse("retry run id 123 -- step 1 --keys 0x3ff").unwrap() else{panic!()};
        assert!(matches!(action,Action::Step{frames:1,keys:0x3ff}));
        for input in ["retry run id 0123 -- step 1","retry run id 123 step 1","retry run id 123 -- status","retry run id 123 -- jobs status","retry run id 123 -- help","retry run id 123 -- retry run id 123 -- step 1","retry run id 123 -- step 01","retry '' id 123 -- step 1"]{assert!(parse(input).is_err(),"{input}");}
    }
    #[test]
    fn content_discovery_uses_bounded_typed_requests(){
        assert!(matches!(parse("content refresh"),Ok(Command::Control(Action::ContentRefresh))));
        assert!(matches!(parse("content list --offset 2 --limit 1 --generation 3"),Ok(Command::Control(Action::ContentList{offset:2,limit:1,generation:Some(3)}))));
        for input in ["content refresh extra","content list --limit 17","content list --limit 0","content list --offset 01","content list --limit 2 --limit 3","retry run id 123 -- content list"]{assert!(parse(input).is_err(),"{input}");}
    }
    #[test]
    fn cheat_commands_are_typed_bounded_and_non_shell(){
        assert!(matches!(parse("cheats select content-123"),Ok(Command::Select{slot:menu::ContentSlot::CheatCatalog,..})));
        assert!(matches!(parse("cheats refresh"),Ok(Command::Control(Action::CheatRefresh))));
        assert!(matches!(parse("cheats list --offset 2 --limit 1"),Ok(Command::Control(Action::CheatList{offset:2,limit:1,..}))));
        assert!(matches!(parse("cheats show CH-0001 --offset 4096"),Ok(Command::Control(Action::CheatShow{id,offset:4096,..})) if id=="CH-0001"));
        assert!(matches!(parse("retry run id 123 -- cheats select content-123"),Ok(Command::Retry{action:Action::SelectId{slot:crate::debug_control::SelectionSlot::CheatCatalog,..},..})));
        let hash="a".repeat(64);let preview="b".repeat(64);
        assert!(matches!(parse(&format!("cheats prepare CH-0001 0 CH-0002 {hash}")),Ok(Command::Control(Action::CheatPrepare{provenance:0,..}))));
        assert!(matches!(parse(&format!("cheats apply-once CH-0001 0 CH-0002 {hash} {preview}")),Ok(Command::Control(Action::CheatApply{provenance:0,..}))));
        assert!(matches!(parse(&format!("cheats arm CH-0001 0 CH-0002 {hash} {preview}")),Ok(Command::Control(Action::CheatArm{provenance:0,..}))));
        assert!(matches!(parse("cheats disarm CH-0001"),Ok(Command::Control(Action::CheatDisarm{id:Some(id)})) if id=="CH-0001"));
        assert!(matches!(parse("cheats disarm all"),Ok(Command::Control(Action::CheatDisarm{id:None}))));
        assert!(parse("cheats disarm invalid").is_err());
        assert!(parse(&format!("cheats prepare CH-0001 00 CH-0002 {hash}")).is_err());
        for text in ["cheats list --offset 01","cheats list --limit 17","cheats show /tmp/code","cheats show CH-0001 --generation invalid","cheats refresh extra","retry run id 123 -- cheats list"]{assert!(parse(text).is_err(),"{text}");}
    }
    #[test]
    fn replay_grammar_uses_shared_typed_operations_and_rejects_paths(){
        for (text,operation) in [("replay run","replay.run"),("replay run audit","audit.run"),("replay run capture","capture.run"),("replay run compare","compare.run")] {
            let Command::Control(action)=parse(text).unwrap() else{panic!("{text}")};assert_eq!(action.operation_id(),operation);
        }
        assert!(matches!(parse("retry run id 123 -- replay run audit"),Ok(Command::Retry{action:Action::Replay{..},..})));
        for text in ["replay","replay run /tmp/profile","replay run plain extra","replay run invalid"]{assert!(parse(text).is_err(),"{text}");}
    }
    #[test]
    fn parse_error_and_unavailable_session_have_distinct_copyable_results(){
        let t=tempfile::tempdir().unwrap();let syntax=execute(t.path(),"step 01");
        assert_eq!((syntax["outcome"].as_str(),syntax["delivery"].as_str()),(Some("parse-error"),Some("not-sent")));
        let unavailable=execute(t.path(),"status");assert_eq!((unavailable["outcome"].as_str(),unavailable["delivery"].as_str()),(Some("unavailable"),Some("not-sent")));
        assert!(!t.path().join("Data").exists());
    }
    #[test]
    fn completed_arguments_use_the_existing_parser_and_preserve_other_tokens(){
        let mut editor=editor::Editor::default();editor.insert("step 1 --k").unwrap();
        let mut cycle=completion::Cycle::default();assert!(matches!(cycle.apply(&mut editor,&[],&[],&[],&[],&[]),Ok(completion::Applied::One)));editor.insert("0x3ff").unwrap();
        assert!(matches!(parse(editor.text()),Ok(Command::Control(Action::Step{frames:1,keys:0x3ff}))));
        editor.clear();editor.insert("watch r 0x02000000 0x02000004").unwrap();editor.set_cursor(7,false);
        assert!(matches!(cycle.apply(&mut editor,&[],&[],&[],&[],&[]),Ok(completion::Applied::One)));
        assert!(matches!(parse(editor.text()),Ok(Command::Control(Action::Find{..}))));
    }
    #[test]
    fn ambiguous_id_completion_cycles_without_dispatch_and_rechecks_candidates(){
        let mut e=editor::Editor::default();let mut cycle=completion::Cycle::default();let ids=vec!["content-b".into(),"content-a".into()];e.insert("catalog select content-").unwrap();
        assert!(matches!(cycle.apply(&mut e,&[],&ids,&[],&[],&[]),Ok(completion::Applied::Choice{index:1,total:2,..})));assert_eq!(e.text(),"catalog select content-a ");
        assert!(matches!(cycle.apply(&mut e,&[],&ids,&[],&[],&[]),Ok(completion::Applied::Choice{index:2,total:2,..})));assert_eq!(e.text(),"catalog select content-b ");
        let changed=vec!["content-a".into()];assert!(matches!(cycle.apply(&mut e,&[],&changed,&[],&[],&[]),Ok(completion::Applied::None)));assert_eq!(e.text(),"catalog select content-b ");
        e.clear();e.insert("step 1 --keys 0x3ff").unwrap();e.set_cursor(10,false);assert!(matches!(cycle.apply(&mut e,&[],&[],&[],&[],&[]),Ok(completion::Applied::One)));assert_eq!(e.text(),"step 1 --keys 0x3ff");
    }
}
