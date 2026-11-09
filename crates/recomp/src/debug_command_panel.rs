//! Host command window. Submissions use the same local-control client as the CLI.
use std::{collections::VecDeque, num::NonZeroU32, path::PathBuf, sync::Arc, thread::JoinHandle, time::{Duration,Instant}};
use winit::{event::{ElementState,WindowEvent},event_loop::ActiveEventLoop,keyboard::{KeyCode,ModifiersState,PhysicalKey},window::Window};
use crate::debug_command::{self,controller::{self,Controller},editor::Editor};
use input_config::Language;
use menu::i18n::format as tr;

type Job=JoinHandle<serde_json::Value>;
fn insert_unidentified_text(editor:&mut Editor,text:Option<&str>,modifiers:ModifiersState,context_visible:bool)->Result<(),&'static str>{
    if !context_visible && !modifiers.control_key() && !modifiers.alt_key() && !modifiers.super_key() && editor.preedit().is_empty(){
        if let Some(text)=text{return editor.insert(text);}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unidentified_physical_key_preserves_layout_text_and_input_guards() {
        let mut editor=Editor::default();
        insert_unidentified_text(&mut editor,Some("è"),ModifiersState::empty(),false).unwrap();
        insert_unidentified_text(&mut editor,Some("É"),ModifiersState::SHIFT,false).unwrap();
        assert_eq!(editor.text(),"èÉ");
        for modifiers in [ModifiersState::CONTROL,ModifiersState::ALT,ModifiersState::SUPER] {
            insert_unidentified_text(&mut editor,Some("x"),modifiers,false).unwrap();
        }
        insert_unidentified_text(&mut editor,Some("x"),ModifiersState::empty(),true).unwrap();
        editor.set_preedit("候");
        insert_unidentified_text(&mut editor,Some("x"),ModifiersState::empty(),false).unwrap();
        assert_eq!(editor.text(),"èÉ");
    }
}
pub(crate) enum ControllerEvent {None,Complete,Hide}
pub(crate) struct Panel {
    window:Arc<Window>,
    #[cfg(target_os="windows")]
    wine_ime:Option<crate::debug_command_wine_ime::Shim>,
    _context:softbuffer::Context<Arc<Window>>,
    surface:softbuffer::Surface<Arc<Window>,Arc<Window>>,
    visible:bool,
    focused:bool,
    focus_handoff:Option<Instant>,
    hide_handoff:Option<Instant>,
    last_draw:Instant,
    last_observation:Instant,
    language:Language,
    manifest:Vec<u8>,
    run:String,
    manifest_sha256:String,
    target:String,
    pending:String,
    parameters:String,
    pins:Vec<menu::command_console::PinLine>,
    context_visible:bool,
    context_scroll:usize,
    modifiers:ModifiersState,
    editor:Editor,
    completion:debug_command::completion::Cycle,
    controller:Controller,
    clipboard:Option<arboard::Clipboard>,
    jobs:Vec<(String,Job)>,
    lines:VecDeque<String>,
    last_result:String,
    notice:String,
    root:PathBuf,
}
impl Panel {
    pub fn open(event_loop:&ActiveEventLoop,root:PathBuf,run:&str,manifest_sha256:&str,language:Language)->Result<Self,String> {
        let manifest=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)?;
        let target=debug_command::observation::verified_target(&manifest,run,manifest_sha256,language)?;
        let window=Arc::new(event_loop.create_window(Window::default_attributes().with_title(format!("FFTA — {}",language.text("Command console"))).with_inner_size(winit::dpi::LogicalSize::new(900.0,560.0)).with_min_inner_size(winit::dpi::LogicalSize::new(480.0,360.0))).map_err(|e|e.to_string())?);
        let context=softbuffer::Context::new(window.clone()).map_err(|e|e.to_string())?;
        let surface=softbuffer::Surface::new(&context,window.clone()).map_err(|e|e.to_string())?;
        #[cfg(target_os="windows")]
        let wine_ime=crate::debug_command_wine_ime::Shim::install(window.clone());
        window.set_ime_allowed(true);
        Ok(Self{window,#[cfg(target_os="windows")] wine_ime,_context:context,surface,visible:true,focused:true,focus_handoff:Some(Instant::now()),hide_handoff:None,last_draw:Instant::now()-Duration::from_secs(1),last_observation:Instant::now()-Duration::from_secs(1),language,manifest,run:run.into(),manifest_sha256:manifest_sha256.into(),target,pending:language.text("No pending job").into(),parameters:language.text("Parameters: -").into(),pins:Vec::new(),context_visible:false,context_scroll:0,modifiers:ModifiersState::empty(),editor:Editor::default(),completion:Default::default(),controller:Controller::default(),clipboard:None,jobs:Vec::new(),lines:VecDeque::new(),last_result:String::new(),notice:language.text("Type a command; submission is explicit.").into(),root})
    }
    pub fn owns(&self,id:winit::window::WindowId)->bool{self.window.id()==id}
    pub fn show(&mut self,language:Language){
        if self.language!=language {
            self.language=language;
            if let Ok(target)=debug_command::observation::verified_target(&self.manifest,&self.run,&self.manifest_sha256,language){self.target=target;}
            self.window.set_title(&format!("FFTA — {}",language.text("Command console")));
        }
        self.visible=true;self.focused=true;self.controller.reset();self.hide_handoff=None;self.focus_handoff=Some(Instant::now());self.window.set_visible(true);self.window.set_ime_allowed(true);self.window.focus_window();self.last_draw=Instant::now()-Duration::from_secs(1);self.last_observation=Instant::now()-Duration::from_secs(1);}
    pub fn hide(&mut self){self.visible=false;self.focused=false;self.context_visible=false;self.editor.set_preedit("");#[cfg(target_os="windows")] if let Some(ime)=self.wine_ime.as_mut(){ime.reset();}self.controller.reset();self.focus_handoff=None;self.hide_handoff=Some(Instant::now());self.window.set_ime_allowed(false);self.window.set_visible(false);}
    pub fn focused(&self)->bool{self.visible&&self.focused}
    pub fn wants_observation(&self)->bool{self.visible&&self.last_observation.elapsed()>=Duration::from_millis(250)}
    pub fn observe(&mut self,pending:&serde_json::Value,remaining:u32,search_remaining:Option<u64>,replay_progress:Option<(u64,u64)>,pins:&serde_json::Value){
        self.last_observation=Instant::now();let (job,parameters)=debug_command::observation::pending(pending,remaining,search_remaining,replay_progress,self.language);
        if self.pending!=job||self.parameters!=parameters{self.pending=job;self.parameters=parameters;self.last_draw=Instant::now()-Duration::from_secs(1);}
        let pins=debug_command::observation::pin_lines(pins);if self.pins!=pins{self.pins=pins;self.last_draw=Instant::now()-Duration::from_secs(1);}
    }
    pub fn focus_handoff(&self)->bool{self.focus_handoff.is_some_and(|t|t.elapsed()<Duration::from_secs(2))}
    pub fn hide_handoff(&self)->bool{self.hide_handoff.is_some_and(|t|t.elapsed()<Duration::from_millis(400))}
    fn clipboard(&mut self)->Result<&mut arboard::Clipboard,String>{
        if self.clipboard.is_none(){self.clipboard=Some(arboard::Clipboard::new().map_err(|e|e.to_string())?);}
        Ok(self.clipboard.as_mut().unwrap())
    }
    fn line(&mut self,text:String){if self.lines.len()==32{self.lines.pop_front();}self.lines.push_back(text.chars().take(512).collect());}
    fn submit(&mut self){
        if !self.editor.preedit().is_empty(){let error="Complete text composition first.";self.last_result=serde_json::to_string_pretty(&debug_command::local_error("composition-active",error)).unwrap_or_default();self.notice=tr(self.language,"{error} Ctrl+Shift+C copies JSON.",&[("error",self.language.text(error).into())]);return;}
        let command=self.editor.text().to_owned();
        if let Err(error)=debug_command::parse(&command){
            self.last_result=serde_json::to_string_pretty(&debug_command::parse_error(&error)).unwrap_or_default();
            self.notice=tr(self.language,"Syntax error: {error} · Ctrl+Shift+C copies JSON",&[("error",error.to_string())]);return;
        }
        if self.jobs.len()>=4{let error="Four requests are still in progress.";self.last_result=serde_json::to_string_pretty(&debug_command::local_error("unavailable",error)).unwrap_or_default();self.notice=tr(self.language,"{error} Ctrl+Shift+C copies JSON.",&[("error",self.language.text(error).into())]);return;}
        let root=self.root.clone();let submitted=command.clone();
        self.jobs.push((command.clone(),std::thread::spawn(move||debug_command::execute(&root,&submitted))));
        self.line(format!("> {command}"));self.editor.submitted();self.notice=self.language.text("Request submitted; status and cancellation remain available.").into();
    }
    pub fn complete(&mut self,profiles:&[String],catalogs:&[String],cheats:&[String],requests:&[String]){
        let operations=crate::debug_operations::registry().operations.iter().map(|o|o.id.clone()).collect::<Vec<_>>();
        match self.completion.apply(&mut self.editor,profiles,catalogs,cheats,requests,&operations){
            Err(error)=>self.notice=self.language.text(error).into(),
            Ok(debug_command::completion::Applied::None)=>self.notice=self.language.text("No completion available.").into(),
            Ok(debug_command::completion::Applied::One)=>self.notice=self.language.text("Completed.").into(),
            Ok(debug_command::completion::Applied::Choice{index,total,value})=>self.notice=tr(self.language,"{index}/{total}: {value} · Tab for next",&[("index",index.to_string()),("total",total.to_string()),("value",value)]),
        }
        self.last_draw=Instant::now()-Duration::from_secs(1);
    }
    pub fn controller_input(&mut self,input:controller::Input)->ControllerEvent {
        use controller::Action as A;
        let Some(action)=self.controller.poll(input) else{return ControllerEvent::None;};
        self.last_draw=Instant::now()-Duration::from_secs(1);
        if self.context_visible {return ControllerEvent::None;}
        match action {
            A::Changed=>{},
            A::Insert(text)=>if let Err(error)=self.editor.insert(&text){self.notice=error.into();},
            A::Backspace=>self.editor.backspace(),A::Clear=>self.editor.clear(),
            A::Previous=>self.editor.history_previous(),A::Next=>self.editor.history_next(),
            A::Left=>self.editor.left(false),A::Right=>self.editor.right(false),
            A::Complete=>return ControllerEvent::Complete,A::Submit=>self.submit(),
            A::Close=>return ControllerEvent::Hide,
            A::Cancel=>{if self.editor.text().is_empty(){return ControllerEvent::Hide;}self.editor.clear();self.notice=self.language.text("Draft cancelled.").into();},
        }
        ControllerEvent::None
    }
    pub fn event(&mut self,event:&WindowEvent,profiles:&[String],catalogs:&[String],cheats:&[String],requests:&[String])->bool {
        match event {
            WindowEvent::CloseRequested=>return true,
            WindowEvent::Focused(focused)=>{self.focused=*focused;self.controller.reset();if *focused{self.focus_handoff=None;}},
            WindowEvent::ModifiersChanged(m)=>self.modifiers=m.state(),
            WindowEvent::Ime(ime) if !self.context_visible || matches!(ime,winit::event::Ime::Disabled)=>{if let Err(e)=self.editor.ime(ime){self.notice=e.into();}},
            WindowEvent::KeyboardInput{event,is_synthetic:false,..} if event.state==ElementState::Pressed=>{
                let PhysicalKey::Code(code)=event.physical_key else{
                    if let Err(e)=insert_unidentified_text(&mut self.editor,event.text.as_deref(),self.modifiers,self.context_visible){self.notice=e.into();}
                    self.last_draw=Instant::now()-Duration::from_secs(1);return false;
                };
                let control=self.modifiers.control_key();let shift=self.modifiers.shift_key();
                if self.context_visible {
                    match code {
                        KeyCode::F2|KeyCode::Escape if !event.repeat=>{self.context_visible=false;self.controller.reset();},
                        KeyCode::ArrowUp=>self.context_scroll=self.context_scroll.saturating_sub(1),
                        KeyCode::ArrowDown=>self.context_scroll=self.context_scroll.saturating_add(1),
                        KeyCode::PageUp=>self.context_scroll=self.context_scroll.saturating_sub(8),
                        KeyCode::PageDown=>self.context_scroll=self.context_scroll.saturating_add(8),
                        _=>{},
                    }
                    self.last_draw=Instant::now()-Duration::from_secs(1);return false;
                }
                if code==KeyCode::F2 && !event.repeat && !control && !shift && !self.modifiers.alt_key() && !self.modifiers.super_key() && self.editor.preedit().is_empty(){
                    self.context_visible=true;self.context_scroll=0;self.last_draw=Instant::now()-Duration::from_secs(1);return false;
                }
                if control {
                    match code {
                        KeyCode::KeyA=>self.editor.select_all(),
                        KeyCode::KeyC=>{let text=if shift{self.last_result.clone()}else{self.editor.selected_text().to_owned()};self.notice=match self.clipboard().and_then(|c|c.set_text(text).map_err(|e|e.to_string())){Ok(())=>self.language.text("Copied.").into(),Err(e)=>tr(self.language,"Copy: {error}",&[("error",e)])};},
                        KeyCode::KeyX=>{let text=self.editor.selected_text().to_owned();if !text.is_empty(){match self.clipboard().and_then(|c|c.set_text(text).map_err(|e|e.to_string())){Ok(())=>self.editor.delete(),Err(e)=>self.notice=tr(self.language,"Cut: {error}",&[("error",e)])}}},
                        KeyCode::KeyV=>match self.clipboard().and_then(|c|c.get_text().map_err(|e|e.to_string())){Ok(text)=>match self.editor.insert(&text){Ok(())=>self.notice=self.language.text("Pasted into draft; nothing submitted.").into(),Err(e)=>self.notice=e.into()},Err(e)=>self.notice=tr(self.language,"Paste: {error}",&[("error",e)])},
                        _=>{},
                    }
                } else {
                    match code {
                        KeyCode::Enter if !event.repeat=>self.submit(),
                        KeyCode::Escape=>{if self.editor.text().is_empty(){return true;}self.editor.clear();self.notice=self.language.text("Draft cancelled.").into();},
                        KeyCode::ArrowLeft=>self.editor.left(shift),KeyCode::ArrowRight=>self.editor.right(shift),
                        KeyCode::Home=>self.editor.set_cursor(0,shift),KeyCode::End=>self.editor.set_cursor(self.editor.text().len(),shift),
                        KeyCode::Backspace=>self.editor.backspace(),KeyCode::Delete=>self.editor.delete(),
                        KeyCode::ArrowUp=>self.editor.history_previous(),KeyCode::ArrowDown=>self.editor.history_next(),
                        KeyCode::Tab if !event.repeat=>self.complete(profiles,catalogs,cheats,requests),
                        _ if !self.modifiers.alt_key() && !self.modifiers.super_key() && self.editor.preedit().is_empty()=>if let Some(text)=&event.text{if let Err(e)=self.editor.insert(text.as_str()){self.notice=e.into();}},
                        _=>{},
                    }
                }
            },_=>{}
        }
        self.last_draw=Instant::now()-Duration::from_secs(1);false
    }
    pub fn tick(&mut self){
        let mut i=0;while i<self.jobs.len(){
            if !self.jobs[i].1.is_finished(){i+=1;continue;}
            let (command,job)=self.jobs.remove(i);let value=job.join().unwrap_or_else(|_|serde_json::json!({"schema":1,"kind":"ffta-debug-command","ok":false,"outcome":"worker-error","error":"Worker failed","delivery":"unknown"}));
            {
                    self.last_result=serde_json::to_string_pretty(&value).unwrap_or_default();
                    let summary=value.get("outcome").and_then(|v|v.as_str()).unwrap_or(if value["ok"]==false{"error"}else{"ok"});self.line(format!("{command} → {summary}"));
                    if let Some(id)=value.get("id").and_then(|v|v.as_str()){self.line(tr(self.language,"Request: {id}",&[("id",id.into())]));}
                    if let Some(error)=value.get("error").and_then(|v|v.as_str()){self.line(tr(self.language,"Error: {error}",&[("error",error.into())]));}
                    if let Some(status)=value.get("status").filter(|status|status.is_object()){
                        self.line(tr(self.language,"frame {frame} · clock {clock} · paused {paused} · request {request}",&[("frame",status["frame"].to_string()),("clock",status["clock"].to_string()),("paused",status["paused"].to_string()),("request",status["pending_control"]["request_id"].to_string())]));
                    }
                    if let Some(op)=value.get("operation"){
                        self.line(format!("{} · {}",op["id"].as_str().unwrap_or(""),op["title"].as_str().unwrap_or("")));
                        self.line(format!("{}",op["limits"].as_str().unwrap_or("")));
                    }
                    if let Some(operations)=value.get("operations").and_then(|v|v.as_array()){
                        let ids=operations.iter().filter_map(|o|o["id"].as_str()).collect::<Vec<_>>().join(" · ");
                        for chunk in ids.as_bytes().chunks(100){self.line(String::from_utf8_lossy(chunk).into_owned());}
                    }
                    for line in debug_command::observation::result_lines(&value,&command,self.language){self.line(line);}
                    self.notice=self.language.text("Result ready; Ctrl+Shift+C copies JSON.").into();
            }
            self.last_draw=Instant::now()-Duration::from_secs(1);
        }
    }
    pub fn draw_if_due(&mut self)->Result<(),String>{
        if !self.visible||self.last_draw.elapsed()<Duration::from_millis(250){return Ok(());}self.last_draw=Instant::now();
        let size=self.window.inner_size();let(Some(w),Some(h))=(NonZeroU32::new(size.width),NonZeroU32::new(size.height))else{return Ok(())};
        if w.get()>1600||h.get()>1000{return Err("Console supports windows up to 1600×1000".into());}
        let width=w.get() as usize;let height=h.get() as usize;let mut rgba=vec![[0,0,0,255];width*height];
        let draft=format!("> {}",self.editor.display());
        let preedit=tr(self.language,"{preedit}  ·  requests in progress: {count}",&[("preedit",self.editor.preedit().into()),("count",self.jobs.len().to_string())]);
        let keyboard_height=if self.controller.visible(){((height as f32*0.43).clamp(105.0,240.0)) as usize}else{0};
        let count=height.saturating_sub(204+keyboard_height)/17;let results:Vec<String>=self.lines.iter().cloned().collect();let lines:Vec<String>=results.iter().rev().take(count).cloned().collect::<Vec<_>>().into_iter().rev().collect();
        let labels=self.controller.localized_labels(self.language);let keyboard=self.controller.visible().then_some(menu::command_console::Keyboard{labels:&labels,selected:self.controller.selected()});
        let hint=debug_command::completion::hint(self.editor.text(),self.language);
        if self.context_visible {self.context_scroll=menu::command_console::context_lines(self.language,&self.target,&self.pending,&self.parameters,&results,width,height,self.context_scroll).1;menu::command_console::draw_context(self.language,&self.target,&self.pending,&self.parameters,&results,self.context_scroll,&mut rgba,width,height);}
        else {menu::command_console::draw(&menu::command_console::Display{language:self.language,target:&self.target,pending:&self.pending,parameters:&self.parameters,draft:&draft,preedit:&preedit,hint,notice:&self.notice,lines:&lines,pins:&self.pins,keyboard},&mut rgba,width,height);}
        self.surface.resize(w,h).map_err(|e|e.to_string())?;let mut buffer=self.surface.buffer_mut().map_err(|e|e.to_string())?;
        for(out,p)in buffer.iter_mut().zip(rgba){*out=(u32::from(p[0])<<16)|(u32::from(p[1])<<8)|u32::from(p[2]);}buffer.present().map_err(|e|e.to_string())
    }
}
