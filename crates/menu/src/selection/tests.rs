use super::*;
use std::collections::BTreeMap;
fn state(menu:&Menu,av:&AvConfig,cfg:&InputConfig)->BTreeMap<String,String>{menu.selection_state(av,cfg).into_iter().collect()}

#[test]
fn input_tab_and_information_diagnostics_follow_menu_language_without_changing_ids() {
    let mut menu = Menu::unlocked_for_test(true, false);
    let cfg = InputConfig::default();
    menu.open(Page::Inputs);
    for (language, labels, prefix, details) in [
        (Language::default(), ["KEYBOARD", "GAMEPAD", "MOUSE & KEYBOARD"], "Device", "Details · read only"),
        (Language::parse("it"), ["TASTIERA", "GAMEPAD", "MOUSE E TASTIERA"], "Dispositivo", "Dettagli · sola lettura"),
    ] {
        menu.language.set(language);
        let av = AvConfig { menu_language: language, ..AvConfig::default() };
        for (index, label) in labels.iter().enumerate() {
            menu.input_tabs_focus = Some(index as u8);
            let observed = state(&menu, &av, &cfg);
            assert_eq!(observed["item_id"], format!("input-tab/{index}"));
            assert_eq!(observed["label"], *label);
            let focus = menu.debug_focus(854, 480);
            assert_eq!(focus.selected, index);
            assert_eq!(focus.label, format!("{prefix}: {label}"));
        }
        menu.input_tabs_focus = None;
        menu.open(Page::Information);
        assert_eq!(menu.debug_focus(854, 480).label, details);
        menu.open(Page::Inputs);
    }
}

#[test]
fn dropdown_selection_value_and_cancel_are_semantic_and_observation_is_readonly(){
    let mut menu=Menu::unlocked_for_test(true,false);let mut av=AvConfig::default();let cfg=InputConfig::default();
    menu.atlas_section(clan_atlas::Domain::Debug,1);menu.selected=0;
    let initial=state(&menu,&av,&cfg);assert_eq!(initial["item_id"],"Debug/ExtendedSaves");assert_eq!(initial["value"],"OFF");
    let config_before=cfg.serialize();let av_before=av.serialize();
    for _ in 0..10{assert_eq!(state(&menu,&av,&cfg),initial);}
    assert_eq!(cfg.serialize(),config_before);assert_eq!(av.serialize(),av_before);
    assert!(menu.handle(MenuInput::Confirm,&mut av).is_none());
    assert_eq!(state(&menu,&av,&cfg)["dropdown.id"],"ExtendedSaves");
    menu.handle(MenuInput::Right,&mut av);assert_eq!(state(&menu,&av,&cfg)["dropdown.value"],"ON");
    menu.handle(MenuInput::Cancel,&mut av);assert_eq!(state(&menu,&av,&cfg),initial);
    menu.handle(MenuInput::Confirm,&mut av);menu.handle(MenuInput::Right,&mut av);
    assert_eq!(menu.handle(MenuInput::Confirm,&mut av),Some(MenuAction::Changed(Changed::Debug)));
    let current=state(&menu,&av,&cfg);assert_eq!(current["value"],"ON");assert!(!current.contains_key("dropdown.id"));
}

#[test]
fn duplicate_content_labels_keep_distinct_ids_and_repeated_hover_does_not_change_state(){
    let mut menu=Menu::unlocked_for_test(true,false);let av=AvConfig::default();let cfg=InputConfig::default();
    let entry=|id:&str|ContentEntry{id:id.into(),name:"same label".into(),kind:"snapshot".into(),..Default::default()};
    menu.open_content("Synthetic".into(),vec![entry("content-a"),entry("content-b")],String::new(),1);
    let body=clan_atlas::geometry(940,620).body;let mut g=content_selector::geometry(body.w,body.h);g.list[0]+=body.x;g.list[1]+=body.y;
    menu.hover(g.list[0]+5,g.list[1]+5,940,620);let a=state(&menu,&av,&cfg);
    menu.hover(g.list[0]+5,g.list[1]+g.row+5,940,620);let b=state(&menu,&av,&cfg);
    assert_ne!(a["item_id"],b["item_id"]);assert_eq!(a["highlighted_id"],"content-a");assert_eq!(b["highlighted_id"],"content-b");
    for _ in 0..30{menu.hover(g.list[0]+5,g.list[1]+g.row+5,940,620);assert_eq!(state(&menu,&av,&cfg),b);}
    assert!(menu.content.confirmed.is_none());
}

#[test]
fn tool_editor_reports_draft_preedit_commit_cancel_and_field_identity(){
    let mut menu=Menu::unlocked_for_test(true,false);let av=AvConfig::default();let cfg=InputConfig::default();
    menu.open_tool(DebugTool::Inspect,vec![ToolField{key:"labels".into(),label:"Label".into(),value:"old".into(),help:String::new(),kind:ToolFieldKind::Text}],String::new());
    let original=state(&menu,&av,&cfg);assert_eq!(original["item_id"],"tool/Inspect/field/labels");
    menu.content_start_search();menu.content_text(" è");menu.content_preedit("候補");
    let editing=state(&menu,&av,&cfg);assert_eq!(editing["editor.preedit"],"候補");assert_eq!(editing["field.labels"],"old");assert!(editing["editor.query"].ends_with(" è"));
    menu.content_finish_search(true);assert_eq!(state(&menu,&av,&cfg),original);
    menu.content_start_search();menu.content_text(" è");menu.content_finish_search(false);
    let accepted=state(&menu,&av,&cfg);assert_eq!(accepted["field.labels"],"old è");assert_eq!(accepted["item_id"],original["item_id"]);assert!(!accepted.contains_key("editor.preedit"));
}

#[test]
fn current_binding_value_and_capture_lifecycle_use_config_without_draw_side_effects(){
    let mut menu=Menu::unlocked_for_test(true,false);let av=AvConfig::default();let mut cfg=InputConfig::default();
    menu.open(Page::Inputs);menu.selected=1;
    let row=rows_of(menu.page)[menu.selected];assert!(matches!(row,Row::Key(_)));
    let before=state(&menu,&av,&cfg);if let Row::Key(button)=row{cfg.keys[button.index()]="F4".into();}
    let after=state(&menu,&av,&cfg);assert_ne!(before["value"],after["value"]);assert_eq!(after["value"],"F4");
    menu.capture=Some(Bind::Menu);assert_eq!(state(&menu,&av,&cfg)["capture"],"Menu");
    menu.end_capture();assert_eq!(state(&menu,&av,&cfg)["capture"],"");
}

#[test]
fn actual_page_feedback_and_slot_availability_are_observed_without_mutating_slots(){
    let mut menu=Menu::unlocked_for_test(true,false);let av=AvConfig::default();let cfg=InputConfig::default();
    menu.open(Page::SaveStates);let before=state(&menu,&av,&cfg);assert_eq!(before["state_slot.occupied"],"false");
    menu.state_result("Refused: no state".into());menu.set_state_undo(Some("Undo save".into()));
    let after=state(&menu,&av,&cfg);assert_eq!(after["feedback"],"Refused: no state");assert_eq!(after["state_undo"],"Undo save");assert_eq!(after["state_slot.occupied"],before["state_slot.occupied"]);
    menu.open(Page::SaveBackups);menu.backup_result("Refused: stale generation".into());assert_eq!(state(&menu,&av,&cfg)["feedback"],"Refused: stale generation");
    menu.open(Page::GameSaves);menu.game_save_result("Refused: unavailable".into());assert_eq!(state(&menu,&av,&cfg)["feedback"],"Refused: unavailable");
    menu.set_catalog(vec![],"All");menu.catalog_result("Refused: empty catalog".into());assert_eq!(state(&menu,&av,&cfg)["catalog.feedback"],"Refused: empty catalog");
}
