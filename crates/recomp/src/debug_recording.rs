//! Snapshot-relative captures. Incomplete folders are evidence, never valid replays.
use gba_core::Machine;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const MAX_FRAMES: u64 = 360_000;
const MAX_EVENTS: u64 = 200_000;
const MAX_INPUT: u64 = 8 * 1024 * 1024;
const MAX_CAPTURES: usize = 8;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    pub(crate) schema: u32,
    pub(crate) kind: String,
    pub status: String,
    pub(crate) reason: String,
    pub(crate) rom_sha256: String,
    pub(crate) bios_sha256: String,
    pub(crate) executable_sha256: String,
    pub(crate) dataset_manifest_sha256: Option<String>,
    pub(crate) interp: bool,
    pub(crate) first_frame: u64,
    pub(crate) first_clock: u64,
    pub(crate) initial_keys: u16,
    pub frames: u64,
    pub(crate) events: u64,
    pub(crate) last_frame: u64,
    pub(crate) last_clock: u64,
    pub(crate) extra: usize,
    pub(crate) extra_y: usize,
    pub(crate) start_sha256: String,
    pub(crate) input_sha256: Option<String>,
    pub(crate) end_sha256: Option<String>,
}

pub(crate) struct Capture {
    pub directory: PathBuf,
    manifest: Manifest,
    input: File,
    last_keys: u16,
    next_frame: u64,
    bytes: u64,
}

fn publish(path: &Path, bytes: &[u8], replace: bool) -> Result<(), String> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing parent")?)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    if replace {
        file.persist(path)
    } else {
        file.persist_noclobber(path)
    }
    .map_err(|e| e.error.to_string())?;
    Ok(())
}

fn identity(path: &Path, limit: u64) -> Result<String, String> {
    Ok(gba_core::ffta_rom::digest(&crate::diagnostics::read(
        path, limit,
    )?))
}

impl Capture {
    pub fn lifecycle_status(&self)->serde_json::Value{
        serde_json::json!({"first_frame":self.manifest.first_frame,"expected_next_frame":self.next_frame,"input_bytes":self.bytes,"last_keys":self.last_keys,"scope":"Input writer metadata; expected next frame is not proof of a completed frame"})
    }
    pub fn start(
        directory: &Path,
        machine: &Machine,
        interp: bool,
        dataset: Option<&Path>,
        av: &input_config::AvConfig,
        input: &input_config::InputConfig,
    ) -> Result<Self, String> {
        // All serialization/identity checks precede creation; never touch the machine.
        let start = machine.save_state()?;
        let executable_sha256 = identity(
            &std::env::current_exe().map_err(|e| e.to_string())?,
            256 * 1024 * 1024,
        )?;
        let dataset_manifest_sha256 = dataset
            .map(|p| {
                identity(
                    &p.join("manifest.toml"),
                    gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
                )
            })
            .transpose()?;
        let manifest = Manifest {
            schema: 1,
            kind: "ffta-input-recording".into(),
            status: "recording".into(),
            reason: String::new(),
            rom_sha256: gba_core::ffta_rom::digest(&machine.bus.rom),
            bios_sha256: gba_core::ffta_rom::digest(&machine.bus.bios),
            executable_sha256,
            dataset_manifest_sha256,
            interp,
            first_frame: machine.bus.frames,
            first_clock: machine.bus.clock,
            initial_keys: machine.bus.keys & 0x3ff,
            frames: 0,
            events: 0,
            last_frame: machine.bus.frames,
            last_clock: machine.bus.clock,
            extra: gba_core::mem::extra(),
            extra_y: gba_core::mem::extra_y(),
            start_sha256: gba_core::ffta_rom::digest(&start),
            input_sha256: None,
            end_sha256: None,
        };
        fs::create_dir(directory)
            .map_err(|e| format!("Cannot create new recording folder: {e}"))?;
        publish(&directory.join(".gitignore"), b"*\n", false)?;
        publish(&directory.join("start.ffstate"), &start, false)?;
        publish(&directory.join("av.cfg"), av.serialize().as_bytes(), false)?;
        publish(
            &directory.join("input.cfg"),
            input.serialize().as_bytes(),
            false,
        )?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("input.input"))
            .map_err(|e| e.to_string())?;
        let mut capture = Self {
            directory: directory.into(),
            last_keys: manifest.initial_keys,
            next_frame: manifest.first_frame,
            manifest,
            input: file,
            bytes: 0,
        };
        capture.write(b"gba-input v1\n")?;
        capture.event(&format!("0 {:03x}\n", capture.last_keys))?;
        capture.save_manifest()?;
        Ok(capture)
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        if self.bytes + bytes.len() as u64 > MAX_INPUT {
            return Err("Recording reached its 8 MiB limit".into());
        }
        self.input.write_all(bytes).map_err(|e| e.to_string())?;
        self.bytes += bytes.len() as u64;
        Ok(())
    }
    fn event(&mut self, line: &str) -> Result<(), String> {
        if self.manifest.events >= MAX_EVENTS {
            return Err("Recording reached its 200000-event limit".into());
        }
        self.write(line.as_bytes())?;
        self.manifest.events += 1;
        Ok(())
    }
    fn save_manifest(&self) -> Result<(), String> {
        publish(
            &self.directory.join("recording.json"),
            &serde_json::to_vec_pretty(&self.manifest).map_err(|e| e.to_string())?,
            true,
        )
    }
    pub fn before_frame(&mut self, machine: &Machine, mouse: &[u16]) -> Result<(), String> {
        if machine.bus.frames != self.next_frame {
            return Err("Recording timeline changed; start a new capture".into());
        }
        let relative = self
            .next_frame
            .checked_sub(self.manifest.first_frame)
            .ok_or("Recording frame rewind")?;
        if relative >= MAX_FRAMES {
            return Err("Recording reached its 360000-frame limit".into());
        }
        if (gba_core::mem::extra(), gba_core::mem::extra_y())
            != (self.manifest.extra, self.manifest.extra_y)
        {
            return Err("View dimensions changed; start a new recording for this view".into());
        }
        for index in mouse {
            self.event(&format!("{relative} mouse:{index}\n"))?;
        }
        let keys = machine.bus.keys & 0x3ff;
        if keys != self.last_keys {
            self.event(&format!("{relative} {keys:03x}\n"))?;
            self.last_keys = keys;
        }
        self.next_frame = self
            .next_frame
            .checked_add(1)
            .ok_or("Recording frame overflow")?;
        Ok(())
    }
    pub fn finish(
        mut self,
        machine: &Machine,
        reason: &str,
        complete: bool,
    ) -> Result<PathBuf, String> {
        self.input.sync_all().map_err(|e| e.to_string())?;
        self.manifest.reason = reason.chars().take(2048).collect();
        self.manifest.status = "incomplete".into();
        if complete
            && machine.bus.frames == self.next_frame
            && machine.bus.frames >= self.manifest.first_frame
            && (gba_core::mem::extra(), gba_core::mem::extra_y())
                == (self.manifest.extra, self.manifest.extra_y)
        {
            let end = machine.save_state()?;
            publish(&self.directory.join("end.ffstate"), &end, false)?;
            self.manifest.frames = machine.bus.frames - self.manifest.first_frame;
            self.manifest.last_frame = machine.bus.frames;
            self.manifest.last_clock = machine.bus.clock;
            self.manifest.input_sha256 =
                Some(identity(&self.directory.join("input.input"), MAX_INPUT)?);
            self.manifest.end_sha256 = Some(gba_core::ffta_rom::digest(&end));
            self.manifest.status = "complete".into();
        }
        self.save_manifest()?;
        if self.manifest.status != "complete" {
            return Err(format!(
                "Incomplete recording retained: {} ({reason})",
                self.directory.display()
            ));
        }
        Ok(self.directory)
    }
}

pub(crate) fn destination() -> Result<PathBuf, String> {
    let base = input_config::config_root()
        .ok_or("Cannot locate recording storage")?
        .join("recordings");
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    if fs::read_dir(&base).map_err(|e| e.to_string())?.count() >= MAX_CAPTURES {
        return Err(
            "Eight recordings already exist. Move retained recordings before starting another."
                .into(),
        );
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    Ok(base.join(format!("recording-{stamp}-{}", std::process::id())))
}

/// Validate monotonic order rather than silently sorting a damaged new capture.
pub(crate) fn script(bytes: &[u8], frames: u64) -> Result<crate::InputScript, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let mut previous = 0;
    let mut count = 0;
    for line in text.lines().skip(1) {
        let (frame, value) = line.split_once(' ').ok_or("Malformed recording event")?;
        let frame: u64 = frame.parse().map_err(|_| "Invalid recording frame")?;
        if frame < previous
            || frame > frames
            || (count == 0 && (frame != 0 || value.starts_with("mouse:")))
        {
            return Err(
                "Recording events are not ordered from an initial key state at frame zero".into(),
            );
        }
        if !value.starts_with("mouse:")
            && u16::from_str_radix(value, 16).map_err(|_| "Invalid recording keys")? > 0x3ff
        {
            return Err("Recording key mask is outside the GBA keys".into());
        }
        previous = frame;
        count += 1;
        if count > MAX_EVENTS {
            return Err("Recording exceeds event limit".into());
        }
    }
    if count == 0 {
        return Err("Recording has no initial keys".into());
    }
    crate::InputScript::parse(text, "recording input")
}

pub(crate) struct Export {
    pub manifest: Manifest,
    pub start: Vec<u8>,
    pub end: Vec<u8>,
    pub input: Vec<u8>,
}

/// Shared immutable input authentication for focused checks and replay profiles.
pub(crate) fn export(directory: &Path) -> Result<Export, String> {
    let manifest: Manifest = serde_json::from_slice(&crate::diagnostics::read(
        &directory.join("recording.json"),
        64 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    if manifest.schema != 1
        || manifest.kind != "ffta-input-recording"
        || manifest.status != "complete"
        || manifest.frames > MAX_FRAMES
    {
        return Err("Recording is incomplete or incompatible".into());
    }
    let start = crate::diagnostics::read(
        &directory.join("start.ffstate"),
        gba_core::save_state::MAX_STATE_BYTES as u64,
    )?;
    let end = crate::diagnostics::read(
        &directory.join("end.ffstate"),
        gba_core::save_state::MAX_STATE_BYTES as u64,
    )?;
    let input = crate::diagnostics::read(&directory.join("input.input"), MAX_INPUT)?;
    if gba_core::ffta_rom::digest(&start) != manifest.start_sha256
        || Some(gba_core::ffta_rom::digest(&end)) != manifest.end_sha256
        || Some(gba_core::ffta_rom::digest(&input)) != manifest.input_sha256
    {
        return Err("Recording checksum mismatch".into());
    }
    script(&input, manifest.frames)?;
    Ok(Export {
        manifest,
        start,
        end,
        input,
    })
}

/// Focused functional interpreter replay for DBG-04; no benchmark or save writer.
pub(crate) fn verify(directory: &Path, machine: &mut Machine) -> Result<serde_json::Value, String> {
    let manifest: Manifest = serde_json::from_slice(&crate::diagnostics::read(
        &directory.join("recording.json"),
        64 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    if manifest.schema != 1
        || manifest.kind != "ffta-input-recording"
        || manifest.status != "complete"
        || manifest.frames > MAX_FRAMES
    {
        return Err("Recording is incomplete or incompatible".into());
    }
    if !manifest.interp {
        return Err("This focused check requires an interpreter recording; native replay belongs to the selected runtime workflow".into());
    }
    if manifest.rom_sha256 != gba_core::ffta_rom::digest(&machine.bus.rom)
        || manifest.bios_sha256 != gba_core::ffta_rom::digest(&machine.bus.bios)
    {
        return Err("Recording ROM/BIOS mismatch".into());
    }
    if manifest.executable_sha256
        != identity(
            &std::env::current_exe().map_err(|e| e.to_string())?,
            256 * 1024 * 1024,
        )?
    {
        return Err("Recording was made with a different executable".into());
    }
    let Export {
        start, end, input, ..
    } = export(directory)?;
    let input = script(&input, manifest.frames)?;
    // All preflight checks use a disposable machine owned by the caller.
    machine.load_state(&start)?;
    if machine.bus.frames != manifest.first_frame
        || machine.bus.clock != manifest.first_clock
        || machine.bus.keys != manifest.initial_keys
    {
        return Err("Recording snapshot origin mismatch".into());
    }
    if manifest.extra > gba_core::mem::MAX_EXTRA || manifest.extra_y > gba_core::mem::MAX_EXTRA_Y {
        return Err("Invalid recording view".into());
    }
    struct View(usize, usize);
    impl Drop for View {
        fn drop(&mut self) {
            gba_core::mem::set_extra(self.0);
            gba_core::mem::set_extra_y(self.1);
        }
    }
    let _view = View(gba_core::mem::extra(), gba_core::mem::extra_y());
    gba_core::mem::set_extra(manifest.extra);
    gba_core::mem::set_extra_y(manifest.extra_y);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    for frame in 0..manifest.frames {
        if std::time::Instant::now() > deadline {
            return Err("Recording verification timeout (120 seconds)".into());
        }
        machine.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut machine.bus, frame)?;
        let before = machine.bus.frames;
        machine.run_frame(5_000_000);
        if machine.bus.frames != before + 1 {
            return Err("Recording replay incomplete frame".into());
        }
        // Match the normal player's host consumption, which does not advance clocks.
        machine.bus.audio_buf.clear();
        machine.bus.psg_tap.clear();
        machine.bus.hle_tap.clear();
        machine.bus.fifo_tap[0].clear();
        machine.bus.fifo_tap[1].clear();
    }
    let actual = machine.save_state()?;
    let ok = actual == end
        && machine.bus.frames == manifest.last_frame
        && machine.bus.clock == manifest.last_clock;
    Ok(
        serde_json::json!({"schema":1,"kind":"ffta-recording-verification","ok":ok,"frames":manifest.frames,
        "first_frame":manifest.first_frame,"last_frame":machine.bus.frames,"clock":machine.bus.clock,
        "state_sha256":gba_core::ffta_rom::digest(&actual),"expected_state_sha256":manifest.end_sha256}),
    )
}

pub(crate) fn cmd(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("start") if args.len() == 2 => {
            let root = fs::canonicalize(&args[1]).map_err(|e| e.to_string())?;
            let session = crate::debug_session::open(&root)?;
            if session.state().is_none() {
                return Err(
                    "Recording start needs an isolated session with a snapshot seed".into(),
                );
            }
            drop(session);
            let status = crate::debug_session::launch_recording(&root)?
                .wait()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("Recording session failed: {status}"));
            }
            Ok(())
        }
        Some("verify") if args.len() == 4 && args[2] == "--session" => {
            let session = crate::debug_session::open(Path::new(&args[3]))?;
            let rom = crate::diagnostics::read(&session.rom(), 32 * 1024 * 1024)?;
            let bios = session
                .bios()
                .map(|p| crate::diagnostics::read(&p, 16384))
                .transpose()?;
            let directory = Path::new(&args[1]);
            let manifest: Manifest = serde_json::from_slice(&crate::diagnostics::read(
                &directory.join("recording.json"),
                64 * 1024,
            )?)
            .map_err(|e| e.to_string())?;
            let dataset = session.dataset();
            if dataset
                .as_ref()
                .map(|p| {
                    identity(
                        &p.join("manifest.toml"),
                        gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
                    )
                })
                .transpose()?
                != manifest.dataset_manifest_sha256
            {
                return Err("Recording dataset mismatch".into());
            }
            let mut machine = crate::make_machine(rom, bios.as_deref());
            if let Some(dataset) = dataset {
                crate::rom_data::attach_from_directory(&mut machine, &dataset)?;
            }
            let report = verify(directory, &mut machine)?;
            println!("{}", report);
            if report["ok"] == true {
                Ok(())
            } else {
                Err("Recording endpoint differs from the captured state/clock".into())
            }
        }
        _ => Err(
            "Use debug-record start SESSION or debug-record verify RECORDING --session SESSION"
                .into(),
        ),
    }
}

#[cfg(test)]
mod tests;
