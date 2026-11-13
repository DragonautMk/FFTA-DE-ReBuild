//! Readonly operation metadata shared by help, menu diagnostics and local control.
//! Descriptors never execute an operation or replace its existing validation.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::OnceLock;

pub(crate) const VERSION: u32 = 1;
const INVENTORY: &str = include_str!("debug_operations.json");

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Parameter {
    pub name: String,
    pub value_type: String,
    pub constraint: String,
    pub current_menu: String,
    pub default: Value,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MenuRoute {
    pub current: Option<String>,
    pub destination: String,
    pub implemented: bool,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dialog {
    pub site: String,
    pub when: String,
    pub automatic: bool,
    pub follow_up: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Operation {
    pub id: String,
    pub title: String,
    pub source: String,
    pub handler: String,
    pub cli: Option<String>,
    pub parameters: Vec<Parameter>,
    pub selection: Vec<String>,
    pub menu: MenuRoute,
    pub allowed_state: String,
    pub effects: String,
    pub result: String,
    pub limits: String,
    pub open_work: Vec<String>,
    pub external_dialog: Option<Dialog>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Registry {
    pub schema: u32,
    pub kind: String,
    pub version: u32,
    pub execution: String,
    pub result_contract: Value,
    pub operations: Vec<Operation>,
}
pub(crate) fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY
        .get_or_init(|| serde_json::from_str(INVENTORY).expect("embedded debug operation contract"))
}
pub(crate) fn get(id: &str) -> Option<&'static Operation> {
    registry().operations.iter().find(|op| op.id == id)
}

pub(crate) fn workbench_operation(tool:menu::DebugTool)->&'static str{use menu::DebugTool::*;match tool{Replay=>"workbench.replay",Audit=>"workbench.audit",Capture=>"workbench.capture",Compare=>"workbench.compare",Step=>"workbench.step",Inspect=>"workbench.inspect",Audio=>"workbench.audio",Code=>"workbench.code",Gameplay=>"workbench.gameplay",Recent=>"workbench.recent",Graphics=>"workbench.graphics",Issue=>"workbench.issue",Homepage=>"tools.homepage",CampaignReport=>"tools.campaign-report",ArtifactPreview=>"tools.artifact-preview",Recipe=>"tools.recipe",GuestScripts=>"tools.guest-scripts"}}

/// Exhaustive match: adding a local-control action requires choosing its stable ID.
impl crate::debug_control::Action {
    pub(crate) fn operation_id(&self) -> &'static str {
        use crate::debug_control::Action::*;
        match self {
            Status => "control.status",
            Cancel { .. } => "control.cancel",
            SelectId { .. } => "content.select",
            ContentRefresh => "content.scan",
            ContentList { .. } => "content.list",
            CheatRefresh => "cheats.refresh",
            CheatList { .. } => "cheats.list",
            CheatShow { .. } => "cheats.show",
            CheatCapabilities { .. } => "cheats.capabilities",
            CheatPrepare { .. } => "cheats.prepare-once",
            CheatApply { .. } => "cheats.apply-once",
            CheatArm { .. } => "cheats.arm",
            CheatDisarm { .. } => "cheats.disarm",
            CheatPresetSave { .. } => "cheats.preset-save",
            CheatPresetPrepare { .. } => "cheats.preset-prepare",
            CheatPresetArm { .. } => "cheats.preset-arm",
            Replay { mode,.. } => match mode {crate::debug_control::ReplayMode::Plain=>"replay.run",crate::debug_control::ReplayMode::Audit=>"audit.run",crate::debug_control::ReplayMode::Capture=>"capture.run",crate::debug_control::ReplayMode::Comparison=>"compare.run"},
            Pause => "control.pause",
            Resume => "control.resume",
            Step { .. } => "control.step",
            Inspect { .. } => "control.inspect",
            PinAdd { .. } => "pins.add",
            PinRemove { .. } => "pins.remove",
            PinRefresh => "pins.refresh",
            PinView { .. } => "pins.view",
            TimelineBookmark { .. } => "timeline.bookmark",
            TimelineRecipe { .. } => "timeline.recipe",
            TimelineCheckpoint { .. } => "timeline.checkpoint",
            Find { settings } => match settings.condition {
                crate::debug_probe::Condition::Pc { .. } => "control.break",
                crate::debug_probe::Condition::Access { .. } => "control.watch",
            },
            Graphics { .. } => "control.graphics",
            UiStart => "control.ui-start",
            UiReport => "control.ui-report",
            AudioStart { .. } => "control.audio-start",
            AudioReport => "control.audio-report",
            CodeStart { .. } => "control.code-start",
            CodeReport => "control.code-report",
            GameplayStart { .. } => "control.gameplay-start",
            GameplayReport => "control.gameplay-report",
            RecentStart { .. } => "control.recent-start",
            RecentFreeze => "control.recent-freeze",
            RecentReport => "control.recent-report",
            IssueReport { .. } => "control.issue-report",
        }
    }
}
/// This mapping does not dispatch. Selection-only actions keep their own IDs.
pub(crate) fn menu_operation(action: &menu::MenuAction) -> Option<&'static str> {
    use menu::MenuAction::*;
    Some(match action {
        ContentOpen => "content.open",
        ChooseToolInput(_) => "content.choose-input",
        OpenDebugTool(tool)=>workbench_operation(*tool),
        ToolRequest(request)=>match request{
            menu::ToolRequest::Validate=>"workbench.validate",menu::ToolRequest::SavePreset=>"workbench.save-preset",menu::ToolRequest::LoadPreset=>"workbench.load-preset",menu::ToolRequest::Prepare=>"workbench.prepare",menu::ToolRequest::Run=>"workbench.run",menu::ToolRequest::Back=>"workbench.back",
        },
        ContentRefresh => "content.scan",
        ContentBrowseFile => "content.browse-file",
        ContentAddFolder => "content.add-folder",
        ContentSelect { .. } => "content.select",
        ContentPreview { .. } => "content.preview",
        ContentCancel => "content.cancel",
        CopyDiagnosis => "diagnose",
        StartDebugSession | StartInterpreterDebugSession => "session.create",
        CloseDebugSession => "session.close",
        OpenDebugCatalog => "catalog.list",
        OpenCheatCatalog | CheatReload => "cheats.open",
        CheatChoose => "cheats.choose",
        CheatManage => "cheats.open",
        CheatRequest(request) => match request {
            menu::CheatRequest::Prepare=>"cheats.prepare-once",menu::CheatRequest::ApplyOnce=>"cheats.apply-once",menu::CheatRequest::Arm=>"cheats.arm",menu::CheatRequest::Disarm|menu::CheatRequest::DisarmAll=>"cheats.disarm",
            menu::CheatRequest::PresetSave=>"cheats.preset-save",menu::CheatRequest::PresetPrepare=>"cheats.preset-prepare",menu::CheatRequest::PresetArm=>"cheats.preset-arm",
        },
        ChooseDebugCatalog => "catalog.choose",
        CatalogSelection(_) => "catalog.select",
        CatalogImport(_) => "catalog.import",
        CatalogOpen(_) => "catalog.open",
        StartInputRecording => "record.start",
        StopInputRecording => "record.stop",
        ReplayCase => "replay.run",
        ChooseReplayProfile => "replay.select",
        CancelReplay => "replay.cancel",
        AuditReplay => "audit.run",
        CaptureReplay => "capture.run",
        CompareReplay => "compare.run",
        DebugStep(_) => "control.step",
        DebugResume => "control.resume",
        InspectPC => "workbench.inspect",
        CaptureGraphics => "workbench.graphics",
        StartUiTrace => "control.ui-start",
        SaveUiReport => "control.ui-report",
        StartAudioCapture => "workbench.audio",
        SaveAudioReport => "control.audio-report",
        StartCodeTrace => "workbench.code",
        SaveCodeReport => "control.code-report",
        StartGameplayTrace => "workbench.gameplay",
        SaveGameplayReport => "control.gameplay-report",
        StartRecentEvents => "workbench.recent",
        SaveRecentEvents => "control.recent-report",
        CreateIssueReport => "workbench.issue",
        SaveState { .. } => "state.save",
        LoadState(_) => "state.load",
        UndoState => "state.undo",
        RestoreBackup(_) => "save.restore",
        SwitchGameSave(_) => "save.switch",
        CopyGameSave { .. } => "save.copy",
        RestartRom => "runtime.restart",
        Resume => "runtime.continue",
        Changed(menu::Changed::Debug) => "saves.extended",
        Quit | ToggleFastForward | ToggleEventConsole | OpenCommandConsole | SwitchRom | ChooseAssetFolder | Changed(_) | InputDevice(_) | ResetGamepad => return None,
    })
}

fn query(args: &[String]) -> Result<Value, String> {
    let args = if args.last().is_some_and(|s| s == "--json") {
        &args[..args.len() - 1]
    } else {
        args
    };
    let operations = match args {
        [] => &registry().operations[..],
        [verb] if verb == "list" => &registry().operations[..],
        [verb, id] if verb == "show" && id.len() <= 96 => {
            std::slice::from_ref(get(id).ok_or_else(|| format!("Unknown debug operation: {id}"))?)
        }
        _ => return Err("Use debug-operations [list | show ID] [--json]".into()),
    };
    Ok(
        json!({"schema":1,"kind":"ffta-debug-operations","outcome":"completed",
        "message":"Readonly operation descriptors; selection does not execute an action.",
        "contract_version":VERSION,"execution":registry().execution,
        "result_contract":registry().result_contract,"operations":operations}),
    )
}
pub(crate) fn cmd(args: &[String]) -> u8 {
    match query(args) {
        Ok(value) => {
            if args.last().is_some_and(|s| s == "--json") {
                println!("{value}");
            } else {
                println!("Debug operation contract v{VERSION} (readonly)");
                for op in value["operations"].as_array().unwrap() {
                    println!(
                        "{}: {}",
                        op["id"].as_str().unwrap(),
                        op["title"].as_str().unwrap()
                    );
                    if value["operations"].as_array().unwrap().len() == 1 {
                        println!("{}", serde_json::to_string_pretty(op).unwrap());
                    }
                }
                println!("Details: debug-operations show ID --json");
            }
            0
        }
        Err(message) => {
            println!(
                "{}",
                json!({"schema":1,"kind":"ffta-debug-operations",
                "contract_version":VERSION,"outcome":"invalid-request","message":message})
            );
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|s| (*s).into()).collect()
    }

    #[test]
    fn atlas_cards_keep_registry_identity_routes_and_parameters() {
        let cards=atlas_references();
        assert_eq!(cards.len(),registry().operations.len());
        for (card,op) in cards.iter().zip(&registry().operations) {
            assert_eq!(card.id,op.id);
            assert_eq!(card.section,menu::clan_atlas::reference_group(&op.menu.destination));
            assert!(card.details.contains(&format!("Route: {}",op.menu.current.as_deref().unwrap_or(&op.menu.destination))));
            assert!(card.details.contains("This card is read-only."));
            for parameter in &op.parameters { assert!(card.details.contains(&parameter.name)); }
        }
    }

    #[test]
    fn contract_ids_routes_parameters_and_source_files_are_consistent() {
        let r = registry();
        assert_eq!((r.schema, r.version), (1, VERSION));
        assert!(r.operations.len() >= 65);
        let mut ids = BTreeSet::new();
        for op in &r.operations {
            assert!(ids.insert(&op.id), "duplicate {}", op.id);
            assert!(
                op.id.len() <= 96
                    && op
                        .id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b == b'.' || b == b'-')
            );
            assert!(
                !op.title.is_empty() && !op.handler.is_empty() && !op.menu.destination.is_empty()
            );
            assert_eq!(op.menu.implemented, op.menu.current.is_some());
            assert!(
                !op.allowed_state.is_empty()
                    && !op.effects.is_empty()
                    && !op.result.is_empty()
                    && !op.limits.is_empty()
            );
            let mut parameters = BTreeSet::new();
            for p in &op.parameters {
                assert!(
                    parameters.insert(&p.name),
                    "duplicate parameter {} in {}",
                    p.name,
                    op.id
                );
                assert!(!p.value_type.is_empty() && !p.constraint.is_empty());
                assert!(
                    ["fixed", "selected", "unavailable", "not-applicable"]
                        .contains(&p.current_menu.as_str())
                );
                if p.current_menu == "unavailable" {
                    assert!(!op.open_work.is_empty(), "untracked gap {}", op.id);
                }
            }
        }
    }

    #[test]
    fn list_show_and_malformed_queries_are_metadata_only() {
        let all = query(&args(&["list", "--json"])).unwrap();
        assert_eq!(
            all["operations"].as_array().unwrap().len(),
            registry().operations.len()
        );
        let one = query(&args(&["show", "control.step", "--json"])).unwrap();
        assert_eq!(one["operations"][0]["id"], "control.step");
        for bad in [
            vec!["run"],
            vec!["show"],
            vec!["show", "missing"],
            vec!["show", "../file"],
            vec!["list", "--execute"],
            vec!["list", "--json", "--json"],
            vec!["--json", "list"],
        ] {
            assert!(query(&args(&bad)).is_err(), "accepted {bad:?}");
        }
        assert_eq!(all, query(&args(&[])).unwrap());
    }

    #[test]
    fn menu_and_control_use_same_ids_without_losing_selection_or_fixed_parameters() {
        use crate::debug_control::Action;
        for (menu, control) in [
            (
                menu::MenuAction::DebugStep(10),
                Action::Step {
                    frames: 10,
                    keys: 0x3ff,
                },
            ),
        ] {
            assert_eq!(menu_operation(&menu), Some(control.operation_id()));
            assert!(get(control.operation_id()).is_some());
        }
        assert_eq!(menu_operation(&menu::MenuAction::StartAudioCapture),Some("workbench.audio"));
        for (action,id) in [(menu::MenuAction::OpenCheatCatalog,"cheats.open"),(menu::MenuAction::CheatReload,"cheats.open"),(menu::MenuAction::CheatChoose,"cheats.choose")]{assert_eq!(menu_operation(&action),Some(id));assert!(get(id).is_some());}
        for id in ["cheats.list","cheats.show","cheats.info"]{assert!(get(id).is_some());}
        assert_ne!(menu_operation(&menu::MenuAction::StartAudioCapture),Some(Action::AudioStart{settings:Default::default()}.operation_id()));
        assert_ne!(
            menu_operation(&menu::MenuAction::ChooseReplayProfile),
            menu_operation(&menu::MenuAction::ReplayCase)
        );
        let op = get("control.step").unwrap();
        assert!(
            op.parameters
                .iter()
                .any(|p| p.name == "--keys" && p.current_menu == "fixed")
        );
        assert!(get("replay.run").unwrap().external_dialog.is_none());
        assert!(registry().operations.iter().filter_map(|o|o.external_dialog.as_ref()).all(|d|!d.automatic));
        assert!(get("replay.create").unwrap().menu.current.is_none());
    }
}

/// Descriptions only: no command string is interpreted by the menu.
pub(crate) fn atlas_references()->Vec<menu::clan_atlas::OperationReference>{
    registry().operations.iter().map(|op|{
        let parameters=op.parameters.iter().map(|p|format!("{} ({}): {} · default {}",p.name,p.value_type,p.constraint,p.default)).collect::<Vec<_>>().join("; ");
        menu::clan_atlas::OperationReference{id:op.id.clone(),title:op.title.clone(),section:menu::clan_atlas::reference_group(&op.menu.destination),details:format!("{} · {}\n{}\nCLI: {}\nRoute: {}\nPrerequisites: {}\nParameters: {}\nEffects: {}\nResult: {}\nLimits: {}\nHandler: {}\nThis card is read-only. Use the menu action or the explicit CLI.",op.id,op.title,if op.menu.implemented {"Menu available · existing handler"}else{"CLI only · no execution from this card"},op.cli.as_deref().unwrap_or("No CLI command"),op.menu.current.as_deref().unwrap_or(&op.menu.destination),op.allowed_state,parameters,op.effects,op.result,op.limits,op.handler)}
    }).collect()
}
