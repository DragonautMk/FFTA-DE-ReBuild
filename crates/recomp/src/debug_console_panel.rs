//! Optional host window for the read-only event console.
use std::{fs, io::Write, num::NonZeroU32, path::{Path, PathBuf}, sync::Arc, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use winit::{event::{ElementState, MouseScrollDelta, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, ModifiersState, PhysicalKey}, window::Window};
use crate::debug_console::Window as EventWindow;
use input_config::Language;
use menu::i18n::format as tr;

const CATEGORIES: &[&str] = &["all", "input", "selection", "command", "runtime", "guest", "capture", "diagnostic", "unknown"];
const LEVELS: &[&str] = &["all", "info", "error"];

/// Presentation never changes canonical filter tokens, event data or exports.
fn presentation(model:&EventWindow,language:Language,category:&str,level:&str,query:&str,editing:bool,visible:usize,total:usize)->(String,String,String) {
    let status=model.status();
    let summary=tr(language,"Collection: {collection}  ·  view: {view}  ·  poll: {poll}  ·  retained {retained}  ·  evicted {evicted}  ·  oversize {oversize}  ·  budget {budget}  ·  visible {visible}/{total}",&[
        ("collection",language.text(if model.enabled(){"Event collection active"}else{"Event collection off"}).into()),
        ("view",language.text(if model.view_paused(){"Event view paused"}else{"live"}).into()),
        ("poll",language.text(if model.coalesce_polls{"Event polls coalesced"}else{"Event polls uncoalesced"}).into()),
        ("retained",status["retained"].to_string()),("evicted",status["evicted"].to_string()),
        ("oversize",status["oversize"].to_string()),("budget",status["budget_dropped"].to_string()),
        ("visible",visible.to_string()),("total",total.to_string()),
    ]);
    let key=if editing{"Category: {category}  Level: {level}  Search (editing): {query}"}else{"Category: {category}  Level: {level}  Search: {query}"};
    let filter=tr(language,key,&[("category",category.into()),("level",level.into()),("query",query.into())]);
    let range=tr(language,"Current window: {first} → {last}  ·  guest reads aggregated per frame",&[
        ("first",status["first_event_id"].as_str().unwrap_or("—").into()),
        ("last",status["last_event_id"].as_str().unwrap_or("—").into()),
    ]);
    (summary,filter,range)
}

pub(crate) struct Panel {
    window: Arc<Window>,
    visible: bool,
    language: Language,
    focus_handoff: Option<Instant>,
    _context: softbuffer::Context<Arc<Window>>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    last_draw: Instant,
    modifiers: ModifiersState,
    query: String,
    editing: bool,
    category: usize,
    level: usize,
    offset: usize,
    notice: String,
    clipboard: Option<arboard::Clipboard>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn localization_preserves_filter_tokens_and_exported_data() {
        let mut model=EventWindow::default();model.start();model.pause_view(true);
        model.filter.set_categories(["runtime".into()]);model.filter.set_levels(["error".into()]);
        model.filter.set_query("{category}");
        let exported=model.export();
        let (en,filter_en,range_en)=presentation(&model,Language::default(),"runtime","error","{category}",true,0,0);
        let (it,filter_it,range_it)=presentation(&model,Language::parse("it"),"runtime","error","{category}",true,0,0);
        assert!(en.contains("Collection: active")&&en.contains("view: paused"));
        assert!(it.contains("Raccolta: attiva")&&it.contains("vista: in pausa"));
        assert!(filter_en.contains("Category: runtime  Level: error  Search (editing): {category}"));
        assert!(filter_it.contains("Categoria: runtime  Livello: error  Cerca (modifica): {category}"));
        assert!(range_en.starts_with("Current window:")&&range_it.starts_with("Finestra attuale:"));
        assert_eq!(model.export(),exported);
    }
}
impl Panel {
    pub fn open(event_loop: &ActiveEventLoop, model: &mut EventWindow, language:Language) -> Result<Self, String> {
        let attributes=Window::default_attributes().with_title(format!("FFTA — {}",language.text("Event console")))
            .with_inner_size(winit::dpi::LogicalSize::new(900.0, 560.0));
        let window=Arc::new(event_loop.create_window(attributes).map_err(|e|e.to_string())?);
        let context=softbuffer::Context::new(window.clone()).map_err(|e|e.to_string())?;
        let surface=softbuffer::Surface::new(&context,window.clone()).map_err(|e|e.to_string())?;
        model.start();
        Ok(Self{window,visible:true,language,focus_handoff:Some(Instant::now()),_context:context,surface,last_draw:Instant::now()-Duration::from_secs(1),
            modifiers:ModifiersState::empty(),query:String::new(),editing:false,category:0,level:0,offset:0,
            notice:language.text("Collection active; pausing the view does not pause the game.").into(),clipboard:None})
    }
    pub fn owns(&self,id:winit::window::WindowId)->bool {self.window.id()==id}
    pub fn show(&mut self,model:&mut EventWindow,language:Language) { self.language=language; self.window.set_title(&format!("FFTA — {}",language.text("Event console"))); self.visible=true; self.focus_handoff=Some(Instant::now()); model.start(); self.window.set_visible(true); self.window.focus_window(); self.last_draw=Instant::now()-Duration::from_secs(1); }
    pub fn hide(&mut self,model:&mut EventWindow) { self.visible=false; self.focus_handoff=None; model.stop(); self.window.set_visible(false); }
    pub fn is_visible(&self)->bool { self.visible }
    pub fn focus_handoff(&self)->bool { self.focus_handoff.is_some_and(|t|t.elapsed()<Duration::from_secs(2)) }
    fn filter(&self, model:&mut EventWindow) {
        model.filter.set_query(&self.query);
        model.filter.set_categories((self.category!=0).then(||CATEGORIES[self.category].to_string()));
        model.filter.set_levels((self.level!=0).then(||LEVELS[self.level].to_string()));
    }
    fn scroll(&mut self,model:&mut EventWindow,delta:isize) {
        if delta>0 {self.offset=self.offset.saturating_add(delta as usize);} else {self.offset=self.offset.saturating_sub(delta.unsigned_abs());}
        self.offset=self.offset.min(model.visible_count().saturating_sub(1));
        model.autoscroll=self.offset==0;
    }
    fn copy(&mut self,model:&EventWindow)->Result<(),String> {
        let text=serde_json::to_string_pretty(&model.export()).map_err(|e|e.to_string())?;
        if self.clipboard.is_none(){self.clipboard=Some(arboard::Clipboard::new().map_err(|e|e.to_string())?);}
        self.clipboard.as_mut().unwrap().set_text(text).map_err(|e|e.to_string())
    }
    fn save(&self,model:&EventWindow,root:&Path)->Result<PathBuf,String> {
        let bytes=serde_json::to_vec_pretty(&model.export()).map_err(|e|e.to_string())?;
        if bytes.len()>2*1024*1024{return Err(self.language.text("Exported window exceeds 2 MiB").into());}
        let mut dir=root.to_owned();
        for part in ["Data","config","event-console"] {
            dir.push(part);
            match fs::symlink_metadata(&dir) {
                Ok(meta)=>{
                    let linked=meta.file_type().is_symlink();
                    #[cfg(windows)] let linked={use std::os::windows::fs::MetadataExt;linked||meta.file_attributes()&0x400!=0};
                    if linked||!meta.is_dir(){return Err(self.language.text("The destination directory is not an ordinary directory").into());}
                },
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>fs::create_dir(&dir).map_err(|e|e.to_string())?,
                Err(e)=>return Err(e.to_string()),
            }
        }
        if fs::read_dir(&dir).map_err(|e|e.to_string())?.count()>=32{return Err(self.language.text("Limit of 32 exports; archive the previous exports").into());}
        let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos();
        let path=dir.join(format!("events-{stamp}-{}.json",std::process::id()));
        let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e|e.to_string())?;
        if let Err(e)=file.write_all(&bytes).and_then(|_|file.sync_all()) {drop(file);let _=fs::remove_file(&path);return Err(e.to_string());}
        Ok(path)
    }
    /// Returns true when the user requests hiding the window.
    pub fn event(&mut self,event:&WindowEvent,model:&mut EventWindow,root:Option<&Path>)->bool {
        match event {
            WindowEvent::CloseRequested=>return true,
            WindowEvent::Focused(true)=>self.focus_handoff=None,
            WindowEvent::ModifiersChanged(m)=>self.modifiers=m.state(),
            WindowEvent::MouseWheel{delta,..}=>{let v=match delta{MouseScrollDelta::LineDelta(_,y)=>*y as f64,MouseScrollDelta::PixelDelta(p)=>p.y};if v!=0.0{self.scroll(model,if v>0.0{3}else{-3});}},
            WindowEvent::Ime(winit::event::Ime::Commit(text)) if self.editing=>{self.query.extend(text.chars().filter(|c|!c.is_control()).take(256usize.saturating_sub(self.query.chars().count())));self.filter(model);self.offset=0;},
            WindowEvent::KeyboardInput{event,is_synthetic:false,..} if event.state==ElementState::Pressed=>{
                let PhysicalKey::Code(code)=event.physical_key else{return false};
                if self.modifiers.control_key() && code==KeyCode::KeyC {
                    self.notice=match self.copy(model){Ok(())=>self.language.text("Window copied to clipboard").into(),Err(e)=>tr(self.language,"Copy failed: {error}",&[("error",e)])};
                } else if self.modifiers.control_key() && code==KeyCode::KeyS {
                    self.notice=match root.ok_or_else(||self.language.text("Application directory is unavailable").to_string()).and_then(|root|self.save(model,root)){Ok(p)=>tr(self.language,"Saved: {path}",&[("path",p.display().to_string())]),Err(e)=>tr(self.language,"Save failed: {error}",&[("error",e)])};
                } else if self.editing {
                    match code {
                        KeyCode::Escape|KeyCode::Enter=>{self.editing=false;self.window.set_ime_allowed(false);},
                        KeyCode::Backspace=>{self.query.pop();self.filter(model);self.offset=0;},
                        _=>if let Some(text)=&event.text {self.query.extend(text.chars().filter(|c|!c.is_control()).take(256usize.saturating_sub(self.query.chars().count())));self.filter(model);self.offset=0;},
                    }
                } else {
                    match code {
                        KeyCode::Escape=>return true,
                        KeyCode::Slash=>{self.editing=true;self.window.set_ime_allowed(true);},
                        KeyCode::KeyC=>{self.category=(self.category+1)%CATEGORIES.len();self.filter(model);self.offset=0;},
                        KeyCode::KeyL=>{self.level=(self.level+1)%LEVELS.len();self.filter(model);self.offset=0;},
                        KeyCode::KeyP=>model.pause_view(!model.view_paused()),
                        KeyCode::KeyA=>model.coalesce_polls=!model.coalesce_polls,
                        KeyCode::ArrowUp=>self.scroll(model,1),KeyCode::ArrowDown=>self.scroll(model,-1),
                        KeyCode::PageUp=>self.scroll(model,10),KeyCode::PageDown=>self.scroll(model,-10),
                        KeyCode::End=>{self.offset=0;model.autoscroll=true;},_=>{},
                    }
                }
            },_=>{},
        }
        self.last_draw=Instant::now()-Duration::from_secs(1);false
    }
    pub fn draw_if_due(&mut self, model:&EventWindow)->Result<(),String> {
        if !self.visible || self.last_draw.elapsed()<Duration::from_millis(250){return Ok(());}
        self.last_draw=Instant::now();let size=self.window.inner_size();let (Some(w),Some(h))=(NonZeroU32::new(size.width),NonZeroU32::new(size.height))else{return Ok(())};
        if w.get()>1600||h.get()>1000{return Err("Console supports windows up to 1600×1000".into());}
        let width=w.get() as usize;let height=h.get() as usize;let count=height.saturating_sub(147)/17;
        let total=model.visible_count();let end=total.saturating_sub(self.offset.min(total));let start=end.saturating_sub(count);
        let visible=model.visible_page(start,end-start);
        let lines:Vec<String>=visible.iter().map(|v|{
            let detail=serde_json::to_string(&v["data"]).unwrap_or_default();
            let request=v["request_id"].as_str().unwrap_or("-");
            format!("#{:>6} {:<10} {:<24} req:{} {}",v["event_sequence"].as_u64().unwrap_or(0),v["category"].as_str().unwrap_or("?"),v["stage"].as_str().unwrap_or("?"),request,detail.chars().take(160).collect::<String>())
        }).collect();
        let (summary,filter,range)=presentation(model,self.language,CATEGORIES[self.category],LEVELS[self.level],&self.query,self.editing,end.saturating_sub(start),total);
        let mut rgba=vec![[0,0,0,255];width*height];menu::event_console::draw(&menu::event_console::Display{language:self.language,status:&summary,filter:&filter,range:&range,notice:&self.notice,lines:&lines},&mut rgba,width,height);
        self.surface.resize(w,h).map_err(|e|e.to_string())?;let mut buffer=self.surface.buffer_mut().map_err(|e|e.to_string())?;
        for (out,p) in buffer.iter_mut().zip(rgba){*out=(u32::from(p[0])<<16)|(u32::from(p[1])<<8)|u32::from(p[2]);}buffer.present().map_err(|e|e.to_string())
    }
}
