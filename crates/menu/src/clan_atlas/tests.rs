use super::*;
use crate::{AvConfig, MenuAction, MenuInput};
#[test]
fn shell_is_bounded_and_fixed_for_all_supported_viewports() {
    for (w, h) in [(1280, 720), (1024, 768), (854, 480), (640, 480), (480, 640),(480,360)] {
        let g = geometry(w, h);
        for r in [
            g.header,
            g.sidebar,
            g.domain,
            g.binding,
            g.back,
            g.body,
            g.title,
            g.feedback,
            g.shortcuts,
            g.list,
            g.detail,
        ] {
            assert!(r.x + r.w <= w && r.y + r.h <= h, "{w}x{h} {r:?}");
        }
        for r in crate::atlas_theme::overview_rects(w, h) {
            assert!(
                r.x >= g.body.x
                    && r.y >= g.body.y
                    && r.x + r.w <= g.body.x + g.body.w
                    && r.y + r.h <= g.body.y + g.body.h,
                "overview{w}x{h} {r:?} body{:?}",
                g.body
            );
        }
        let mut menu = Menu::unlocked_for_test(true, true);
        let mut av = AvConfig::default();
        for _ in 0..25 {
            menu.handle(MenuInput::Down, &mut av);
        }
        assert_eq!(geometry(w, h), g);
    }
}
#[test]
fn all_legacy_debug_actions_have_at_least_one_group_and_tabs_retain_focus() {
    for old in crate::rows_of(Page::Debug) {
        assert!(
            (0..8).any(|section| debug_rows(section).contains(old)),
            "orphan {old:?}"
        );
    }
    let mut m = Menu::unlocked_for_test(true, true);
    m.atlas_section(Domain::Debug, 4);
    m.selected = 5;
    m.scroll.set(3);
    m.atlas_section(Domain::Game, 5);
    m.selected = 2;
    m.atlas_section(Domain::Debug, 4);
    assert_eq!((m.page, m.selected, m.scroll.get()), (Page::Debug, 5, 3));
}
#[test]
fn overview_slot_selection_is_readonly_and_overwrite_remains_confirmed() {
    let mut m = Menu::unlocked_for_test(true, true);
    m.set_state_slots(vec![
        crate::StateSlot {
            occupied: true,
            preview: Vec::new(),
            status: "Test fixture".into()
        };
        99
    ]);
    let mut av = AvConfig::default();
    m.selected = 1;
    assert_eq!(m.handle(MenuInput::Confirm, &mut av), None);
    assert_eq!(m.handle(MenuInput::Right, &mut av), None);
    assert_eq!(m.selected_state_slot(), 1);
    assert_eq!(m.handle(MenuInput::Confirm, &mut av), None);
    assert_eq!(m.selected_state_slot(), 2);
    m.selected = 2;
    assert_eq!(m.handle(MenuInput::Confirm, &mut av), None);
    assert_eq!(m.page, Page::ReplaceState);
    assert_eq!(m.selected, 1);
    m.selected = 0;
    assert_eq!(
        m.handle(MenuInput::Confirm, &mut av),
        Some(MenuAction::SaveState {
            slot: 2,
            overwrite: true
        })
    );
    m.reset();
    assert_eq!(m.selected_state_slot(), 2);
}
#[test]
fn global_chords_preserve_preferences_and_work_from_debug() {
    let mut m = Menu::unlocked_for_test(true, true);
    m.atlas_section(Domain::Debug, 7);
    let mut av = AvConfig::default();
    let volume = av.audio_volume;
    let speed = av.ff_speed;
    assert_eq!(
        m.global_shortcut(crate::shortcuts::Shortcut::Mute, &mut av),
        Some(MenuAction::Changed(crate::Changed::Audio))
    );
    assert!(av.audio_mute);
    assert_eq!(av.audio_volume, volume);
    assert_eq!(
        m.global_shortcut(crate::shortcuts::Shortcut::FastToggle, &mut av),
        Some(MenuAction::ToggleFastForward)
    );
    assert_eq!(av.ff_speed, speed);
}
#[test]
fn utc_dates_are_real_calendar_values_and_unavailable_is_not_fabricated() {
    assert_eq!(timestamp_utc(0).as_deref(), Some("1970-01-01 00:00 UTC"));
    assert_eq!(
        timestamp_utc(951782400).as_deref(),
        Some("2000-02-29 00:00 UTC")
    );
    assert_eq!(
        timestamp_utc(1709164800).as_deref(),
        Some("2024-02-29 00:00 UTC")
    );
    assert_eq!(timestamp_utc(u64::MAX), None);
}
#[test]
#[ignore = "native atlas image export; explicit local output directory, Wine only"]
fn render_native_atlas_viewports() {
    let root = std::path::PathBuf::from(std::env::var_os("FFTA_ATLAS_RENDER_DIR").unwrap());
    assert!(!root.exists());
    std::fs::create_dir_all(&root).unwrap();
    let mut menu = Menu::unlocked_for_test(true, true);
    let av = AvConfig::default();
    let input = crate::InputConfig::default();
    for (w, h) in [(1280, 720), (1024, 768), (854, 480), (640, 480),(480,640)] {
        for (label, domain, section) in [
            ("overview", Domain::Game, 0),
            ("video", Domain::Game, 5),
            ("controls",Domain::Game,6),
            ("saves",Domain::Game,1),
            ("debug-v2",Domain::Debug,0),
            ("catalog",Domain::Debug,1),
            ("selector",Domain::Debug,1),
            ("tool",Domain::Debug,5),
            ("debug-traces", Domain::Debug, 4),
        ] {
            menu.atlas_section(domain, section);
            if label=="saves" {menu.open(Page::SaveStates);}
            if label=="debug-v2" {menu.atlas_reference_tab();}
            if label=="catalog" {menu.set_catalog(vec![crate::CatalogEntry{key:"Synthetic-DS".into(),name:"Synthetic catalog fixture".into(),region:"EUR".into(),..Default::default()}],"EUR");}
            if label=="selector" {menu.open_content("Synthetic content fixture".into(),(0..20).map(|i|crate::ContentEntry{id:format!("synthetic-{i}"),name:format!("Synthetic entry {i}"),description:"Local rendering fixture, no operation".into(),..Default::default()}).collect(),String::new(),1);}
            if label=="tool" {menu.open_tool(crate::DebugTool::Inspect,vec![crate::ToolField{key:"address".into(),label:"Address".into(),value:"02000000".into(),help:"Synthetic read-only rendering fixture".into(),kind:crate::ToolFieldKind::Text}],String::new());}
            let mut rgba = vec![[0, 0, 0, 255]; w * h];
            menu.draw_inputs(&av, &input, &mut rgba, w, h);
            let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
            for p in rgba {
                ppm.extend_from_slice(&p[..3]);
            }
            std::fs::write(root.join(format!("{label}-{w}x{h}.ppm")), ppm).unwrap();
        }
    }
}

#[test]
fn columns_and_readonly_reference_lists_keep_shared_pointer_rectangles() {
    let mut m=Menu::unlocked_for_test(true,true);let mut av=AvConfig::default();
    for (w,h)in [(1280,720),(1024,768),(854,480),(640,480),(480,640)] {
        m.atlas_section(Domain::Game,5);
        for index in 0..m.rows().len(){
            m.selected=index;
            let r=row_rect(&m,index,w,h).unwrap();
            assert_eq!(m.row_at(r.x+r.w/2,r.y+r.h/2,w,h),Some(index));
            let body=geometry(w,h).body;assert!(r.y+r.h<=body.y+body.h);
        }
        for count in [3,9,28] {
            m.atlas_section(Domain::Debug,0);
            m.set_operation_references((0..count).map(|i|OperationReference{id:format!("fixture-{i}"),title:"Synthetic readonly fixture".into(),section:1,details:"No executable action".into()}).collect());
            m.open(Page::CliReference);
            let original=av.serialize();let frame=geometry(w,h);
            for i in 0..count {
                m.atlas_reference_select(i);let focus=m.debug_focus(w,h);
                let r=focus.selected_bounds.unwrap();m.hover(r[0]+5,r[1]+5,w,h);
                assert_eq!(m.atlas_reference_cursor(),i);
                assert_eq!(m.handle(MenuInput::Confirm,&mut av),None);
                assert_eq!(geometry(w,h),frame);
            }
            assert_eq!(av.serialize(),original);
        }
    }
    m.atlas_section(Domain::Debug,0);m.atlas_reference_tab();
    let tasks=super::references::planned();assert_eq!(tasks.len(),28);
    assert_eq!(tasks.iter().filter(|t|t.done).count(),12);
    assert_eq!(tasks.first().unwrap().id,"DBG2-01");assert_eq!(tasks.last().unwrap().id,"DBG2-28");
    assert_eq!(m.atlas_reference_items().len(),28);
    assert_eq!(m.handle(MenuInput::Confirm,&mut av),None);
    m.handle(MenuInput::Cancel,&mut av);assert!(!m.atlas.planned);
}

#[test]
fn planned_reference_follows_the_checklist_and_localizes_its_status() {
    let mut m=Menu::unlocked_for_test(true,true);
    m.atlas_section(Domain::Debug,0);m.atlas_reference_tab();
    let en=m.atlas_reference_items();
    assert!(en[9].0.starts_with("DBG2-10 · Runtime and tool events"));
    assert!(en[9].1.contains("Completed in the current checklist"));
    assert!(en[12].1.contains("Planned · unavailable"));
    assert!(en[12].1.contains("Checklist reference; this view does not execute operations."));
    m.language.set(input_config::Language::parse("it"));
    let it=m.atlas_reference_items();
    assert!(it[9].1.contains("Completata nella checklist corrente"));
    assert!(it[12].1.contains("Pianificata · non disponibile"));
    assert_eq!(en[12].0,it[12].0);
}

#[test]
fn switching_sections_preserves_subform_parent_and_scroll() {
    let mut m=Menu::unlocked_for_test(true,true);let mut av=AvConfig::default();
    m.atlas_section(Domain::Game,1);m.selected=0;m.scroll.set(2);
    m.handle(MenuInput::Confirm,&mut av);assert_eq!(m.page,Page::SaveStates);
    m.atlas_section(Domain::Debug,0);m.atlas_section(Domain::Game,1);
    assert_eq!(m.page,Page::SaveStates);
    m.handle(MenuInput::Cancel,&mut av);assert_eq!(m.page,Page::Saves);
    assert_eq!(m.selected,0);assert_eq!(m.scroll.get(),2);
}

#[test]
fn dropdown_owns_focus_and_reference_detail_scroll_stops_at_its_limit() {
    let mut m=Menu::unlocked_for_test(true,true);let mut av=AvConfig::default();
    m.atlas_section(Domain::Game,5);m.selected=2;
    m.handle(MenuInput::Confirm,&mut av);assert!(m.dropdown.is_some());
    let g=geometry(854,480);m.hover(g.nav[0].x+5,g.nav[0].y+5,854,480);
    assert!(!m.atlas.sidebar_focus);m.handle(MenuInput::Cancel,&mut av);assert!(m.dropdown.is_none());
    m.atlas_section(Domain::Debug,0);m.atlas_reference_tab();
    let mut px=vec![[0,0,0,255];854*480];m.draw_inputs(&av,&crate::InputConfig::default(),&mut px,854,480);
    let limit=m.atlas.detail_limit.get();assert!(limit>0);
    for _ in 0..100{m.handle(MenuInput::Right,&mut av);}
    assert_eq!(m.atlas.detail_scroll,limit);
    m.handle(MenuInput::Left,&mut av);assert_eq!(m.atlas.detail_scroll,limit.saturating_sub(3));
}

#[test]
fn information_keeps_full_text_and_returns_without_changing_selection_or_config() {
    let mut m=Menu::unlocked_for_test(true,true);let av=AvConfig::default();let mut cfg=crate::InputConfig::default();
    cfg.menu_key="LeftControl+LeftShift+LeftAlt+F10".into();
    m.atlas_section(Domain::Game,5);m.selected=7;m.scroll.set(3);m.atlas.sidebar_focus=true;
    let message=format!("{}FINE DETTAGLIO","Synthetic long validation message: ".repeat(600));m.diagnosis_result(message.clone());
    let before=(m.page,m.selected,m.scroll.get(),cfg.serialize(),av.serialize());
    m.information(&av,&cfg);assert_eq!(m.page,Page::Information);assert!(m.atlas.information.contains(&message));assert!(m.atlas.information.contains(&cfg.menu_key));
    let mut pixels=vec![[0,0,0,255];640*480];m.draw_inputs(&av,&cfg,&mut pixels,640,480);assert!(m.atlas.detail_limit.get()>0);
    let mut av=av;for _ in 0..1000{assert_eq!(m.handle(MenuInput::Down,&mut av),None);}
    assert_eq!(m.atlas.detail_scroll,m.atlas.detail_limit.get());
    let capture=crate::diagnostics::capture(||m.draw_inputs(&av,&cfg,&mut pixels,640,480)).unwrap();
    assert!(capture.runs.iter().map(|r|r.text.as_str()).collect::<String>().contains("FINE DETTAGLIO"));
    assert_eq!(m.handle(MenuInput::Cancel,&mut av),None);
    assert_eq!((m.page,m.selected,m.scroll.get(),cfg.serialize(),av.serialize()),before);assert!(m.atlas.sidebar_focus);
    m.atlas.sidebar_focus=false;m.capture=Some(crate::Bind::Menu);m.information(&av,&cfg);assert_eq!(m.page,Page::Video);
    m.capture=None;m.open_tool(crate::DebugTool::Inspect,Vec::new(),String::new());m.tool.busy=true;
    m.information(&av,&cfg);assert_eq!(m.page,Page::DebugTool,"An information overlay must not stale an in-flight tool result");
    m.tool.busy=false;m.information(&av,&cfg);assert_eq!(m.page,Page::Information);m.information(&av,&cfg);assert_eq!(m.page,Page::DebugTool);
}

#[test]
fn real_atlas_geometry_and_fonts_scale_from_hd_to_4k() {
    let hd=geometry(1920,1080);let uhd=geometry(3840,2160);
    assert!(uhd.scale_milli.abs_diff(2*hd.scale_milli)<=2);
    for(a,b)in [(hd.sidebar,uhd.sidebar),(hd.body,uhd.body),(hd.feedback,uhd.feedback),(hd.information,uhd.information)]{
        assert!(b.x.abs_diff(2*a.x)<=8 && b.y.abs_diff(2*a.y)<=8);
        assert!(b.w.abs_diff(2*a.w)<=10 && b.h.abs_diff(2*a.h)<=10);
    }
    let m=Menu::unlocked_for_test(true,true);let a=list_layout(&m,1920,1080);let b=list_layout(&m,3840,2160);assert!(b.line_h.abs_diff(2*a.line_h)<=1);
}

#[test]
fn nested_selectors_and_catalog_keep_readable_bounded_shared_rectangles() {
    for(w,h)in[(1280,720),(1024,768),(854,480),(640,480),(480,640),(480,360)]{
        let body=geometry(w,h).body;let g=crate::content_selector::geometry(body.w,body.h);
        assert!(13.*g.scale>=10.5);
        for r in [g.panel,g.search,g.list,g.details].into_iter().chain(g.controls).chain(g.actions){assert!(r[0]+r[2]<=body.w&&r[1]+r[3]<=body.h,"{w}x{h} {r:?} {body:?}");}
        let mut m=Menu::unlocked_for_test(true,true);m.atlas_section(Domain::Debug,1);m.open(Page::DebugCatalog);
        for index in 0..m.rows().len(){let r=row_rect(&m,index,w,h).unwrap();assert!(r.x+r.w<=body.x+body.w&&r.y+r.h<=body.y+body.h);assert_eq!(m.row_at(r.x+r.w/2,r.y+r.h/2,w,h),Some(index));}
    }
}
