//! Explicit bounded frontend observation; never injects input or runs a guest step.
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const MAX_EVENTS: usize = 1024;
#[derive(Default)]
pub(crate) struct Trace {
    clock: crate::debug_events::Clock,
    origin: Option<Value>,
    started: Option<Instant>,
    events: Vec<Value>,
    last_poll: Option<Value>,
    last_pad: Option<Value>,
    reason: &'static str,
    collecting: bool,
    frames: usize,
    pub report: Option<String>,
}
impl Trace {
    pub fn enabled(&self) -> bool {
        self.collecting
    }
    pub fn lifecycle_status(&self)->Value {
        json!({"collecting":self.collecting,"reason":self.reason,"completed_guest_frames":self.frames,"last_report":self.report})
    }
    pub fn start(&mut self, origin: Value) -> Result<(), String> {
        if self.origin.is_some() {
            return Err("Save the current UI trace before starting another".into());
        }
        self.origin = Some(origin);
        self.started = Some(Instant::now());
        self.events.clear();
        self.last_poll = None;
        self.last_pad = None;
        self.reason = "collecting";
        self.collecting = true;
        self.frames = 0;
        Ok(())
    }
    pub fn start_shared(&mut self, origin: Value, clock: crate::debug_events::Clock) -> Result<(), String> {
        self.start(origin)?;
        self.clock = clock;
        Ok(())
    }
    #[cfg(test)]
    pub fn event(&mut self, frame: u64, clock: u64, stage: &str, data: Value) {
        if stage == "mapped-poll" {
            if self.last_poll.as_ref() == Some(&data) {
                return;
            }
            self.last_poll = Some(data.clone());
        }
        if stage == "controller-poll" {
            if self.last_pad.as_ref() == Some(&data) {
                return;
            }
            self.last_pad = Some(data.clone());
        }
        let event = self.clock.event(frame, clock, stage, data);
        self.accept(&event);
    }
    pub fn accept(&mut self, event: &crate::debug_events::Event) {
        if !self.collecting {
            return;
        }
        let elapsed = self.started.unwrap().elapsed().as_millis();
        if elapsed >= 30_000 {
            self.freeze("30-second-limit");
            return;
        }
        if self.events.len() >= MAX_EVENTS {
            self.freeze("event-limit");
            return;
        }
        let stage = event.stage.as_str();
        if stage == "guest-frame" {
            if self.frames == 600 {
                self.freeze("frame-limit");
                return;
            }
            self.frames += 1;
        }
        self.events.push(event.value(self.events.len() as u64, elapsed.min(u128::from(u64::MAX)) as u64));
    }
    pub fn freeze(&mut self, reason: &'static str) {
        if self.collecting {
            self.collecting = false;
            self.reason = reason;
        }
    }
    pub fn status(&self) -> Value {
        json!({"collecting":self.collecting,"has_trace":self.origin.is_some(),"events":self.events.len(),"frames":self.frames,"stop_reason":self.reason,"last_report":self.report,"max_events":MAX_EVENTS,"max_frames":600,"max_seconds":30,"event_clock":self.clock.status()})
    }
    pub fn save(
        &mut self,
        root: &Path,
        mut frozen: Value,
        pixels: &[[u8; 4]],
        w: usize,
        h: usize,
    ) -> Result<String, String> {
        capacity(root)?;
        if self.origin.is_none() {
            return Err("Start a UI trace first".into());
        }
        if w == 0 || h == 0 || w > 1600 || h > 1000 || w.checked_mul(h) != Some(pixels.len()) {
            return Err("UI report requires a canvas up to1600x1000".into());
        }
        self.freeze("report-request");
        frozen["schema"] = json!(1);
        frozen["kind"] = json!("ffta-ui-inspection");
        frozen["origin"] = self.origin.clone().unwrap();
        frozen["trace"] = self.status();
        frozen["events"] = json!(self.events);
        frozen["event_coverage"] = crate::debug_events::coverage();
        frozen["scope"] = json!({"host_text":"Actual host-menu font/advance/outline metrics on disposable redraw; original width-stop and canvas clipping unchanged", "guest_text":"Not decoded here; native guest-layer bounds and recognized selectors only", "input":"Observed keyboard/controller tokens, mapped polls, host dispatch and actual guest-frame latch; absent raw hooks remain uncovered", "timeline":"Host sequence/ms separate from guest frame/clock; stopped/truncated prefixes are not a complete replay", "no_input_injection":true,"no_guest_mutation":true});
        let bytes = serde_json::to_vec(&frozen).map_err(|e| e.to_string())?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err("UI JSON exceeds2MiB; trace retained".into());
        }
        let parent = root.join("Data/config/ui");
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        let name = format!(
            "{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos(),
            std::process::id()
        );
        let dir = parent.join(&name);
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        let raw: Vec<_> = pixels.iter().flat_map(|p| p.iter().copied()).collect();
        image::save_buffer_with_format(
            dir.join("menu.png"),
            &raw,
            w as u32,
            h as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .map_err(|e| e.to_string())?;
        let escaped = String::from_utf8(bytes.clone())
            .unwrap()
            .replace('<', "\\u003c");
        fs::write(
            dir.join("index.html"),
            include_str!("debug_ui/report.html").replacen("/*REPORT*/", &escaped, 1),
        )
        .map_err(|e| e.to_string())?;
        fs::write(dir.join("ui.json"), bytes).map_err(|e| e.to_string())?;
        let path = format!("Data/config/ui/{name}/index.html");
        self.origin = None;
        self.started = None;
        self.events.clear();
        self.last_poll = None;
        self.last_pad = None;
        self.report = Some(path.clone());
        Ok(path)
    }
}
pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let base = root.join("Data/config/ui");
    if base.exists() && fs::read_dir(base).map_err(|e| e.to_string())?.count() >= 8 {
        return Err("Eight UI reports already exist; retain/archive before another trace".into());
    }
    Ok(())
}
pub(crate) fn focus(menu: &menu::Menu, w: usize, h: usize) -> Value {
    let f = menu.debug_focus(w, h);
    json!({"page":f.page,"row":f.selected,"label":f.label,"capture":f.capture,"panel":f.panel,"selected_bounds":f.selected_bounds,"first_visible":f.first_visible,
        "dropdown":f.dropdown.map(|d|json!({"label":d.label,"index":d.index,"option":d.option,"panel":d.panel,"first_visible":d.first_visible}))})
}
pub(crate) fn text(capture: menu::diagnostics::TextCapture) -> Value {
    json!({"dropped":capture.dropped,"runs":capture.runs.into_iter().map(|r|json!({
        "text":r.text,"font":r.font,"size":r.size,"origin":r.origin,"max_width":r.max_width,
        "measured_chars":r.measured_chars,"source_limit":r.source_limit,"rendered_chars":r.rendered_chars,
        "advance":r.advance,"full_advance":r.full_advance,"ascent":r.ascent,"descent":r.descent,
        "ink_bounds":r.ink_bounds,"canvas":r.canvas,"width_stopped":r.width_stopped,"canvas_clipped":r.canvas_clipped
    })).collect::<Vec<_>>()})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trace_is_bounded_deduplicated_and_preserves_failed_report() {
        let root = tempfile::tempdir().unwrap();
        let mut trace = Trace::default();
        trace.start(json!({"state":"seed"})).unwrap();
        assert!(trace.start(json!({})).is_err());
        for _ in 0..5 {
            trace.event(1, 2, "mapped-poll", json!({"keys":1023}));
        }
        assert_eq!(trace.events.len(), 1);
        for n in 0..1100 {
            trace.event(1, 2, "keyboard", json!({"n":n}));
        }
        assert_eq!(trace.events.len(), 1024);
        assert_eq!(trace.reason, "event-limit");
        assert!(trace.save(root.path(), json!({}), &[], 1, 1).is_err());
        assert!(trace.origin.is_some());
        let path = trace
            .save(
                root.path(),
                json!({"text":{"runs":[]}}),
                &[[0, 0, 0, 255]],
                1,
                1,
            )
            .unwrap();
        let report: Value = serde_json::from_slice(
            &fs::read(root.path().join(path).with_file_name("ui.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["events"].as_array().unwrap().len(), 1024);
        assert_eq!(report["trace"]["stop_reason"], "event-limit");
        assert!(trace.origin.is_none());
        let parent = root.path().join("Data/config/ui");
        for n in 0..7 {
            fs::create_dir(parent.join(format!("{n}"))).unwrap();
        }
        assert!(capacity(root.path()).is_err());
    }
    #[test]
    fn time_and_frame_limits_keep_explicit_prefix() {
        let mut trace = Trace::default();
        trace.start(json!({})).unwrap();
        for n in 0..601 {
            trace.event(n, 0, "guest-frame", json!({"keys":1023}));
        }
        assert_eq!(trace.frames, 600);
        assert_eq!(trace.reason, "frame-limit");
        trace.origin = None;
        trace.start(json!({})).unwrap();
        trace.started = Some(Instant::now() - std::time::Duration::from_secs(31));
        trace.event(0, 0, "mapped-poll", json!({}));
        assert_eq!(trace.reason, "30-second-limit");
        assert!(trace.events.is_empty());
    }
}
