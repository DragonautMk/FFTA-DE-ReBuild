use super::*;

#[test]
fn cheat_action_labels_use_english_keys_and_preserve_italian_presentation() {
    let mut selector = Selector::default();
    selector.cheat_mode = true;
    let italian = input_config::Language::parse("it");
    let expected = [
        "Dettagli", "Preferito", "Attiva...", "Ricarica",
        "Importa catalogo...", "Disattiva tutti", "Indietro", "Mostra preferiti",
    ];
    for (index, (&key, &translation)) in CHEAT_ACTIONS.iter().zip(&expected).enumerate() {
        selector.focus = index + 6;
        assert_eq!(selector.focus_label(), key);
        assert_eq!(input_config::Language::default().text(key), key);
        assert_eq!(italian.text(key), translation);
    }
}

#[test]
fn selector_instructions_and_footer_follow_language_without_replacing_values() {
    let mut selector = Selector::default();
    selector.set("Synthetic".into(), vec![ContentEntry {
        id: "x{id}".into(), name: "Example".into(), ..Default::default()
    }], "{count}".into(), 0);
    selector.confirmed = Some("x{id}".into());
    selector.busy = true;
    let capture = |selector: &Selector, language| {
        let mut pixels = vec![[0, 0, 0, 255]; 940 * 620];
        diagnostics::capture(|| draw(selector, &mut pixels, 940, 620, language))
            .unwrap().runs.into_iter().map(|run| run.text).collect::<Vec<_>>()
    };
    for (language, footer, filter, title, hint) in [
        (input_config::Language::default(), "1 items / working | {count} | Selected x{id}",
         "Filter choice: arrows · OK applies · Cancel discards", "CHEAT DETAILS · READ ONLY", "Up/Down: scroll · OK/Cancel: return"),
        (input_config::Language::parse("it"), "1 elementi / in corso | {count} | Selezionato x{id}",
         "Scelta filtro: frecce · OK applica · Annulla scarta", "DETTAGLI CHEAT · SOLA LETTURA", "Su/Giù: scorri · OK/Annulla: torna"),
    ] {
        assert!(capture(&selector, language).iter().any(|text| text == footer));
        selector.focus = 1;
        selector.choice_edit = Some(0);
        assert!(capture(&selector, language).iter().any(|text| text == filter));
        selector.choice_edit = None;
        selector.cheat_mode = true;
        selector.detail_mode = true;
        let details = capture(&selector, language);
        assert!(details.iter().any(|text| text == title));
        assert!(details.iter().any(|text| text == hint));
        selector.cheat_mode = false;
        selector.detail_mode = false;
    }
}

#[test]
fn cheat_view_keeps_all_rows_and_never_dispatches_selection(){
    let mut s=Selector::default();s.cheat_mode=true;
    s.set("Cheat".into(),(0..600).map(|i|ContentEntry{id:format!("CH-{i:04}"),name:format!("CH-{i:04}"),available:true,..Default::default()}).collect(),String::new(),0);
    assert_eq!(s.entries.len(),600);s.cursor=599;s.focus=6;assert!(s.handle(MenuInput::Confirm).is_none());assert!(s.detail_mode);assert!(s.confirmed.is_none());
    s.entries[s.visible[599]].reason="Unknown ROM; decoder unsupported".into();s.entries[s.visible[599]].description="SYNTHETIC CODE".into();let detail=s.cheat_details(&crate::i18n::Locale(input_config::Language::default()));assert!(detail.find("decoder unsupported").unwrap()<detail.find("SYNTHETIC CODE").unwrap());
    let g=geometry(627,218);let hidden=g.actions[4];assert!(s.click(hidden[0]+2,hidden[1]+2,627,218,true).is_none());assert!(s.detail_mode);
    assert_eq!(s.focus(627,218).selected_bounds,Some(g.panel));
    s.handle(MenuInput::Cancel);s.focus=10;assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatChoose));s.focus=9;assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatReload));
    s.set("Cheat".into(),Vec::new(),String::new(),0);s.focus=6;assert!(s.handle(MenuInput::Confirm).is_none());
    assert!(s.message.contains("Import catalog"));
    s.focus=11;assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::DisarmAll)));
    assert!(crate::i18n::Locale(input_config::Language::parse("it")).tr(&s.message).contains("Importa catalogo"));
}

#[test]
fn management_requires_explicit_source_and_dispatches_separate_requests(){
    let mut s=Selector::default();s.cheat_mode=true;s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),name:"Example".into(),..Default::default()}],String::new(),0);
    s.focus=8;assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatManage));
    s.open_cheat_management(vec![super::super::CheatSource{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"manual-once-subset".into(),parameters:vec![]}],vec![]);
    assert!(s.cheat_selected_source().is_none());s.handle(MenuInput::Right);assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::Prepare)));assert!(s.cheat_selected_source().is_none());
    s.handle(MenuInput::Left);s.handle(MenuInput::Confirm);assert_eq!(s.cheat_selected_source().unwrap().index,0);
    s.handle(MenuInput::Right);assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::Prepare)));
    s.cheat_management_feedback("Review".into(),Some("plan and footprint".into()));assert!(s.detail_mode);assert!(!s.cheat_preview_reviewed());s.handle(MenuInput::Cancel);assert!(!s.detail_mode);assert!(s.cheat_preview_reviewed());
    assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::Arm)));
    s.handle(MenuInput::Cancel);assert!(!s.cheat_management_active());
}

#[test]
fn parameter_selection_requires_confirmation_and_ignores_action_clicks(){
    let mut s=Selector::default();s.cheat_mode=true;s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),name:"Example".into(),..Default::default()}],String::new(),0);
    let parameter=super::super::CheatParameter{name:"amount".into(),domain:super::super::CheatParameterDomain::List(vec![2,7,10]),labels:vec![]};
    s.open_cheat_management(vec![super::super::CheatSource{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![parameter]}],vec![]);
    s.handle(MenuInput::Confirm);assert!(s.manage.as_ref().unwrap().parameter_mode);assert!(s.cheat_selected_parameters().is_none());
    let g=s.layout(940,620);let action=g.actions[2];assert!(s.click(action[0]+2,action[1]+2,940,620,true).is_none());assert!(s.cheat_selected_parameters().is_none());
    s.tab(false);assert_eq!(s.manage.as_ref().unwrap().focus,0);
    let next=g.controls[1];s.click(next[0]+2,next[1]+2,940,620,true);assert_eq!(s.manage.as_ref().unwrap().current_parameter(),Some(("amount",7)));
    let select=g.controls[2];s.click(select[0]+2,select[1]+2,940,620,true);assert_eq!(s.cheat_selected_parameters(),Some(vec![("amount",7)]));
    assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::Prepare)));
    s.cheat_management_feedback("Review".into(),Some("preview".into()));s.handle(MenuInput::Cancel);assert!(s.cheat_preview_reviewed());
    s.handle(MenuInput::Left);s.handle(MenuInput::Confirm);assert!(s.manage.as_ref().unwrap().parameter_mode);assert!(!s.cheat_preview_reviewed());assert!(s.cheat_selected_parameters().is_none());
    s.handle(MenuInput::Cancel);assert!(s.cheat_management_active());assert!(s.cheat_selected_source().is_none());
}

#[test]
fn parameter_selector_renders_value_and_focus_in_both_languages(){
    let mut s=Selector::default();s.cheat_mode=true;s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),name:"Example".into(),..Default::default()}],String::new(),0);
    let parameter=super::super::CheatParameter{name:"amount".into(),domain:super::super::CheatParameterDomain::Range{min:2,max:8,step:2},labels:vec![]};
    s.open_cheat_management(vec![super::super::CheatSource{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![parameter]}],vec![]);
    s.handle(MenuInput::Confirm);s.handle(MenuInput::Down);assert_eq!(s.focus_label(),"Declared parameter: amount · 4");
    for (language,select,description) in [(input_config::Language::default(),"Select value","Declared parameter: amount · 4"),(input_config::Language::parse("it"),"Seleziona valore","Parametro dichiarato: amount · 4")] {
        let mut pixels=vec![[0,0,0,255];940*620];let runs=diagnostics::capture(||draw(&s,&mut pixels,940,620,language)).unwrap().runs;
        assert!(runs.iter().any(|r|r.text==select));assert!(runs.iter().any(|r|r.text==description));
        assert!(!runs.iter().any(|r|r.text=="Arm" || r.text=="Attiva"));
    }
    s.handle(MenuInput::Confirm);assert_eq!(s.cheat_selected_parameters(),Some(vec![("amount",4)]));assert_eq!(s.focus_label(),"Prepare preview");
}

#[test]
fn parameter_label_follows_the_numeric_choice_and_keeps_explicit_confirmation(){
    let mut s=Selector::default();s.cheat_mode=true;s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),name:"Example".into(),..Default::default()}],String::new(),0);
    let parameter=super::super::CheatParameter{name:"ability".into(),domain:super::super::CheatParameterDomain::List(vec![2,10]),labels:vec![(2,"Battle Tech".into()),(10,"Black Magic".into())]};
    s.open_cheat_management(vec![super::super::CheatSource{index:0,name:"source".into(),master:"CH-0002".into(),assessment:"parameter-required".into(),parameters:vec![parameter]}],vec![]);
    s.handle(MenuInput::Confirm);assert_eq!(s.focus_label(),"Declared parameter: ability · 2 · Battle Tech");
    s.handle(MenuInput::Down);assert_eq!(s.focus_label(),"Declared parameter: ability · 10 · Black Magic");
    assert!(s.cheat_selected_parameters().is_none());
    for language in [input_config::Language::default(),input_config::Language::parse("it")] {
        let mut pixels=vec![[0,0,0,255];940*620];let runs=diagnostics::capture(||draw(&s,&mut pixels,940,620,language)).unwrap().runs;
        assert!(runs.iter().any(|r|r.text.contains("ability · 10 · Black Magic")));
    }
    s.handle(MenuInput::Confirm);assert_eq!(s.cheat_selected_parameters(),Some(vec![("ability",10)]));
}

#[test]
fn cheat_favorites_and_contextual_actions_work_with_mouse_and_controller() {
    let mut s=Selector::default();s.cheat_mode=true;
    s.set("Cheats".into(),vec![
        ContentEntry{id:"CH-0001".into(),name:"CH-0001 · First".into(),cheat_filter:Some(CheatFilter::default()),..Default::default()},
        ContentEntry{id:"CH-0002".into(),name:"CH-0002 · Second".into(),cheat_filter:Some(CheatFilter{armed:true,..Default::default()}),..Default::default()},
    ],String::new(),0);
    s.focus=7;s.handle(MenuInput::Confirm); // Favorite first entry.
    let g=s.layout(940,620);let button=g.actions[7];
    assert_eq!(s.click(button[0]+2,button[1]+2,940,620,true),None);
    assert_eq!(s.collection,1);assert_eq!(s.visible.len(),1);assert_eq!(s.focus_label(),"Show all");
    s.handle(MenuInput::Confirm);assert_eq!(s.collection,0);assert_eq!(s.visible.len(),2);
    s.focus=5;assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatManage));
    s.cursor=1;s.focus=8;
    assert_eq!(s.focus_label(),"Disarm");
    assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(super::super::CheatRequest::Disarm)));
}

#[test]
fn cheat_shortcuts_use_existing_actions_and_are_disabled_in_editors() {
    let mut s=Selector::default();s.cheat_mode=true;
    s.set("Cheats".into(),vec![ContentEntry{id:"CH-0001".into(),name:"Example".into(),cheat_filter:Some(CheatFilter::default()),..Default::default()}],String::new(),0);
    assert!(s.cheat_shortcuts_available());
    assert_eq!(s.cheat_shortcut(CheatShortcut::Favorite),None);
    assert!(s.favorites.contains("CH-0001"));
    assert_eq!(s.cheat_shortcut(CheatShortcut::Favorites),None);assert_eq!(s.collection,1);
    assert_eq!(s.cheat_shortcut(CheatShortcut::Activate),Some(MenuAction::CheatManage));
    s.cheat_shortcut(CheatShortcut::Details);assert!(s.detail_mode);
    assert!(!s.cheat_shortcuts_available());
    assert_eq!(s.cheat_shortcut(CheatShortcut::Favorite),None);assert!(s.favorites.contains("CH-0001"));
    s.detail_mode=false;s.start_search();assert!(!s.cheat_shortcuts_available());
}

#[test]
fn cheat_catalog_uses_two_action_rows_and_gives_the_list_more_space() {
    let mut s=Selector::default();s.cheat_mode=true;
    for (w,h) in [(627,241),(940,360),(1280,480)] {
        let normal=geometry(w,h);let compact=s.layout(w,h);
        assert!(compact.visible>normal.visible,"{w}x{h}");
        assert_eq!(compact.actions[0][1],compact.actions[3][1]);
        assert_eq!(compact.actions[4][1],compact.actions[7][1]);
        assert!(compact.list[1]+compact.list[3]<=compact.actions[0][1]);
        for r in compact.actions {assert!(r[0]+r[2]<=w && r[1]+r[3]<=h);}
    }
}

#[test]
fn managed_cheat_identity_survives_filter_hiding_it_after_activation() {
    let mut s=Selector::default();s.cheat_mode=true;
    let entry=|id:&str,armed|ContentEntry{id:id.into(),name:id.into(),cheat_filter:Some(CheatFilter{armed,..Default::default()}),..Default::default()};
    s.set("Cheats".into(),vec![entry("CH-0001",false),entry("CH-0002",false)],String::new(),0);
    s.filter=5;s.rebuild(None);
    s.open_cheat_management(vec![],vec![]);
    s.set("Cheats".into(),vec![entry("CH-0001",true),entry("CH-0002",false)],String::new(),0);
    assert_eq!(s.highlighted_id(),Some("CH-0002"));
    assert_eq!(s.cheat_managed_id(),Some("CH-0001"));
    let mut pixels=vec![[0,0,0,255];940*620];
    let runs=diagnostics::capture(||draw(&s,&mut pixels,940,620,input_config::Language::parse("it"))).unwrap().runs;
    assert!(runs.iter().any(|run|run.text.contains("CH-0001")));
    assert!(runs.iter().any(|run|run.text.contains("Attivo: disattiva")));
}

#[test]
fn preset_page_requires_explicit_selection_and_resets_review_on_change() {
    use super::super::CheatRequest as R;
    let mut s=Selector::default();s.cheat_mode=true;
    s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),..Default::default()}],String::new(),0);
    s.open_cheat_management(vec![],vec!["alpha".into(),"beta".into()]);
    s.handle(MenuInput::Right);for _ in 0..5{s.handle(MenuInput::Down);}s.handle(MenuInput::Confirm);
    assert!(s.manage.as_ref().unwrap().preset_mode);assert!(s.cheat_preset_selected().is_none());
    s.handle(MenuInput::Right);s.handle(MenuInput::Down);
    assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(R::PresetPrepare)));
    assert!(s.cheat_preset_selected().is_none());
    s.handle(MenuInput::Left);s.handle(MenuInput::Confirm);assert_eq!(s.cheat_preset_selected(),Some("alpha"));
    s.cheat_management_feedback("Review".into(),Some("all operations".into()));s.cheat_set_preview_token("token".into());
    assert!(s.cheat_preview_token().is_none());s.handle(MenuInput::Cancel);assert_eq!(s.cheat_preview_token(),Some("token"));
    s.handle(MenuInput::Down);s.handle(MenuInput::Confirm);assert_eq!(s.cheat_preset_selected(),Some("beta"));assert!(s.cheat_preview_token().is_none());
    s.handle(MenuInput::Right);for _ in 0..4{s.handle(MenuInput::Down);}s.handle(MenuInput::Confirm);
    assert!(s.preset_name_edit);s.text("my_preset");s.finish_search(false);assert_eq!(s.cheat_preset_name(),Some("my_preset"));
    s.handle(MenuInput::Confirm);s.text("discard");s.finish_search(true);assert_eq!(s.cheat_preset_name(),Some("my_preset"));
    s.handle(MenuInput::Up);s.handle(MenuInput::Up);s.handle(MenuInput::Up);s.handle(MenuInput::Up);
    assert_eq!(s.handle(MenuInput::Confirm),Some(MenuAction::CheatRequest(R::PresetSave)));
}

#[test]
fn preset_page_draws_actions_and_supports_pointer_selection() {
    let mut s=Selector::default();s.cheat_mode=true;
    s.set("Cheat".into(),vec![ContentEntry{id:"CH-0001".into(),..Default::default()}],String::new(),0);
    s.open_cheat_management(vec![],vec!["example".into()]);
    let g=geometry(940,620);let center=|r:[usize;4]|(r[0]+r[2]/2,r[1]+r[3]/2);
    let (x,y)=center(g.actions[5]);s.click(x,y,940,620,true);
    let mut pixels=vec![[0,0,0,255];940*620];
    let capture=diagnostics::capture(||draw(&s,&mut pixels,940,620,input_config::Language::default())).unwrap();
    for label in ["CHEAT PRESETS","Select preset","Save preset","Prepare preset","Arm preset","Edit preset name"] {
        assert!(capture.runs.iter().any(|run|run.text==label),"missing {label}");
    }
    let (x,y)=center(g.controls[2]);s.click(x,y,940,620,true);assert_eq!(s.cheat_preset_selected(),Some("example"));
    let (x,y)=center(g.actions[1]);assert_eq!(s.click(x,y,940,620,true),Some(MenuAction::CheatRequest(super::super::CheatRequest::PresetPrepare)));
}

#[test]
fn semantic_observation_never_adjusts_geometry_scroll_or_confirms_content(){
    let mut selector=Selector::default();
    selector.set("Synthetic".into(),(0..40).map(|i|ContentEntry{id:format!("id-{i}"),name:format!("row-{i}"),..Default::default()}).collect(),String::new(),1);
    selector.first.set(17);selector.detail_scroll.set(23);
    let before=(selector.first.get(),selector.detail_scroll.get(),selector.cursor,selector.confirmed.clone());
    for _ in 0..20{let _=selector.selection_state();}
    assert_eq!(before,(selector.first.get(),selector.detail_scroll.get(),selector.cursor,selector.confirmed.clone()));
}
#[test]
fn selector_information_localizes_labels_without_reinterpreting_values() {
    let mut s=Selector::default();s.set("Cheat · read only".into(),vec![ContentEntry{id:"CH-0001".into(),kind:"{query}".into(),..Default::default()}],String::new(),0);
    s.query="{kind}".into();s.kind=1;
    let en=s.description(input_config::Language::default());let it=s.description(input_config::Language::parse("it"));
    assert!(en.contains("Search: {kind}\nType: {query}"));
    assert!(it.contains("Ricerca: {kind}\nTipo: {query}"));
    assert!(it.starts_with("Cheat · sola lettura"));
}
fn entries() -> Vec<ContentEntry> {
    (0..30)
        .map(|n| ContentEntry {
            id: format!("content-{n:064}"),
            name: if n < 2 {
                "Same name".into()
            } else {
                format!("Item {n:02}")
            },
            kind: if n % 2 == 0 {
                "replay-profile"
            } else {
                "snapshot"
            }
            .into(),
            location: format!("portable/Data/config/{n}/replay.json"),
            identity: "a".repeat(64),
            description:
                "A long description with Unicode è, Ω and text that must remain readable. "
                    .repeat(8),
            reason: "Listing only; no execution".into(),
            status: "available".into(),
            compatibility: "unknown".into(),
            available: true,
            cheat_filter: None,
        })
        .collect()
}

#[test]
fn cheat_filters_use_machine_metadata_and_follow_armed_changes() {
    let entry = |id: &str, armed, declared_match, plan_candidate, values_required| ContentEntry {
        id: id.into(), name: id.into(), available: true,
        // Localized presentation must not control the filter.
        status: "Attivo".into(), compatibility: "Non verificato".into(),
        cheat_filter: Some(CheatFilter { armed, declared_match, plan_candidate, values_required }),
        ..Default::default()
    };
    let mut s = Selector::default();
    s.cheat_mode = true;
    s.set("Cheats".into(), vec![entry("a", true, true, true, false), entry("b", false, true, false, true), entry("c", false, false, false, false)], String::new(), 0);
    s.focus = 2;
    let visible = |s: &Selector| s.visible.iter().map(|&i| s.entries[i].id.clone()).collect::<Vec<_>>();
    for expected in [vec!["a"], vec!["a", "b"], vec!["b", "c"], vec!["a"], vec!["b", "c"], vec!["b"], vec!["a", "b", "c"]] {
        s.handle(MenuInput::Confirm);
        s.handle(MenuInput::Right);
        s.handle(MenuInput::Confirm);
        assert_eq!(visible(&s), expected);
    }
    s.handle(MenuInput::Confirm);
    for _ in 0..4 { s.handle(MenuInput::Right); }
    s.handle(MenuInput::Confirm);
    assert_eq!(visible(&s), vec!["a"]);
    s.set("Cheats".into(), vec![entry("a", false, true, true, false), entry("b", true, true, false, true)], String::new(), 1);
    assert_eq!(visible(&s), vec!["b"]);
    assert_eq!(input_config::Language::parse("it").text(CHEAT_FILTERS[4]), "Attivi");
    assert_eq!(input_config::Language::parse("it").text(CHEAT_FILTERS[6]), "Valori da scegliere");
}
fn selector() -> Selector {
    let mut s = Selector::default();
    s.set("Test".into(), entries(), "Refresh explicitly".into(), 7);
    s
}
#[test]
fn same_names_keep_identity_and_explicit_selection_is_distinct_from_highlight() {
    let mut s = selector();
    let first = s.entries.iter().position(|e| e.id.ends_with('0')).unwrap();
    let second = 1;
    s.cursor = s.visible.iter().position(|i| *i == first).unwrap();
    s.focus = 5;
    assert!(s.handle(MenuInput::Confirm).is_none());
    assert_eq!(s.focus, 6);
    assert_eq!(
        s.handle(MenuInput::Confirm),
        Some(MenuAction::ContentSelect {
            index: first,
            generation: 7
        })
    );
    s.cursor = s.visible.iter().position(|i| *i == second).unwrap();
    assert_eq!(
        s.handle(MenuInput::Confirm),
        Some(MenuAction::ContentSelect {
            index: second,
            generation: 7
        })
    );
    assert_ne!(s.entries[first].id, s.entries[second].id);
}
#[test]
fn unicode_search_cancel_filter_sort_favorites_and_recent_do_not_execute() {
    let mut s = selector();
    s.start_search();
    s.text("Same name");
    assert_eq!(s.visible.len(), 2);
    s.finish_search(true);
    assert_eq!(s.visible.len(), 30);
    s.start_search();
    s.text("è");
    assert_eq!(s.visible.len(), 30);
    s.text_backspace();
    s.finish_search(false);
    s.focus = 1;
    assert!(s.handle(MenuInput::Confirm).is_none());
    assert!(s.handle(MenuInput::Right).is_none());
    assert_eq!(s.visible.len(), 30);
    assert!(s.handle(MenuInput::Cancel).is_none());
    assert_eq!(s.visible.len(), 30);
    s.handle(MenuInput::Confirm);
    s.handle(MenuInput::Right);
    s.handle(MenuInput::Confirm);
    assert_eq!(s.visible.len(), 15);
    s.focus = 7;
    s.confirm();
    assert_eq!(s.favorites.len(), 1);
    s.collection = 1;
    s.rebuild(None);
    assert_eq!(s.visible.len(), 1);
    let id = s.highlighted_id().unwrap().to_owned();
    s.mark_selected(&id);
    assert_eq!(s.recent.len(), 1);
    s.collection = 2;
    s.rebuild(None);
    assert_eq!(s.visible.len(), 1);
    s.sort = 2;
    s.rebuild(None);
    assert_eq!(s.highlighted_id(), Some(id.as_str()));
}
#[test]
fn empty_unavailable_and_busy_never_open_external_dialogs_implicitly() {
    let mut s = Selector::default();
    s.set("Empty".into(), Vec::new(), String::new(), 1);
    s.focus = 6;
    assert!(s.confirm().is_none());
    assert!(s.message.contains("No matching"));
    s.focus = 9;
    assert_eq!(s.confirm(), Some(MenuAction::ContentRefresh));
    s.focus = 10;
    assert_eq!(s.confirm(), Some(MenuAction::ContentBrowseFile));
    s.focus = 11;
    assert_eq!(s.confirm(), Some(MenuAction::ContentAddFolder));
    s.busy = true;
    assert!(s.confirm().is_none());
    s.focus = 12;
    assert_eq!(s.confirm(), Some(MenuAction::ContentCancel));
    let mut s = selector();
    for e in &mut s.entries {
        e.available = false;
    }
    s.focus = 6;
    assert!(s.confirm().is_none());
}
#[test]
fn controller_keyboard_supports_edit_backspace_done_and_cancel() {
    let mut s = selector();
    s.focus = 0;
    s.confirm();
    s.key = 0;
    s.handle(MenuInput::Confirm);
    assert_eq!(s.query, "a");
    s.key = 37;
    s.handle(MenuInput::Confirm);
    assert_eq!(s.query, "");
    s.text("Unicode è");
    s.key = 39;
    s.handle(MenuInput::Confirm);
    assert!(!s.editing);
    assert_eq!(s.query, "Unicode è");
    s.start_search();
    s.text(" draft");
    s.handle(MenuInput::Cancel);
    assert_eq!(s.query, "Unicode è");
    assert!(!s.editing);
}
#[test]
fn mouse_hit_testing_scroll_details_and_full_ids_are_accessible() {
    let mut s = selector();
    let g = geometry(940, 620);
    s.hover(g.list[0] + 5, g.list[1] + g.row + 4, 940, 620);
    assert_eq!(s.cursor, 1);
    assert!(s
        .click(g.list[0] + 5, g.list[1] + g.row + 4, 940, 620, true)
        .is_none());
    assert_eq!(s.focus, 5);
    for _ in 0..12 {
        s.wheel(false);
    }
    let f = s.focus(940, 620);
    assert!(f.first_visible > 0);
    assert!(f.label.contains("content-"));
    s.focus = 8;
    assert!(matches!(
        s.confirm(),
        Some(MenuAction::ContentPreview { .. })
    ));
    s.handle(MenuInput::Down);
    assert_eq!(s.detail_scroll.get(), 1);
    s.handle(MenuInput::Cancel);
    assert!(!s.detail_mode);
    assert_eq!(s.focus, 5);
}
#[test]
fn shared_text_wrap_preserves_every_character_and_render_stays_in_canvas() {
    let text = "Very long path / 同名 / è / ".repeat(30);
    let lines = Canvas::wrap_text(&text, 13., 80.);
    assert_eq!(lines.concat(), text);
    let mut s = selector();
    s.entries[0].reason = text;
    s.focus = 5;
    for (w, h) in [(1280, 720), (940, 620), (620, 480), (400, 300)] {
        let mut px = vec![[15, 25, 35, 255]; w * h];
        draw(&s, &mut px, w, h, input_config::Language::default());
        assert_eq!(px.len(), w * h);
        let g = geometry(w, h);
        assert!(g.panel[0] + g.panel[2] <= w && g.panel[1] + g.panel[3] <= h);
        let text = diagnostics::capture(|| draw(&s, &mut px, w, h, input_config::Language::default())).unwrap();
        assert!(!text.runs.is_empty());
        assert!(text.runs.iter().all(|r| !r.canvas_clipped));
    }
}

#[test]
fn required_role_survives_category_changes_and_can_be_cleared_for_generic_browse() {
    let mut s = Selector::default();
    let mut a = ContentEntry::default();
    a.id = "a".into();
    a.name = "Same".into();
    a.kind = "snapshot".into();
    a.available = true;
    let mut b = a.clone();
    b.id = "b".into();
    b.kind = "runtime".into();
    s.set("role".into(), vec![a, b], String::new(), 1);
    s.scope(vec!["snapshot".into()], None);
    assert_eq!(s.visible, vec![0]);
    s.focus = 1;
    s.handle(MenuInput::Confirm);
    s.handle(MenuInput::Right);
    s.handle(MenuInput::Confirm);
    assert!(!s.visible.contains(&1));
    s.handle(MenuInput::Confirm);
    assert!(s.choice_focus());
    s.set("Replaced".into(),Vec::new(),String::new(),2);
    assert!(!s.choice_focus());
    s.set("role".into(),vec![ContentEntry{id:"a".into(),kind:"snapshot".into(),..Default::default()},ContentEntry{id:"b".into(),kind:"runtime".into(),..Default::default()}],String::new(),3);
    s.scope(Vec::new(), None);
    assert_eq!(s.visible.len(), 2);
}

#[test]
fn controller_punctuation_page_can_enter_multi_value_parameter_separators() {
    let mut s = Selector::default();
    s.start_editor("streams".into(), String::new(), 2048);
    for _ in 0..3 {
        s.key = 41;
        s.key_confirm();
    }
    assert_eq!(s.key_labels()[0], ",");
    s.key = 0;
    s.key_confirm();
    assert_eq!(s.edited_value(), ",");
    s.key = 39;
    s.key_confirm();
    assert!(!s.editing());
}

#[test]
fn preference_restore_keeps_favorites_recent_and_highlight_without_confirming_or_emitting() {
    let mut s = selector();
    let id = s.entries[3].id.clone();
    let view = ContentViewPreferences {
        query: "Item".into(),
        kind: "snapshot".into(),
        filter: 0,
        sort: 2,
        collection: 1,
        highlighted: Some(id.clone()),
    };
    let history = ContentHistory {
        favorites: vec![id.clone()],
        recent: vec![id.clone()],
    };
    s.restore_preferences(view.clone(), history.clone())
        .unwrap();
    assert_eq!(s.highlighted_id(), Some(id.as_str()));
    assert!(s.confirmed.is_none());
    assert!(!s.busy);
    assert_eq!(s.history(), history);
    assert_eq!(s.preferences(), view);
    let before = s.preferences();
    let mut invalid = before.clone();
    invalid.sort = 99;
    assert!(s
        .restore_preferences(invalid, ContentHistory::default())
        .is_err());
    assert_eq!(s.preferences(), before);
    assert_eq!(s.history(), history);
}

#[test]
fn preference_export_keeps_committed_search_while_an_editor_draft_is_open() {
    let mut s = selector();
    s.query = "Item".into();
    s.rebuild(None);
    let before = s.preferences();
    s.start_search();
    s.text(" draft");
    assert_eq!(s.preferences(), before);
    s.finish_search(true);
    assert_eq!(s.preferences(), before);
    s.start_search();
    s.text(" 03");
    s.finish_search(false);
    assert_eq!(s.preferences().query, "Item 03");
    assert!(s.confirmed.is_none());
}

#[test]
fn virtual_keyboard_keys_remain_inside_small_atlas_bodies() {
    let s=selector();
    for(w,h)in[(352,212),(352,480),(627,218),(950,330)]{
        let g=geometry(w,h);
        for i in 0..42{let r=s.key_rect(g,i);assert!(r[0]+r[2]<=w&&r[1]+r[3]<=h,"{w}x{h} key{i} {r:?}");}
    }
}
