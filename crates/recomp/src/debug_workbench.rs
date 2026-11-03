//! Menu drafts and presets over the existing CLI parsers and pinned input service.
use crate::{content_picker::Picker, debug_control::Action};
use menu::{
    ContentSlot as Slot, DebugTool as Tool, ToolField as Field, ToolFieldKind as Kind, ToolRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    thread::JoinHandle,
    time::{SystemTime, UNIX_EPOCH},
};
type Values = BTreeMap<String, String>;
fn field(key: &str, value: &str, kind: Kind, help: &str) -> Field {
    Field {
        key: key.into(),
        label: key.trim_start_matches('-').replace('-', " "),
        value: value.into(),
        kind,
        help: help.into(),
    }
}
fn text(key: &str, value: &str, help: &str) -> Field {
    field(key, value, Kind::Text, help)
}
fn choice(key: &str, value: &str, options: &[&str], help: &str) -> Field {
    field(
        key,
        value,
        Kind::Choice(options.iter().map(|v| (*v).into()).collect()),
        help,
    )
}
fn input(slot: Slot) -> Field {
    Field{key:format!("input-{slot:?}"),label:slot.label().into(),value:"Not selected".into(),kind:Kind::Input(slot),help:"Choose an internal reference. Selection is separate from execution; identity is rechecked before use.".into()}
}
pub(crate) fn fields(tool: Tool) -> Vec<Field> {
    let mut f = if tool.creates_profile() {
        vec![
            input(Slot::Recording),
            input(Slot::Session),
            input(Slot::RuntimeB),
            input(Slot::Library),
            choice(
                "--native",
                "off",
                &["off", "on"],
                "On requires the selected native library; capture remains interpreter only.",
            ),
            text(
                "--case",
                "Debug case",
                "Printable case label, 1..64 UTF-8 bytes.",
            ),
            text(
                "--timeout-ms",
                "120000",
                "Supervisor timeout 1..120000 ms; no automatic start.",
            ),
        ]
    } else {
        Vec::new()
    };
    f.extend(match tool {
        Tool::Homepage => vec![input(Slot::Session), input(Slot::HostRuntime), input(Slot::HostTool), choice("--source","embedded",&["embedded","external"],"External requires exact declared script and sibling dependency hashes; embedded ignores any previous override."), text("--timeout-ms","120000","Total host deadline20..120000ms; explicit cancel keeps logs.")],
        Tool::CampaignReport => vec![input(Slot::Campaign), input(Slot::HostRuntime), input(Slot::HostTool), choice("--source","embedded",&["embedded","external"],"External requires exact declared script and sibling dependency hashes; embedded ignores any previous override."), text("--case","","One required explicit campaign case ID; no replay or default all."), text("--timeout-ms","120000","Total host deadline20..120000ms.")],
        Tool::ArtifactPreview => vec![input(Slot::Session), input(Slot::HostRuntime), input(Slot::HostTool), choice("--source","embedded",&["embedded","external"],"External requires exact declared script and sibling dependency hashes; embedded ignores any previous override."), text("--timeout-ms","120000","Readonly preview on a closed session; no artifact deletion/apply.")],
        Tool::Recipe => vec![input(Slot::Recipe), input(Slot::Session), input(Slot::RuntimeB), text("--timeout-ms","120000","Total recipe deadline20..120000ms;16readonly status/inspect steps maximum.")],
        Tool::GuestScripts => vec![input(Slot::GuestScripts)],
        Tool::Replay => Vec::new(),
        Tool::Audit => vec![
            input(Slot::Dataset),
            input(Slot::KnownPlan),
            choice(
                "--use-dataset",
                "off",
                &["off", "on"],
                "Apply selected dataset only to the NEW replay worker.",
            ),
            choice(
                "--use-known",
                "off",
                &["off", "on"],
                "Compare reads to the selected authenticated resource plan.",
            ),
            choice(
                "--preset",
                "all",
                &["all", "bus", "host"],
                "Observed source classes; missing hooks remain unknown.",
            ),
            text("--from", "0", "Relative start frame inclusive."),
            text(
                "--to",
                "",
                "Relative end exclusive; empty uses the recorded limit.",
            ),
            text(
                "--ranges",
                "",
                "Optional comma-separated START:END physical ranges; at most32; decimal or 0xhex.",
            ),
            text("--limit", "20000", "1..50000 distinct audit events."),
        ],
        Tool::Capture => vec![
            input(Slot::Dataset),
            input(Slot::KnownPlan),
            choice(
                "--use-dataset",
                "off",
                &["off", "on"],
                "Selected dataset for a new worker; never attaches to current Machine.",
            ),
            choice(
                "--trigger",
                "frame",
                &["frame", "instruction", "resource"],
                "Exactly one condition is emitted; inactive trigger fields are ignored.",
            ),
            text(
                "--frame",
                "1",
                "Completed relative frame trigger, 1..recording length.",
            ),
            text("--limit-frames", "", "Empty uses all recorded frames."),
            text("--pc", "08000000", "Aligned ROM PC hexadecimal."),
            choice("--mode", "arm", &["arm", "thumb"], "Instruction mode."),
            text(
                "--opcode",
                "eafffffe",
                "Expected ROM opcode hexadecimal; checked by trigger preflight.",
            ),
            text(
                "--resource",
                "",
                "Exact authenticated resource ID from the selected plan.",
            ),
            choice(
                "--preset",
                "all",
                &["all", "bus", "host"],
                "Source observer class for resource triggers.",
            ),
        ],
        Tool::Compare => vec![
            input(Slot::Dataset),
            choice(
                "--use-dataset",
                "off",
                &["off", "on"],
                "Candidate dataset only; raw interpreter reference remains independent.",
            ),
            text(
                "--frames",
                "",
                "1..min(recorded frames,3600); empty chooses that limit.",
            ),
            choice(
                "--backend",
                "interp",
                &["interp", "native"],
                "Native requires native=on and a compatible selected library.",
            ),
            choice(
                "--audio",
                "preserve",
                &["preserve", "arm-hle"],
                "Explicit replay audio semantics, recorded in the profile.",
            ),
        ],
        Tool::Step => vec![
            text("frames", "1", "Complete1..600frames and return to pause."),
            text(
                "--keys",
                "3ff",
                "Active-low hexadecimal GBA key mask000..3ff; input is explicit.",
            ),
        ],
        Tool::Inspect => vec![
            text(
                "address",
                "08000000",
                "Readonly backing address, hexadecimal; no guest bus I/O.",
            ),
            text("length", "16", "1..256bytes without address wrap."),
        ],
        Tool::Audio => vec![
            text(
                "--frames",
                "60",
                "Capture1..180completedframes; starting only arms the observer.",
            ),
            text(
                "--streams",
                "mixed,psg,hle,fifo0,fifo1",
                "1..5distinct streams; comma-separated mixed,psg,hle,fifo0,fifo1.",
            ),
        ],
        Tool::Code => vec![
            input(Slot::Labels),
            text("--frames", "1", "1..60completedframes."),
            text("--events", "512", "1..512events."),
            text("--start", "0", "Inclusive hexadecimal PC filter."),
            text("--end", "ffffffff", "Exclusive hexadecimal PC filter."),
            choice(
                "--kind",
                "all",
                &["all", "native", "missing", "system", "hook", "interpreter"],
                "Dispatch source class; native/interpreter scopes remain distinct.",
            ),
            choice(
                "--mode",
                "all",
                &["all", "arm", "thumb"],
                "Instruction mode filter.",
            ),
        ],
        Tool::Gameplay => vec![
            text(
                "--frames",
                "1",
                "1..60completedframes; requires authenticated supported ROM models.",
            ),
            text("--events", "512", "1..512events."),
            choice(
                "--family",
                "all",
                &["all", "task", "script", "unit"],
                "Available authenticated task/script/unit contracts.",
            ),
            text(
                "--context",
                "",
                "Optional RAM context pointer hexadecimal; empty means all.",
            ),
        ],
        Tool::Recent => vec![
            text("--events", "1024", "1..1024retained events."),
            choice(
                "--reads",
                "on",
                &["on", "off"],
                "Readonly ROM observer; requires available read-audit hooks.",
            ),
            text(
                "--start",
                "0",
                "Inclusive physical ROM filter, hexadecimal.",
            ),
            text(
                "--end",
                "02000000",
                "Exclusive physical ROM filter up to02000000.",
            ),
            choice(
                "--on-error",
                "on",
                &["on", "off"],
                "Freeze/save metadata on a supported host error.",
            ),
        ],
        Tool::Graphics => vec![choice(
            "--compare-last",
            "off",
            &["off", "on"],
            "Compare to the last compatible captured graphics report; no emulation step.",
        )],
        Tool::Issue => vec![
            input(Slot::Replay),
            text(
                "--symptom",
                "",
                "At most2048UTF-8bytes; kept private in the issue bundle. No automatic send.",
            ),
        ],
    });
    f
}
fn defaults(tool: Tool) -> Values {
    fields(tool)
        .into_iter()
        .filter(|f| !matches!(f.kind, Kind::Input(_)))
        .map(|f| (f.key, f.value))
        .collect()
}
fn checked_values(tool: Tool, values: Values) -> Result<Values, String> {
    let schema = fields(tool);
    let expected = defaults(tool);
    if values.len() != expected.len()
        || values.keys().any(|k| !expected.contains_key(k))
        || values
            .values()
            .any(|v| v.len() > 2048 || v.chars().any(|c| c.is_control() && c != '\n' && c != '\t'))
    {
        return Err("Invalid or oversized tool draft fields".into());
    }
    if serde_json::to_vec(&values)
        .map_err(|e| e.to_string())?
        .len()
        > 8192
    {
        return Err("Tool draft exceeds8KiB".into());
    }
    for f in schema {
        if let Kind::Choice(options) = f.kind {
            if !options.contains(&values[&f.key]) {
                return Err(format!("Invalid {} choice", f.label));
            }
        }
    }
    Ok(values)
}
pub(crate) struct Draft {
    pub tool: Tool,
    pub values: Values,
    pub refs: BTreeMap<Slot, crate::debug_inputs::Reference>,
    pub rom: String,
    pub config: PathBuf,
}
impl Draft {
    pub(crate) fn value(&self, key: &str) -> &str {
        self.values.get(key).map(String::as_str).unwrap_or("")
    }
    pub(crate) fn path(&self, slot: Slot) -> Result<PathBuf, String> {
        self.refs
            .get(&slot)
            .ok_or_else(|| format!("Select {} first", slot.label()))?
            .recheck(slot, &self.rom)
    }
    pub(crate) fn optional(&self, slot: Slot) -> Result<Option<PathBuf>, String> {
        self.refs.get(&slot).map(|_| self.path(slot)).transpose()
    }
    pub fn control(&self) -> Result<Action, String> {
        let command = match self.tool {
            Tool::Step => "step",
            Tool::Inspect => "inspect",
            Tool::Audio => "audio-start",
            Tool::Code => "code-start",
            Tool::Gameplay => "gameplay-start",
            Tool::Recent => "recent-start",
            Tool::Graphics => "graphics",
            Tool::Issue => "issue-report",
            _ => return Err("This draft prepares a profile".into()),
        };
        let mut args = vec![command.into(), "selected-session".into()];
        match self.tool {
            Tool::Step => args.extend([
                self.value("frames").into(),
                "--keys".into(),
                self.value("--keys").into(),
            ]),
            Tool::Inspect => {
                args.extend([self.value("address").into(), self.value("length").into()])
            }
            Tool::Graphics => {
                if self.value("--compare-last") == "on" {
                    args.push("--compare-last".into());
                }
            }
            Tool::Issue => {
                args.extend([
                    "--profile".into(),
                    self.path(Slot::Replay)?.to_string_lossy().into_owned(),
                    "--symptom".into(),
                    self.value("--symptom").into(),
                ]);
            }
            _ => {
                for (k, v) in &self.values {
                    if !v.is_empty() {
                        args.extend([k.clone(), v.clone()]);
                    }
                }
            }
        }
        let action = crate::debug_control::parse(&args)?.1;
        if self.tool == Tool::Code {
            self.optional(Slot::Labels)?;
        }
        if !cfg!(feature = "rom-data-read-audit")
            && self.tool == Tool::Recent
            && self.value("--reads") == "on"
        {
            return Err("Selected build lacks rom-data-read-audit; turn reads off or choose a supported runtime".into());
        }
        Ok(action)
    }
    pub fn profile_args(&self, destination: &Path) -> Result<Vec<String>, String> {
        let recording = self.path(Slot::Recording)?;
        let session = self.path(Slot::Session)?;
        let runtime = self
            .optional(Slot::RuntimeB)?
            .unwrap_or(std::env::current_exe().map_err(|e| e.to_string())?);
        let mut args = vec![
            "--recording".into(),
            recording.to_string_lossy().into_owned(),
            "--session".into(),
            session.to_string_lossy().into_owned(),
            "--dir".into(),
            destination.to_string_lossy().into_owned(),
            "--runtime".into(),
            runtime.to_string_lossy().into_owned(),
            "--case".into(),
            self.value("--case").into(),
            "--timeout-ms".into(),
            self.value("--timeout-ms").into(),
        ];
        if self.value("--native") == "on" {
            args.extend([
                "--library".into(),
                self.path(Slot::Library)?.to_string_lossy().into_owned(),
            ]);
        }
        if matches!(self.tool, Tool::Audit | Tool::Capture)
            && !cfg!(feature = "rom-data-read-audit")
        {
            return Err("Selected build lacks rom-data-read-audit; cannot prepare this observer profile here.".into());
        }
        let mut push = |key: &str, value: &str| {
            if !value.is_empty() {
                args.extend([key.into(), value.into()]);
            }
        };
        if self.value("--use-dataset") == "on" {
            push("--dataset", &self.path(Slot::Dataset)?.to_string_lossy());
        }
        match self.tool {
            Tool::Replay => {}
            Tool::Audit => {
                for k in ["--preset", "--from", "--to", "--limit"] {
                    push(k, self.value(k));
                }
                if self.value("--use-known") == "on" {
                    let mut p = self.path(Slot::KnownPlan)?;
                    if p.is_dir() {
                        p = p.join("manifest.toml");
                    }
                    push("--known-manifest", &p.to_string_lossy());
                }
                let ranges = self.value("--ranges");
                if !ranges.is_empty() {
                    for r in ranges.split(',') {
                        push("--range", r.trim());
                    }
                }
            }
            Tool::Capture => {
                push("--limit-frames", self.value("--limit-frames"));
                match self.value("--trigger") {
                    "frame" => push("--frame", self.value("--frame")),
                    "instruction" => {
                        for k in ["--pc", "--mode", "--opcode"] {
                            push(k, self.value(k));
                        }
                    }
                    "resource" => {
                        push("--resource", self.value("--resource"));
                        push("--preset", self.value("--preset"));
                        let mut p = self.path(Slot::KnownPlan)?;
                        if p.is_dir() {
                            p = p.join("manifest.toml");
                        }
                        push("--manifest", &p.to_string_lossy());
                    }
                    _ => return Err("Unsupported trigger".into()),
                }
            }
            Tool::Compare => {
                for k in ["--frames", "--backend", "--audio"] {
                    push(k, self.value(k));
                }
            }
            _ => return Err("Not a profile preparation draft".into()),
        }
        Ok(args)
    }
    pub fn profile(&self, destination: &Path, verify: bool) -> Result<PathBuf, String> {
        let args = self.profile_args(destination)?;
        match self.tool {
            Tool::Replay => crate::debug_replay::create_dispatch(&args, None, None, None, verify),
            Tool::Audit => crate::debug_rom_audit::create_from_menu(&args, verify),
            Tool::Capture => crate::debug_capture::create_from_menu(&args, verify),
            Tool::Compare => crate::debug_compare::create_from_menu(&args, verify),
            _ => Err("Not a profile tool".into()),
        }
    }
    fn validate(&self) -> Result<String, String> {
        if self.tool.library() { return crate::debug_tools::details(self); }
        if self.tool.creates_profile() {
            let destination = self
                .config
                .join("debug-profiles")
                .join("verify-only-never-created");
            self.profile(&destination, true)?;
            Ok(format!("Valid pinned input and parameters. CLI argv: {:?}. No profile created or replay started; worker runtime features/ABI still require execution preflight.",self.profile_args(&destination)?))
        } else {
            Ok(format!("Validated by shared CLI parser: {:?}. Start is a separate action; current session/busy/model constraints remain checked by its handler.",self.control()?))
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preset {
    schema: u32,
    kind: String,
    tool: String,
    values: Values,
}
fn tool_key(tool: Tool) -> String {
    format!("{tool:?}").to_ascii_lowercase()
}
fn preset_base(config: &Path, tool: Tool) -> PathBuf {
    config.join("debug-presets").join(tool_key(tool))
}
fn ordinary_tree(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        if current.exists() {
            crate::debug_content::ordinary(&current)?;
        }
    }
    Ok(())
}
fn stamp() -> Result<String, String> {
    Ok(format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos(),
        std::process::id()
    ))
}
fn save_preset(d: &Draft) -> Result<String, String> {
    let base = preset_base(&d.config, d.tool);
    ordinary_tree(&base)?;
    if base.exists() && fs::read_dir(&base).map_err(|e| e.to_string())?.count() >= 8 {
        return Err(
            "Eight presets for this tool already exist; retain/archive before saving another."
                .into(),
        );
    }
    let preset = Preset {
        schema: 1,
        kind: "ffta-debug-tool-preset".into(),
        tool: tool_key(d.tool),
        values: d.values.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&preset).map_err(|e| e.to_string())?;
    if bytes.len() > 16384 {
        return Err("Preset exceeds16KiB".into());
    }
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let name = format!("{}.json", stamp()?);
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(base.join(&name))
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "Saved new {} preset {name}; selected input references were not persisted or executed.",
        d.tool.label()
    ))
}
fn load_preset(config: &Path, tool: Tool) -> Result<Values, String> {
    let base = preset_base(config, tool);
    ordinary_tree(&base)?;
    let mut paths = fs::read_dir(&base)
        .map_err(|e| e.to_string())?
        .take(9)
        .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if paths.len() > 8 {
        return Err("Preset directory exceeds8entries".into());
    }
    paths.sort();
    let path = paths.last().ok_or("No saved preset for this tool")?;
    crate::debug_content::ordinary(path)?;
    let p: Preset = serde_json::from_slice(&crate::diagnostics::read(path, 16384)?)
        .map_err(|e| e.to_string())?;
    if p.schema != 1 || p.kind != "ffta-debug-tool-preset" || p.tool != tool_key(tool) {
        return Err("Preset schema/tool mismatch; draft preserved".into());
    }
    checked_values(tool, p.values)
}
enum Output {
    Message(String),
    Observed(crate::debug_tools::RunOutput),
    Prepared(PathBuf),
    Loaded(Values),
}
struct Job {
    tool: Tool,
    request: ToolRequest,
    revision: u64,
    view: u64,
    handle: JoinHandle<Result<Output, String>>,
}
#[derive(Default)]
pub(crate) struct Workbench {
    view: u64,
    drafts: BTreeMap<Tool, Values>,
    job: Option<Job>,
    cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub prepared: Option<(Tool, PathBuf)>,
    pub last: String,
    pub observation: Option<Value>,
}
impl Workbench {
    pub fn open(&mut self, menu: &mut menu::Menu, tool: Tool) {
        self.remember(menu);
        self.view = self.view.wrapping_add(1);
        let values = self.drafts.entry(tool).or_insert_with(|| defaults(tool));
        let mut f = fields(tool);
        for field in &mut f {
            if let Some(v) = values.get(&field.key) {
                field.value = v.clone();
            }
        }
        menu.open_tool(
            tool,
            f,
            "Edit a draft; Verify does not run. Prepare creates a NEW profile; Start is explicit."
                .into(),
        );
        let summary=if tool.library() {crate::debug_tools::description(tool)} else {self.summary()};
        menu.tool_feedback(self.last.clone(), summary, self.job.is_some());
    }
    pub fn remember(&mut self, menu: &menu::Menu) {
        let (t, v, _) = menu.tool_values();
        if let Ok(values) = checked_values(t, v.into_iter().collect()) {
            self.drafts.insert(t, values);
        }
    }
    pub fn draft(
        &mut self,
        menu: &menu::Menu,
        picker: &Picker,
        rom: String,
    ) -> Result<Draft, String> {
        self.remember(menu);
        let (t, v, _) = menu.tool_values();
        Ok(Draft {
            tool: t,
            values: checked_values(t, v.into_iter().collect())?,
            refs: picker.references(),
            rom,
            config: input_config::config_root().ok_or("No selected configuration root")?,
        })
    }
    pub fn request(
        &mut self,
        menu: &mut menu::Menu,
        draft: Draft,
        r: ToolRequest,
    ) -> Result<(), String> {
        if self.job.is_some() {
            return Err("A workbench host operation is still finishing".into());
        }
        let (_, _, revision) = menu.tool_values();
        let tool = draft.tool;
        let cancel=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        if r == ToolRequest::Run && tool.library() { self.cancel=Some(cancel.clone()); }
        let task = move || match r {
            ToolRequest::Run if tool.library() => crate::debug_tools::run_observed(draft,cancel).map(Output::Observed),
            ToolRequest::LoadPreset => load_preset(&draft.config, tool).map(Output::Loaded),
            ToolRequest::SavePreset => {
                draft.validate()?;
                save_preset(&draft).map(Output::Message)
            }
            ToolRequest::Prepare if tool.creates_profile() => {
                let parent = draft.config.join("debug-profiles");
                ordinary_tree(&parent)?;
                if parent.exists() && fs::read_dir(&parent).map_err(|e| e.to_string())?.count() >= 8
                {
                    return Err("Eight prepared profiles already exist; retain/archive before preparing another.".into());
                }
                let dest = parent.join(stamp()?);
                draft.profile(&dest, true)?;
                fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
                draft.profile(&dest, false).map(Output::Prepared)
            }
            ToolRequest::Validate | ToolRequest::Prepare => draft.validate().map(Output::Message),
            _ => Err("Use the explicit Start handler".into()),
        };
        self.observation=None;
        self.job = Some(Job {
            view: self.view,
            tool,
            request:r,
            revision,
            handle: std::thread::spawn(task),
        });
        menu.tool_feedback(
            "Checking selected host inputs; no guest execution.".into(),
            String::new(),
            true,
        );
        Ok(())
    }
    pub fn poll(&mut self, menu: &mut menu::Menu) -> Option<PathBuf> {
        if !self.job.as_ref().is_some_and(|j| j.handle.is_finished()) {
            return None;
        }
        let job = self.job.take().unwrap();
        self.cancel=None;
        let result = job
            .handle
            .join()
            .unwrap_or_else(|_| Err("Workbench worker failed".into()));
        let mut prepared = None;
        let current = menu.tool_values();
        let same = self.view == job.view
            && menu.tool_active()
            && current.0 == job.tool
            && current.2 == job.revision;
        let details=match &result {
            Ok(Output::Observed(result))=>result.observation.clone(),
            Ok(Output::Prepared(path))=>json!({"outcome":"completed","profile":path.to_string_lossy().chars().take(1024).collect::<String>(),"guest_execution":false}),
            Ok(_)=>json!({"outcome":"completed","guest_execution":false}),
            Err(error)=>json!({"outcome":"refused","error":error.chars().take(1024).collect::<String>(),"guest_execution":false}),
        };
        self.observation=Some(json!({"tool":format!("{:?}",job.tool),"request":format!("{:?}",job.request),"view":job.view,"revision":job.revision,"current_draft":same,"outcome":details["outcome"],"result":details}));
        self.last = match result {
            Ok(Output::Message(m)) => m,
            Ok(Output::Observed(result)) => result.message,
            Ok(Output::Prepared(p)) => {
                self.prepared = Some((job.tool, p.clone()));
                if same {
                    prepared = Some(p);
                }
                "NEW profile prepared and pinned. No replay started. Use Run prepared profile explicitly.".into()
            }
            Ok(Output::Loaded(v)) => {
                if same {
                    self.drafts.insert(job.tool, v.clone());
                    let mut f = fields(job.tool);
                    for field in &mut f {
                        if let Some(v) = v.get(&field.key) {
                            field.value = v.clone();
                        }
                    }
                    menu.open_tool(job.tool, f, String::new());
                    "Preset loaded as draft; inputs and execution unchanged. Verify before starting.".into()
                } else {
                    "Preset read completed for an old draft; current fields preserved.".into()
                }
            }
            Err(e) => format!("Refused: {e}"),
        };
        if menu.tool_active() {
            menu.tool_feedback(
                self.last.clone(),
                if same {
                    self.summary()
                } else {
                    "Result belongs to an earlier draft; current parameters are unchanged.".into()
                },
                false,
            );
        }
        prepared
    }
    fn summary(&self) -> String {
        let mut s="Input references are rechecked on every action. Empty optional fields keep CLI defaults.".to_owned();
        if let Some((tool, path)) = &self.prepared {
            s.push_str(&format!("\n\nPrepared {tool:?}: {}\nRun uses this separately pinned profile, including its saved parameters. Draft edits require Prepare NEW to change it.",path.display()));
        }
        s
    }
    pub fn cancel_process(&mut self) -> Result<(),String> {
        let c=self.cancel.as_ref().ok_or("No host process is running")?;
        c.store(true,std::sync::atomic::Ordering::SeqCst);
        self.last="Cancellation requested; retain logs and wait for confirmed result.".into();
        Ok(())
    }
    pub fn cancel_view(&mut self) {
        self.view = self.view.wrapping_add(1);
    }
    pub fn busy(&self) -> bool {
        self.job.is_some()
    }
    pub fn prepared_for(&self, tool: Tool) -> Result<PathBuf, String> {
        self.prepared
            .as_ref()
            .filter(|(t, _)| *t == tool)
            .map(|(_, p)| p.clone())
            .ok_or("Prepare a NEW profile for this tool first".into())
    }
    pub fn status(&self) -> Value {
        json!({"busy":self.job.is_some(),"prepared":self.prepared.as_ref().map(|(tool,path)|json!({"tool":format!("{tool:?}"),"path":path})),"last":self.last,"drafts":self.drafts.len(),"host_cancel_requested":self.cancel.as_ref().map(|c|c.load(std::sync::atomic::Ordering::SeqCst)),"scope":"Host drafts/presets; validation and preparation never start guest execution; explicit Start uses existing handlers"})
    }
}

impl Drop for Workbench {
    fn drop(&mut self) {
        if let Some(c)=&self.cancel {c.store(true,std::sync::atomic::Ordering::SeqCst);}
        if self.cancel.is_some() {if let Some(job)=self.job.take(){let _=job.handle.join();}}
    }
}
#[cfg(test)]
mod tests;
