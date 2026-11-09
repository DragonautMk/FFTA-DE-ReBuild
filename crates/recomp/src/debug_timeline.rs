//! Session-local issue context over the existing bounded event collector.
//! Publication order and correlation are observations, never proof of causation.
use serde_json::{json,Value};

pub(crate) struct Timeline {
    enabled:bool,
    events:crate::debug_console::Window,
    recipe:Option<Value>,
    checkpoint:Option<Value>,
}
impl Timeline {
    pub fn new(enabled:bool)->Self {
        let mut events=crate::debug_console::Window::default();if enabled{events.start();}
        Self{enabled,events,recipe:None,checkpoint:None}
    }
    pub fn enabled(&self)->bool{self.enabled}
    pub fn begin_tick(&mut self){self.events.begin_tick();}
    pub fn accept(&mut self,event:&crate::debug_events::Event){
        if self.enabled && matches!(event.stage.as_str(),"host-menu-baseline"|"host-menu-change"|"host-action"|"control-request"|"control-result"|"control-complete"|"runtime-event"|"guest-frame"|"guest-input-latch"|"bookmark"|"error"|"timeline-restored") {self.events.accept(event);}
    }
    pub fn set_recipe(&mut self,recipe:Option<Value>){self.recipe=recipe;self.checkpoint=None;}
    pub fn recipe(&self)->Option<&Value>{self.recipe.as_ref()}
    pub fn set_checkpoint(&mut self,checkpoint:Option<Value>){self.checkpoint=checkpoint;}
    pub fn checkpoint(&self)->Option<&Value>{self.checkpoint.as_ref()}
    pub fn status(&self)->Value{json!({"enabled":self.enabled,"collector":self.events.status(),"selected_recipe":self.recipe.as_ref().map(|v|json!({"name":v["name"],"sha256":v["sha256"]})),"selected_checkpoint":self.checkpoint.as_ref().map(|v|json!({"id":v["id"],"session_run":v["session_run"]}))})}
    pub fn snapshot(&self)->Value {
        let exported=self.events.export();
        json!({"schema":1,"kind":"ffta-issue-timeline","enabled":self.enabled,"collector":exported["collector"],"events":exported["events"],"recipe":self.recipe,"checkpoint":self.checkpoint,
            "coverage":"Selected host menu, control, lifecycle, completed guest-frame, bookmark and error observations only. Sequence gaps, collection start, coalescing, eviction and drops do not prove absence. Native block interiors and uninstrumented guest events are not observed.",
            "causation":"Ordering and matching request IDs are context, not proof that one event caused another.","privacy":"Private local context; event details and labels may contain personal paths or text."})
    }
}

pub(crate) fn valid_label(s:&str)->bool {!s.is_empty() && s.len()<=96 && !s.chars().any(char::is_control)}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_events::Clock;
    #[test]
    fn captures_only_selected_events_with_bookmarks_and_bounded_losses(){
        let clock=Clock::default();let mut timeline=Timeline::new(true);
        timeline.begin_tick();timeline.accept(&clock.event(1,5,"keyboard",json!({"text":"secret"})));
        timeline.accept(&clock.event(1,5,"host-menu-change",json!({"property":"label"})));
        timeline.accept(&clock.event(1,5,"bookmark",json!({"label":"before"})));
        timeline.accept(&clock.event(2,8,"guest-frame",json!({"completed_frame":2})));
        let value=timeline.snapshot();assert_eq!(value["events"].as_array().unwrap().len(),3);assert_eq!(value["events"][1]["data"]["label"],"before");assert!(!value.to_string().contains("secret"));
        for i in 0..1100 {timeline.begin_tick();timeline.accept(&clock.event(i,0,"guest-frame",json!({"i":i})));}
        let value=timeline.snapshot();assert!(value["collector"]["evicted"].as_u64().unwrap()>0);assert!(value["events"].as_array().unwrap().len()<=1024);assert!(serde_json::to_vec(&value).unwrap().len()<300*1024);
        assert!(valid_label("before hit")&&!valid_label("")&&!valid_label("bad\nlabel"));
    }
}
