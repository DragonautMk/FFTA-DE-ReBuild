use super::*;
fn root(path: &Path, portable: bool) -> Root {
    Root::new(path, portable).unwrap()
}
fn write(root: &Path, name: &str, data: impl AsRef<[u8]>) {
    let p = root.join(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, data).unwrap();
}
fn profile(rom: &str, name: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-replay-profile","case":name,"rom_sha256":rom,"region":"EUR"})).unwrap()
}
#[test]
fn empty_explicit_refresh_duplicate_names_hashes_and_declared_compatibility() {
    let t = tempfile::tempdir().unwrap();
    let rom = "a".repeat(64);
    let empty = Index::scan(vec![root(t.path(), true)], Some(rom.clone())).unwrap();
    assert!(empty.entries.is_empty());
    assert!(!empty.stats.truncated);
    write(
        t.path(),
        "Data/config/a/replay.json",
        profile(&rom, "Same name"),
    );
    write(
        t.path(),
        "Data/config/b/replay.json",
        profile(&rom, "Same name"),
    );
    write(
        t.path(),
        "Data/config/c/replay.json",
        profile(&"b".repeat(64), "Same name"),
    );
    write(
        t.path(),
        "Data/config/unknown.recipe.json",
        br#"{"schema":1,"kind":"ffta-debug-recipe","name":"Recipe"}"#,
    );
    let index = Index::scan(vec![root(t.path(), true)], Some(rom)).unwrap();
    assert_eq!(index.entries.len(), 4);
    assert!(index
        .entries
        .iter()
        .all(|e| e.availability == "available" && e.sha256.is_some()));
    assert_eq!(
        index
            .entries
            .iter()
            .filter(|e| e.compatibility == "declared-match")
            .count(),
        2
    );
    assert_eq!(
        index
            .entries
            .iter()
            .filter(|e| e.compatibility == "incompatible")
            .count(),
        1
    );
    let copies: Vec<_> = index
        .entries
        .iter()
        .filter(|e| e.duplicate_count == 1)
        .collect();
    assert_eq!(copies.len(), 2);
    assert_ne!(copies[0].id, copies[1].id);
    assert_eq!(copies[0].name, copies[1].name);
    assert!(index
        .entries
        .iter()
        .any(|e| e.kind == Kind::Recipe && e.compatibility == "unknown"));
}
#[test]
fn portable_move_keeps_ids_and_ignores_undeclared_folders() {
    let t = tempfile::tempdir().unwrap();
    let a = t.path().join("Original è");
    let b = t.path().join("Moved ' à");
    fs::create_dir(&a).unwrap();
    write(
        &a,
        "Data/config/a/replay.json",
        profile(&"a".repeat(64), "Move"),
    );
    write(
        &a,
        "unrelated/private.py",
        b"raise RuntimeError('must not run')",
    );
    write(&a, "Data/other/unknown.exe", b"MUST NOT RUN");
    let before = Index::scan(vec![root(&a, true)], None).unwrap();
    assert_eq!(before.entries.len(), 1);
    fs::rename(&a, &b).unwrap();
    let after = Index::scan(vec![root(&b, true)], None).unwrap();
    assert_eq!(before.entries[0].id, after.entries[0].id);
    assert_eq!(before.entries[0].sha256, after.entries[0].sha256);
    assert!(after.stats.skipped_directories >= 2);
    let explicit = Index::scan(vec![root(&b.join("unrelated"), false)], None).unwrap();
    assert_eq!(explicit.entries[0].kind, Kind::HostTool);
}
#[test]
fn selection_recheck_reports_changed_missing_wrong_kind_and_unknown_ids() {
    let t = tempfile::tempdir().unwrap();
    write(t.path(), "replay.json", profile(&"a".repeat(64), "A"));
    let index = Index::scan(vec![root(t.path(), false)], None).unwrap();
    let id = index.entries[0].id.clone();
    assert_eq!(index.recheck(&id, None).unwrap().availability, "available");
    assert_eq!(
        index
            .recheck(&id, Some(Kind::Runtime))
            .unwrap()
            .compatibility,
        "incompatible"
    );
    write(t.path(), "replay.json", profile(&"a".repeat(64), "B"));
    let changed = index.recheck(&id, None).unwrap();
    assert_eq!(changed.availability, "changed");
    assert_eq!(changed.id, id);
    fs::remove_file(t.path().join("replay.json")).unwrap();
    assert_eq!(index.recheck(&id, None).unwrap().availability, "missing");
    assert!(index.recheck("unknown", None).is_err());
    let mut invalid = index;
    invalid.entries[0].relative_path = "../outside.py".into();
    assert!(invalid.recheck(&id, None).is_err());
}
#[test]
fn all_content_families_are_listed_without_execution_or_full_verification() {
    let t = tempfile::tempdir().unwrap();
    for (name, data) in [
        (
            "a/debug-session.json",
            r#"{"schema":1,"kind":"ffta-debug-session","label":"Session"}"#,
        ),
        (
            "b/recording.json",
            r#"{"schema":1,"kind":"ffta-input-recording"}"#,
        ),
        (
            "c/replay.json",
            r#"{"schema":1,"kind":"ffta-debug-replay-profile"}"#,
        ),
        ("recipe.json", r#"{"schema":1,"kind":"ffta-debug-recipe"}"#),
        (
            "campaign.json",
            r#"{"schema":1,"kind":"ffta-debug-campaign"}"#,
        ),
        ("script-commands.json", r#"{"schema":1}"#),
        ("catalog.json", r#"{"schema":1,"cases":[]}"#),
        ("catalogo.json", r#"{"schema":1}"#),
        ("manifest.toml", "schema = 1"),
        ("known-plan.toml", "schema = 1"),
        ("x.ffstate", "SYNTHETIC not a decoded state"),
        ("recomp.exe", "NOT EXECUTABLE"),
        ("cache.dll", "NOT A LIBRARY"),
        ("map.labels", "SYNTHETIC"),
        ("tool.py", "raise RuntimeError('never run')"),
        ("index.html", "<h1>Synthetic report</h1>"),
    ] {
        write(t.path(), name, data);
    }
    let index = Index::scan(vec![root(t.path(), false)], None).unwrap();
    assert_eq!(index.entries.len(), 16);
    for kind in [
        Kind::Session,
        Kind::Recording,
        Kind::ReplayProfile,
        Kind::Recipe,
        Kind::HostTool,
        Kind::GuestScriptCatalog,
        Kind::Campaign,
        Kind::Snapshot,
        Kind::Runtime,
        Kind::Dataset,
        Kind::NativeLibrary,
        Kind::Labels,
        Kind::KnownManifest,
        Kind::SaveCatalog,
        Kind::CheatCatalog,
        Kind::Report,
    ] {
        assert!(index.entries.iter().any(|e| e.kind == kind), "{kind:?}");
    }
    assert!(index
        .entries
        .iter()
        .all(|e| e.compatibility == "unknown" && e.verification == "content-hash-only"));
    assert!(!t.path().join("Data").exists());
}
#[test]
fn malformed_future_schema_and_mismatched_kind_do_not_claim_compatibility() {
    let t = tempfile::tempdir().unwrap();
    write(t.path(), "bad/replay.json", b"{broken");
    write(
        t.path(),
        "future/replay.json",
        br#"{"schema":999,"kind":"ffta-debug-replay-profile"}"#,
    );
    write(
        t.path(),
        "wrong/replay.json",
        br#"{"schema":1,"kind":"unexpected"}"#,
    );
    let index = Index::scan(vec![root(t.path(), false)], None).unwrap();
    assert_eq!(index.entries.len(), 3);
    assert_eq!(
        index
            .entries
            .iter()
            .filter(|e| e.availability == "unreadable")
            .count(),
        1
    );
    assert_eq!(
        index
            .entries
            .iter()
            .filter(|e| e.compatibility == "unsupported")
            .count(),
        2
    );
}
#[test]
fn budgets_depth_issue_caps_and_duplicate_groups_are_bounded() {
    let t = tempfile::tempdir().unwrap();
    for n in 0..MAX_ENTRIES + 2 {
        write(t.path(), &format!("{n:04}.html"), b"SAME SYNTHETIC");
    }
    let index = Index::scan(vec![root(t.path(), false)], None).unwrap();
    assert_eq!(index.entries.len(), MAX_ENTRIES);
    assert!(index.stats.truncated);
    assert!(index
        .entries
        .iter()
        .all(|e| e.duplicate_count == MAX_ENTRIES - 1 && e.duplicate_ids.len() == 8));
    assert!(serde_json::to_vec(&index).unwrap().len() < MAX_INDEX as usize);
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join("large.exe");
    File::create(&file)
        .unwrap()
        .set_len(MAX_HASH_BYTES + 1)
        .unwrap();
    let index = Index::scan(vec![root(d.path(), false)], None).unwrap();
    assert!(index.entries[0].sha256.is_none());
    assert_eq!(index.stats.hashed_bytes, 0);
    let deep = (0..MAX_DEPTH + 2)
        .map(|_| "a")
        .collect::<Vec<_>>()
        .join("/");
    write(d.path(), &(deep + "/index.html"), b"deep");
    let mut index = Index::scan(vec![root(d.path(), false)], None).unwrap();
    assert!(index.stats.truncated);
    for _ in 0..100 {
        index.issue("bounded".into());
    }
    assert_eq!(index.issues.len(), MAX_ISSUES);
    assert!(index.stats.dropped_issues > 0);
}
#[test]
fn duplicate_roots_and_missing_roots_have_explicit_outcomes() {
    let t = tempfile::tempdir().unwrap();
    write(t.path(), "index.html", b"synthetic");
    let r = root(t.path(), false);
    let index = Index::scan(vec![r.clone(), r], None).unwrap();
    assert_eq!(index.entries.len(), 1);
    assert!(index.issues.iter().any(|s| s.contains("duplicate-root")));
    let index = Index::scan(vec![root(&t.path().join("missing"), false)], None).unwrap();
    assert!(index.entries.is_empty());
    assert_eq!(index.issues.len(), 1);
}
#[test]
fn linked_or_shared_inputs_are_not_followed_or_hashed() {
    fn check(path: &Path) {
        let index = Index::scan(vec![root(path, false)], None).unwrap();
        assert_eq!(index.entries.len(), 2);
        assert!(index
            .entries
            .iter()
            .all(|e| e.availability == "nonordinary" && e.sha256.is_none()));
        assert_eq!(index.stats.hashed_bytes, 0);
        assert!(
            !index.issues.is_empty(),
            "directory link must be refused before enumeration"
        );
    }
    // Wine's Windows symlink-creation API may return success without a usable
    // link. The host harness can supply real filesystem links instead, so the
    // negative test still exercises actual aliases rather than a missing file.
    if let Some(path) = std::env::var_os("FFTA_DEBUG_CONTENT_LINK_FIXTURE") {
        check(Path::new(&path));
        return;
    }
    let t = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    write(external.path(), "outside.py", b"DO NOT READ OR RUN");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            external.path().join("outside.py"),
            t.path().join("linked.py"),
        )
        .unwrap();
        std::os::unix::fs::symlink(external.path(), t.path().join("linked-dir")).unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(
            external.path().join("outside.py"),
            t.path().join("linked.py"),
        )
        .unwrap();
        std::os::windows::fs::symlink_dir(external.path(), t.path().join("linked-dir")).unwrap();
    }
    fs::hard_link(
        external.path().join("outside.py"),
        t.path().join("shared.py"),
    )
    .unwrap();
    assert!(
        fs::symlink_metadata(t.path().join("linked.py")).is_ok(),
        "symlink fixture was not materialized"
    );
    check(t.path());
}
#[test]
fn command_parser_rejects_unknown_duplicate_and_ambiguous_options() {
    let a = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    for args in [
        vec![],
        vec!["run"],
        vec!["scan", "--root"],
        vec!["scan", "--root", ".", "--root", "."],
        vec!["scan", "--rom-sha256", "not-a-hash"],
        vec!["scan", "--execute", "tool.py"],
        vec!["check", "index", "id", "--kind", "shell"],
    ] {
        assert!(execute(&a(&args)).is_err(), "{args:?}");
    }
}

#[test]
fn explicit_file_root_never_enumerates_siblings_and_rechecks_exact_reference() {
    let t = tempfile::tempdir().unwrap();
    write(
        t.path(),
        "chosen/replay.json",
        profile(&"a".repeat(64), "Chosen"),
    );
    write(t.path(), "chosen/sibling.py", b"must not run or index");
    let index = Index::scan(
        vec![Root::single_file(&t.path().join("chosen/replay.json")).unwrap()],
        None,
    )
    .unwrap();
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.stats.visited_nodes, 1);
    assert_eq!(index.entries[0].kind, Kind::ReplayProfile);
    assert!(index
        .resolve(&index.entries[0].id, Some(Kind::ReplayProfile))
        .is_ok());
    let mut malicious = index.clone();
    malicious.entries[0].relative_path = "sibling.py".into();
    assert!(malicious.recheck(&malicious.entries[0].id, None).is_err());
    let mut old = serde_json::to_value(&index.roots[0]).unwrap();
    old.as_object_mut().unwrap().remove("file");
    let old: Root = serde_json::from_value(old).unwrap();
    assert!(old.validated().is_err());
    let ordinary = Root::new(t.path(), false).unwrap();
    let old = serde_json::to_value(&ordinary).unwrap();
    assert!(old.get("file").is_none());
    assert!(serde_json::from_value::<Root>(old)
        .unwrap()
        .validated()
        .is_ok());
}
