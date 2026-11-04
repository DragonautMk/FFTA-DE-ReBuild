//! Rendering for the host debugger command panel.
use crate::canvas::Canvas;

pub struct Display<'a> {
    pub language: input_config::Language,
    pub target: &'a str,
    pub pending: &'a str,
    pub parameters: &'a str,
    pub draft: &'a str,
    pub preedit: &'a str,
    pub hint: &'a str,
    pub notice: &'a str,
    pub lines: &'a [String],
    pub keyboard: Option<Keyboard<'a>>,
}
pub struct Keyboard<'a> {pub labels:&'a [String],pub selected:usize}

/// Wrapped lines and the clamped top line of the read-only context view.
pub fn context_lines(target:&str,pending:&str,parameters:&str,width:usize,height:usize,scroll:usize)->(Vec<String>,usize,usize){
    let available=(width as f32-24.0).max(1.0);
    let mut lines=Vec::new();
    for text in [target,pending,parameters] {lines.extend(Canvas::wrap_text(text,14.0,available));lines.push(String::new());}
    let visible=height.saturating_sub(80)/19;
    let top=scroll.min(lines.len().saturating_sub(visible.max(1)));
    (lines,top,visible)
}

pub fn draw_context(language:input_config::Language,target:&str,pending:&str,parameters:&str,scroll:usize,pixels:&mut [[u8;4]],width:usize,height:usize){
    if width==0 || height==0 || width.checked_mul(height)!=Some(pixels.len()){return;}
    pixels.fill([15,20,27,255]);
    let (lines,top,visible)=context_lines(target,pending,parameters,width,height,scroll);
    let mut canvas=Canvas::new(pixels,width,height);let available=(width as f32-24.0).max(0.0);
    canvas.text(12.0,10.0,language.text("Command context"),[232,224,195],20.0,true,available);
    for (i,line) in lines.iter().skip(top).take(visible).enumerate(){canvas.text(12.0,38.0+i as f32*19.0,line,[218,226,226],14.0,false,available);}
    canvas.text(12.0,(height as f32-30.0).max(0.0),language.text("Up/Down or Page Up/Down: scroll · F2/Esc: return"),[165,183,187],12.0,false,available);
    canvas.finish(pixels);
}

fn wrap_hint(text:&str,capacity:usize)->[String;2]{
    let mut lines=[String::new(),String::new()];let mut row=0;
    for word in text.split_whitespace(){
        let needed=usize::from(!lines[row].is_empty())+word.chars().count();
        if lines[row].chars().count()+needed>capacity && row==0 && !lines[0].is_empty(){row=1;}
        if !lines[row].is_empty(){lines[row].push(' ');}
        lines[row].push_str(word);
    }
    if lines[1].chars().count()>capacity{lines[1]=format!("{}…",lines[1].chars().take(capacity.saturating_sub(1)).collect::<String>());}
    lines
}

pub fn draw(view: &Display<'_>, pixels: &mut [[u8; 4]], width: usize, height: usize) {
    if width == 0 || height == 0 || width.checked_mul(height) != Some(pixels.len()) { return; }
    pixels.fill([15, 20, 27, 255]);
    let mut canvas=Canvas::new(pixels,width,height);
    let available=(width as f32-24.0).max(0.0);
    canvas.text(12.0,10.0,view.language.text("Command console"),[232,224,195],20.0,true,available);
    canvas.text(12.0,32.0,view.target,[181,204,205],12.0,false,available);
    canvas.text(12.0,48.0,view.pending,[218,226,226],12.0,false,available);
    canvas.text(12.0,64.0,view.parameters,[165,183,187],12.0,false,available);
    canvas.text_with_fallback(12.0,82.0,view.draft,[231,210,147],14.0,available);
    canvas.text_with_fallback(12.0,99.0,view.preedit,[181,204,205],12.0,available);
    let hint=wrap_hint(view.hint,((available/7.0) as usize).max(20));
    canvas.text(12.0,115.0,&hint[0],[165,183,187],12.0,false,available);
    canvas.text(12.0,130.0,&hint[1],[165,183,187],12.0,false,available);
    let keyboard_height=if view.keyboard.is_some(){(height as f32*0.43).clamp(105.0,240.0)}else{0.0};
    let keyboard_top=(height as f32-55.0-keyboard_height).max(105.0);
    let bottom=if view.keyboard.is_some(){keyboard_top-4.0}else{(height as f32-52.0).max(105.0)};
    for (i,line) in view.lines.iter().enumerate() {
        let y=145.0+i as f32*17.0;if y+16.0>bottom{break;}
        canvas.text(12.0,y,line,[218,226,226],13.0,false,available);
    }
    if let Some(keyboard)=&view.keyboard {
        let cell_w=available/7.0;let cell_h=keyboard_height/7.0;
        let font=(cell_h*0.46).clamp(8.0,14.0);
        for (i,label) in keyboard.labels.iter().take(49).enumerate(){
            let x=12.0+(i%7) as f32*cell_w;let y=keyboard_top+(i/7) as f32*cell_h;
            let selected=i==keyboard.selected;
            let fill=if selected{[160,117,54]}else{[34,45,54]};
            canvas.rect(x+1.0,y+1.0,(cell_w-3.0).max(0.0),(cell_h-3.0).max(0.0),3.0,fill,fill,if selected{[241,210,142]}else{[82,99,105]});
            canvas.text(x+5.0,y+(cell_h-font)*0.45,label,if selected{[20,22,22]}else{[218,226,226]},font,selected,(cell_w-10.0).max(0.0));
        }
    }
    canvas.text(12.0,(height as f32-48.0).max(0.0),view.notice,[240,187,130],12.0,false,available);
    canvas.text(12.0,(height as f32-30.0).max(0.0),view.language.text(if view.keyboard.is_some(){"Pad: directions / confirm · Back deletes · Send submits"}else{"Enter executes · ↑↓ history · Tab completes · Esc clears/closes"}),[165,183,187],12.0,false,available);
    canvas.text(12.0,(height as f32-15.0).max(0.0),view.language.text("F2: context · Ctrl+A/C/X/V edit · Ctrl+Shift+C copies result"),[165,183,187],12.0,false,available);
    canvas.finish(pixels);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn narrow_context_wraps_long_tokens_and_scrolls_without_losing_text(){
        let target=format!("Session {} | ROM {} | run example", "session-with-a-long-name", "a".repeat(64));
        let parameters=format!("Parameters: select {}", "b".repeat(72));
        let (lines,top,visible)=context_lines(&target,"ID 123 | in progress",&parameters,480,100,usize::MAX);
        assert!(top>0 && visible>0);
        let joined=lines.join("");assert!(joined.contains(&target)&&joined.contains(&parameters));
        assert!(top+visible>=lines.len());
        let mut pixels=vec![[0;4];480*360];draw_context(input_config::Language::default(),&target,"ID 123 | in progress",&parameters,0,&mut pixels,480,360);
        assert!(pixels.iter().any(|p|p[0]>100));
    }
}
