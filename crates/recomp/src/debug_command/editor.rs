//! Bounded draft editing for the debugger command panel. No execution occurs here.
use std::collections::VecDeque;

const MAX_BYTES: usize = 4096;
const MAX_HISTORY: usize = 64;

#[derive(Default)]
pub(crate) struct Editor {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
    preedit: String,
    history: VecDeque<String>,
    history_index: Option<usize>,
    saved_draft: String,
}

impl Editor {
    pub fn text(&self) -> &str { &self.text }
    pub fn cursor(&self) -> usize { self.cursor }
    pub fn preedit(&self) -> &str { &self.preedit }
    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        self.anchor.filter(|a| *a != self.cursor).map(|a| a.min(self.cursor)..a.max(self.cursor))
    }
    pub fn selected_text(&self) -> &str {
        self.selection().map_or("", |range| &self.text[range])
    }
    /// A bounded viewport keeps the caret and selection visible even for a long draft.
    pub fn display(&self) -> String {
        let start=self.text[..self.cursor].char_indices().rev().nth(59).map_or(0,|(i,_)|i);
        let end=self.text[self.cursor..].char_indices().nth(60).map_or(self.text.len(),|(i,_)|self.cursor+i);
        let selection=self.selection();let mut out=String::new();
        if start>0{out.push('…');}
        for (offset,ch) in self.text[start..end].char_indices(){
            let i=start+offset;
            if selection.as_ref().is_some_and(|r|r.start==i){out.push('[');}
            if self.cursor==i{out.push('│');}
            match ch {'\n'=>out.push('↵'),'\r'=>{},'\t'=>out.push('⇥'),_=>out.push(ch)}
            if selection.as_ref().is_some_and(|r|r.end==i+ch.len_utf8()){out.push(']');}
        }
        if self.cursor==end{out.push('│');}
        if end<self.text.len(){out.push('…');}
        out
    }
    pub fn set_preedit(&mut self, text: &str) {
        self.preedit = text.chars().filter(|c| !c.is_control()).take(256).collect();
    }
    fn clear_history_navigation(&mut self) {
        self.history_index = None;
        self.saved_draft.clear();
    }
    /// Insertion is transactional: oversized or disallowed text changes nothing.
    /// Newlines from paste stay visible in the draft and can never submit as a batch.
    pub fn insert(&mut self, text: &str) -> Result<(), &'static str> {
        if text.chars().any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t')) {
            return Err("Unsupported control character");
        }
        let range = self.selection().unwrap_or(self.cursor..self.cursor);
        if self.text.len() - (range.end - range.start) + text.len() > MAX_BYTES {
            return Err("Draft exceeds 4096 bytes");
        }
        self.text.replace_range(range.clone(), text);
        self.cursor = range.start + text.len();
        self.anchor = None;
        self.preedit.clear();
        self.clear_history_navigation();
        Ok(())
    }
    pub fn replace_range(&mut self,range:std::ops::Range<usize>,text:&str)->Result<(),&'static str>{
        if range.start>range.end||range.end>self.text.len()||!self.text.is_char_boundary(range.start)||!self.text.is_char_boundary(range.end){return Err("Invalid replacement boundary");}
        if text.chars().any(|c|c.is_control() && !matches!(c,'\n'|'\r'|'\t')){return Err("Unsupported control character");}
        if self.text.len()-(range.end-range.start)+text.len()>MAX_BYTES{return Err("Draft exceeds 4096 bytes");}
        self.text.replace_range(range.clone(),text);self.cursor=range.start+text.len();self.anchor=None;self.preedit.clear();self.clear_history_navigation();Ok(())
    }
    pub fn set_cursor(&mut self, cursor: usize, selecting: bool) {
        assert!(cursor <= self.text.len() && self.text.is_char_boundary(cursor));
        if selecting { self.anchor.get_or_insert(self.cursor); } else { self.anchor = None; }
        self.cursor = cursor;
    }
    pub fn left(&mut self, selecting: bool) {
        let next = if !selecting { self.selection().map(|r| r.start) } else { None }
            .unwrap_or_else(|| self.text[..self.cursor].char_indices().next_back().map_or(0, |(i, _)| i));
        self.set_cursor(next, selecting);
    }
    pub fn right(&mut self, selecting: bool) {
        let next = if !selecting { self.selection().map(|r| r.end) } else { None }
            .unwrap_or_else(|| self.text[self.cursor..].chars().next().map_or(self.cursor, |c| self.cursor + c.len_utf8()));
        self.set_cursor(next, selecting);
    }
    pub fn select_all(&mut self) { self.anchor = Some(0); self.cursor = self.text.len(); }
    pub fn backspace(&mut self) {
        if self.selection().is_none() { self.left(true); }
        let _ = self.insert("");
    }
    pub fn delete(&mut self) {
        if self.selection().is_none() { self.right(true); }
        let _ = self.insert("");
    }
    pub fn history_previous(&mut self) {
        if self.history.is_empty() { return; }
        let index = match self.history_index {
            None => { self.saved_draft = self.text.clone(); self.history.len() - 1 },
            Some(index) => index.saturating_sub(1),
        };
        self.history_index = Some(index);
        self.text = self.history[index].clone(); self.cursor = self.text.len(); self.anchor = None;
    }
    pub fn history_next(&mut self) {
        let Some(index) = self.history_index else { return; };
        if index + 1 < self.history.len() {
            self.history_index = Some(index + 1); self.text = self.history[index + 1].clone();
        } else {
            self.history_index = None; self.text = std::mem::take(&mut self.saved_draft);
        }
        self.cursor = self.text.len(); self.anchor = None;
    }
    /// Invoke only after parsing and explicit submission; malformed drafts remain editable.
    pub fn submitted(&mut self) {
        if !self.text.is_empty() && self.history.back() != Some(&self.text) {
            if self.history.len() == MAX_HISTORY { self.history.pop_front(); }
            self.history.push_back(self.text.clone());
        }
        self.text.clear(); self.cursor = 0; self.anchor = None; self.preedit.clear();
        self.clear_history_navigation();
    }
    pub fn clear(&mut self) {
        self.text.clear(); self.cursor = 0; self.anchor = None; self.preedit.clear();
        self.clear_history_navigation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_selection_replacement_and_limits_are_transactional() {
        let mut e=Editor::default(); e.insert("aé猫").unwrap();
        e.left(true); assert_eq!(e.selected_text(),"猫"); e.insert("★").unwrap();
        assert_eq!(e.text(),"aé★"); e.left(false); e.backspace(); assert_eq!(e.text(),"a★");
        e.select_all(); let before=e.text().to_owned();
        assert!(e.insert(&"x".repeat(4097)).is_err()); assert_eq!(e.text(),before);
        assert_eq!(e.selected_text(),before); assert!(e.insert("\0").is_err());
        assert!(e.display().contains('['));assert!(e.display().contains(']'));
        assert!(e.replace_range(1..2,"x").is_err());assert_eq!(e.text(),before);
        e.delete(); assert_eq!(e.text(),"");
    }
    #[test]
    fn history_preserves_unsubmitted_draft_and_multiline_paste_is_never_submitted_implicitly() {
        let mut e=Editor::default(); e.insert("step 1").unwrap();e.submitted();
        e.insert("draft\npause").unwrap();assert_eq!(e.text(),"draft\npause");
        e.history_previous();assert_eq!(e.text(),"step 1");e.history_next();assert_eq!(e.text(),"draft\npause");
        e.set_preedit("候補");assert_eq!(e.preedit(),"候補");e.clear();assert!(e.preedit().is_empty());
        e.history_previous();assert_eq!(e.text(),"step 1");
    }
}
