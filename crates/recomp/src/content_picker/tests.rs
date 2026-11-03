use super::*;
fn wait(p: &mut Picker, m: &mut menu::Menu) {
    for _ in 0..2000 {
        p.poll(m);
        if p.job.is_none() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("bounded synthetic worker did not finish");
}
fn fixture() -> (tempfile::TempDir, Picker, menu::Menu) {
    let t = tempfile::tempdir().unwrap();
    std::fs::write(t.path().join("a.html"), b"SYNTHETIC same").unwrap();
    std::fs::write(t.path().join("b.html"), b"SYNTHETIC same").unwrap();
    let mut p = Picker::default();
    p.roots = vec![Root::new(t.path(), false).unwrap()];
    let mut m = menu::Menu::new(true, true);
    m.set_debug_enabled(true);
    p.open(&mut m, "a".repeat(64)).unwrap();
    (t, p, m)
}
fn wait_id(p: &mut Picker, m: &mut menu::Menu) -> Result<String, String> {
    for _ in 0..2000 {
        if let Some(result) = p.poll_id(m) {
            return result;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("bounded ID worker did not finish");
}
#[test]
fn id_selection_reuses_preflight_and_preserves_pins_on_failure_or_cancel() {
    let (t, mut p, mut m) = fixture();
    let path = t.path().join("catalog.json");
    std::fs::write(&path, br#"{"schema":1,"cases":[]}"#).unwrap();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    let id = p.index.as_ref().unwrap().entries.iter()
        .find(|entry| entry.kind == Kind::SaveCatalog).unwrap().id.clone();
    assert!(p.select_id(ContentSlot::Catalog, "unknown").is_err());
    p.select_id(ContentSlot::Catalog, &id).unwrap();
    assert!(p.status()["busy"].as_bool().unwrap());
    assert!(p.refresh(&mut m).is_err());
    assert_eq!(wait_id(&mut p, &mut m).unwrap(), id);
    let previous = p.slots[&ContentSlot::Catalog].entry.sha256.clone();
    p.select_id(ContentSlot::Replay, &id).unwrap();
    assert!(wait_id(&mut p, &mut m).is_err());
    assert!(!p.has_slot(ContentSlot::Replay));
    p.select_id(ContentSlot::Catalog, &id).unwrap();
    assert!(p.cancel_id());
    assert!(wait_id(&mut p, &mut m).unwrap_err().contains("cancelled"));
    assert_eq!(p.slots[&ContentSlot::Catalog].entry.sha256, previous);
    std::fs::write(&path, b"changed").unwrap();
    p.select_id(ContentSlot::Catalog, &id).unwrap();
    assert!(wait_id(&mut p, &mut m).is_err());
    assert_eq!(p.slots[&ContentSlot::Catalog].entry.sha256, previous);
}
#[test]
fn id_selection_discards_results_for_a_changed_context_or_catalog() {
    let (t, mut p, mut m) = fixture();
    std::fs::write(t.path().join("catalog.json"), br#"{"schema":1,"cases":[]}"#).unwrap();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    let id = p.index.as_ref().unwrap().entries.iter()
        .find(|entry| entry.kind == Kind::SaveCatalog).unwrap().id.clone();
    p.select_id(ContentSlot::Catalog, &id).unwrap();
    p.context = Some("different context".into());
    assert!(wait_id(&mut p, &mut m).is_err());
    assert!(!p.has_slot(ContentSlot::Catalog));
    p.select_id(ContentSlot::Catalog, &id).unwrap();
    p.generation = p.generation.wrapping_add(1);
    assert!(wait_id(&mut p, &mut m).is_err());
    assert!(!p.has_slot(ContentSlot::Catalog));
}
#[test]
fn control_scan_pages_and_refuses_cancelled_or_stale_completion(){
    let (t,mut p,_m)=fixture();std::fs::write(t.path().join("catalog.json"),br#"{"schema":1,"cases":[]}"#).unwrap();
    let mut menu=menu::Menu::new(true,true);assert!(!menu.content_active());
    let wait=|p:&mut Picker,m:&mut menu::Menu|{for _ in 0..2000{if let Some(r)=p.poll_refresh_control(m){return r;}std::thread::sleep(std::time::Duration::from_millis(1));}panic!("bounded scan did not finish")};
    assert!(p.page(0,1,None).is_err());p.refresh_control().unwrap();assert!(p.refresh_control().is_err());wait(&mut p,&mut menu).unwrap();assert!(!menu.content_active());
    let first=p.page(0,1,None).unwrap();let generation=first["generation"].as_u64().unwrap();let total=first["total"].as_u64().unwrap();assert!(total>=3);assert_eq!(first["entries"].as_array().unwrap().len(),1);assert_eq!(first["next_offset"],1);
    assert!(p.page(0,17,None).is_err());assert!(p.page(0,1,Some(generation+1)).is_err());assert!(p.page(total as u32+1,1,None).is_err());
    p.refresh_control().unwrap();assert!(p.cancel_refresh_control());assert!(wait(&mut p,&mut menu).unwrap_err().contains("cancelled"));assert_eq!(p.generation,generation);
    p.refresh_control().unwrap();p.context=Some("changed".into());assert!(wait(&mut p,&mut menu).is_err());assert_eq!(p.generation,generation);
}
#[test]
fn opening_cancelled_external_and_highlight_never_start_workers_or_select() {
    let (_t, mut p, mut m) = fixture();
    assert!(!p.status()["busy"].as_bool().unwrap());
    assert_eq!(p.status()["entries"], 0);
    assert!(p.selected.is_none());
    p.external_result(None, true, &mut m).unwrap();
    p.external_result(None, false, &mut m).unwrap();
    assert!(p.job.is_none());
    assert_eq!(p.roots.len(), 1);
    assert!(p.selected.is_none());
}
#[test]
fn refresh_select_recheck_and_generation_refusal_share_the_catalog_service() {
    let (t, mut p, mut m) = fixture();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    assert_eq!(p.status()["entries"], 2);
    let gen = p.generation;
    p.choose(&mut m, 0, gen, false).unwrap();
    wait(&mut p, &mut m);
    let id = p.selected.as_ref().unwrap().entry.id.clone();
    assert!(m.content_active());
    assert!(p.choose(&mut m, 0, gen + 1, false).is_err());
    let path = p.selected.as_ref().unwrap().path.clone();
    std::fs::write(path, b"CHANGED").unwrap();
    p.choose(&mut m, 0, gen, false).unwrap();
    wait(&mut p, &mut m);
    assert_eq!(p.selected.as_ref().unwrap().entry.id, id);
    assert!(t.path().is_dir());
}
#[test]
fn late_selection_after_cancel_is_discarded_and_imported_file_does_not_scan_siblings() {
    let (_t, mut p, mut m) = fixture();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    p.choose(&mut m, 0, p.generation, false).unwrap();
    p.cancel();
    wait(&mut p, &mut m);
    assert!(p.selected.is_none());
    let external = tempfile::tempdir().unwrap();
    let file = external.path().join("one.html");
    std::fs::write(&file, b"SYNTHETIC").unwrap();
    std::fs::write(
        external.path().join("NEVER.py"),
        b"raise RuntimeError('never')",
    )
    .unwrap();
    p.external_result(Some(file), true, &mut m).unwrap();
    wait(&mut p, &mut m);
    assert_eq!(p.status()["entries"], 3);
    assert!(p
        .index
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .all(|e| e.kind != Kind::HostTool));
}

#[test]
#[ignore = "explicit public synthetic GUI fixture; never reads personal game inputs"]
pub(crate) fn write_synthetic_menu_fixture() {
    let root = PathBuf::from(
        std::env::var_os("FFTA_SELECTOR_TEST_ROOT").expect("explicit new fixture root"),
    );
    assert!(!root.exists());
    let executable = PathBuf::from(
        std::env::var_os("FFTA_SELECTOR_TEST_RUNTIME").expect("explicit candidate runtime"),
    );
    let mut rom = vec![0u8; 0x200];
    rom[..4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
    rom[0xa0..0xac].copy_from_slice(b"DBG2-PICKER!");
    rom[0x100..0x106].copy_from_slice(b"SRAM_V");
    let machine = crate::make_machine(rom.clone(), None);
    let state = machine.save_state().unwrap();
    let mut av = input_config::AvConfig::default();
    av.audio_mute = true;
    av.video_aspect = "16:9".into();
    av.screen = "raw".into();
    let mut input = input_config::InputConfig::default();
    input.keys[input_config::Button::A.index()] = "Enter".into();
    input.keys[input_config::Button::Start.index()] = "Space".into();
    let directory = crate::debug_session::create(
        &root,
        crate::debug_session::Source {
            rom: &rom,
            bios: None,
            save: machine.bus.save_data(),
            state: Some(&state),
            av,
            input,
            interp: true,
            dataset: None,
            executable: &executable,
        },
    )
    .unwrap();
    let hash = gba_core::ffta_rom::digest(&rom);
    for name in ["First folder", "Second folder"] {
        let dir = directory.join("Data/config/replay").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("replay.json"),serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-replay-profile","case":"Same name","rom_sha256":hash,"region":"SYNTHETIC","description":"Public host-menu fixture; no functional replay or game data."})).unwrap()).unwrap();
    }
    let mut slots = crate::save_state::Slots::new(&directory.join("Data/saves/ffta.sav"), &rom);
    let image = (0..240 * 160)
        .map(|i| [(i % 240) as u8, (i / 240) as u8, 90, 255])
        .collect::<Vec<_>>();
    slots.save(&machine, 2, &image, false).unwrap();
    std::fs::write(directory.join("fixture-identity.json"),serde_json::to_vec_pretty(&json!({"schema":1,"kind":"public-synthetic-selector-fixture","rom_sha256":hash,"state_sha256":gba_core::ffta_rom::digest(&state),"frame":machine.bus.frames,"clock":machine.bus.clock,"scope":"Host menu only; no copyrighted input or guest execution required"})).unwrap()).unwrap();
}

#[test]
fn completed_preview_never_leaves_a_working_status_even_after_highlight_changes() {
    fn rendered(menu: &menu::Menu) -> String {
        let mut px = vec![[0, 0, 0, 255]; 940 * 620];
        let keys = std::array::from_fn(|_| String::new());
        let text = menu::diagnostics::capture(|| {
            menu.draw(
                &input_config::AvConfig::default(),
                &keys,
                "",
                "",
                "",
                "",
                &mut px,
                940,
                620,
            )
        })
        .unwrap();
        text.runs
            .into_iter()
            .map(|r| r.text)
            .collect::<Vec<_>>()
            .join("\n")
    }
    let (_t, mut p, mut m) = fixture();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    let id = m.content_highlight().unwrap().to_owned();
    let index = p
        .index
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.id == id)
        .unwrap();
    p.choose(&mut m, index, p.generation, true).unwrap();
    wait(&mut p, &mut m);
    let text = rendered(&m);
    assert!(!text.contains("Working on selected"));
    assert!(text.contains("No embedded"));
    assert!(p.selected.is_none());
    p.choose(&mut m, index, p.generation, true).unwrap();
    m.content_start_search();
    m.content_text("b.html");
    m.content_finish_search(false);
    wait(&mut p, &mut m);
    assert!(!rendered(&m).contains("Working on selected"));
    assert!(p.selected.is_none());
}

#[test]
fn typed_slot_results_preserve_prior_refs_and_never_leak_after_view_cancel() {
    let (t, mut p, mut m) = fixture();
    let rom = "a".repeat(64);
    let file = t.path().join("runtime.exe");
    std::fs::write(&file, b"PUBLIC synthetic runtime; never run").unwrap();
    p.open_for(
        &mut m,
        rom.clone(),
        ContentSlot::RuntimeA,
        "Choose only".into(),
    )
    .unwrap();
    p.refresh(&mut m).unwrap();
    wait(&mut p, &mut m);
    let row = p
        .index
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.kind == Kind::Runtime)
        .unwrap();
    p.choose(&mut m, row, p.generation, false).unwrap();
    wait(&mut p, &mut m);
    assert!(p.has_slot(ContentSlot::RuntimeA));
    let old = p.slot_path(ContentSlot::RuntimeA, &rom).unwrap();
    p.open_for(
        &mut m,
        rom.clone(),
        ContentSlot::Library,
        "Wrong types refused".into(),
    )
    .unwrap();
    p.choose(&mut m, row, p.generation, false).unwrap();
    wait(&mut p, &mut m);
    assert!(!p.has_slot(ContentSlot::Library));
    assert_eq!(p.slot_path(ContentSlot::RuntimeA, &rom).unwrap(), old);
    p.open_for(
        &mut m,
        rom.clone(),
        ContentSlot::RuntimeB,
        "Cancel pending".into(),
    )
    .unwrap();
    p.choose(&mut m, row, p.generation, false).unwrap();
    p.cancel();
    wait(&mut p, &mut m);
    assert!(!p.has_slot(ContentSlot::RuntimeB));
    std::fs::write(&file, b"CHANGED").unwrap();
    assert!(p.slot_path(ContentSlot::RuntimeA, &rom).is_err());
    assert!(p.has_slot(ContentSlot::RuntimeA));
}

fn saved_restore_fixture() -> (tempfile::TempDir, Picker, menu::Menu) {
    use crate::debug_choices::{memory::Memory, Anchors};
    let t = tempfile::tempdir().unwrap();
    let anchors = Anchors {
        app: t.path().into(),
        config: t.path().join("Data/config"),
    };
    let mut p = Picker::default();
    p.rom = Some("a".repeat(64));
    let memory = Memory::load(anchors, &"a".repeat(64)).unwrap();
    p.context = Some(memory.context.clone());
    p.memory = Some(memory);
    let mut menu=menu::Menu::new(true,true);menu.set_debug_enabled(true);
    (t,p,menu)
}
#[test]
fn saved_restore_survives_leaving_the_picker_without_opening_pages_or_selecting_content() {
    use crate::debug_choices::memory::Memory;
    let (t, mut p, mut m) = saved_restore_fixture();
    let path = t.path().join("runtime.exe");
    std::fs::write(&path, b"PUBLIC NEVER RUN").unwrap();
    let r = Reference::explicit(&path, ContentSlot::HostRuntime, &"a".repeat(64)).unwrap();
    p.memory
        .as_mut()
        .unwrap()
        .record_input(ContentSlot::HostRuntime, &r);
    let (c, a, inputs) = p.memory.as_ref().unwrap().restore_request();
    let batch = Memory::restore(c, a, inputs, "a".repeat(64));
    p.job = Some(Job {
        view: p.view,
        handle: std::thread::spawn(move || Ok(ResultData::Restored(batch))),
    });
    p.cancel();
    wait(&mut p, &mut m);
    assert!(p.has_slot(ContentSlot::HostRuntime));
    assert!(!m.content_active());
    assert!(p.selected.is_none());
    assert!(p.index.is_none());
    assert!(!p.memory.as_ref().unwrap().file.path.exists());
}
#[test]
fn late_saved_restore_cannot_replace_a_newer_explicit_input_pin() {
    use crate::debug_choices::memory::Memory;
    let (t, mut p, mut m) = saved_restore_fixture();
    let old = t.path().join("old.exe");
    let new = t.path().join("new.exe");
    std::fs::write(&old, b"PUBLIC OLD").unwrap();
    std::fs::write(&new, b"PUBLIC NEW").unwrap();
    let old = Reference::explicit(&old, ContentSlot::HostRuntime, &"a".repeat(64)).unwrap();
    let new = Reference::explicit(&new, ContentSlot::HostRuntime, &"a".repeat(64)).unwrap();
    p.memory
        .as_mut()
        .unwrap()
        .record_input(ContentSlot::HostRuntime, &old);
    let (c, a, inputs) = p.memory.as_ref().unwrap().restore_request();
    let batch = Memory::restore(c, a, inputs, "a".repeat(64));
    p.memory
        .as_mut()
        .unwrap()
        .record_input(ContentSlot::HostRuntime, &new);
    p.slots.insert(ContentSlot::HostRuntime, new.clone());
    p.job = Some(Job {
        view: p.view,
        handle: std::thread::spawn(move || Ok(ResultData::Restored(batch))),
    });
    wait(&mut p, &mut m);
    assert_eq!(
        p.slots[&ContentSlot::HostRuntime].entry.sha256,
        new.entry.sha256
    );
    assert!(p.saved_issues.is_empty());
}
#[test]
fn missing_saved_roles_have_visible_explanations_and_keep_the_original_pin() {
    use crate::debug_choices::memory::Memory;
    let (t, mut p, mut m) = saved_restore_fixture();
    let path = t.path().join("missing.exe");
    std::fs::write(&path, b"PUBLIC MISSING").unwrap();
    let r = Reference::explicit(&path, ContentSlot::HostRuntime, &"a".repeat(64)).unwrap();
    let pin = r.entry.sha256.clone();
    p.memory
        .as_mut()
        .unwrap()
        .record_input(ContentSlot::HostRuntime, &r);
    std::fs::remove_file(path).unwrap();
    let (c, a, inputs) = p.memory.as_ref().unwrap().restore_request();
    let batch = Memory::restore(c, a, inputs, "a".repeat(64));
    p.job = Some(Job {
        view: p.view,
        handle: std::thread::spawn(move || Ok(ResultData::Restored(batch))),
    });
    wait(&mut p, &mut m);
    assert!(!p.has_slot(ContentSlot::HostRuntime));
    assert!(p.saved_issues[&ContentSlot::HostRuntime].contains("unavailable"));
    assert_eq!(
        Some(
            p.memory.as_ref().unwrap().file.state.inputs[0]
                .sha256
                .clone()
        ),
        pin
    );
    m.open_tool(
        menu::DebugTool::Homepage,
        crate::debug_workbench::fields(menu::DebugTool::Homepage),
        String::new(),
    );
    m.handle(
        menu::MenuInput::Down,
        &mut input_config::AvConfig::default(),
    );
    assert!(m
        .debug_focus(940, 620)
        .label
        .contains("Saved choice unavailable"));
}
