use crate::config::{Config, SEED_DOCUMENT_TYPES};
use crate::error::AppError;
use crate::sanitize::{sanitize_name, sanitize_path_string};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, ErrorKind, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct InitOptions {
    pub name: String,
    pub no_git: bool,
    pub force: bool,
}

/// Core artifacts are always active and are never part of the selectable list.
const CORE_ARTIFACTS: [&str; 3] = ["task", "milestone", "release"];
/// Decision types in configuration key order.
const DECISION_ARTIFACTS: [&str; 2] = ["adr", "bdr"];
/// Document types in schema (configuration declaration) order. Derived from
/// the single production definition, [`crate::config::SEED_DOCUMENT_TYPES`], so
/// this module holds no second document-slug list (ADR-0020 decision 1
/// clarification, 2026-11-23).
const DOCUMENT_ARTIFACTS: [&str; SEED_DOCUMENT_TYPES.len()] = document_artifacts_from_seed();

/// Build the selectable document order from the seed table in declaration
/// order. `const` so an add, remove, reorder, or rename on either side is a
/// compile-time and test-time failure.
const fn document_artifacts_from_seed() -> [&'static str; SEED_DOCUMENT_TYPES.len()] {
    let mut artifacts = [""; SEED_DOCUMENT_TYPES.len()];
    let mut index = 0;
    while index < SEED_DOCUMENT_TYPES.len() {
        artifacts[index] = SEED_DOCUMENT_TYPES[index].slug;
        index += 1;
    }
    artifacts
}
const SELECTABLE_ARTIFACT_COUNT: usize = DECISION_ARTIFACTS.len() + DOCUMENT_ARTIFACTS.len();

type Activation = (BTreeMap<String, bool>, BTreeMap<String, bool>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Enter,
    Toggle,
    Up,
    Down,
    Eof,
    Interrupt,
    Ignored,
}

/// Injectable keyboard seam: the list state machine never reads stdin directly.
trait KeySource {
    fn next_key(&mut self) -> io::Result<Key>;
}

/// Real adapter: reads one byte at a time from the same `BufRead` the later
/// prefix/folder prompts use, so buffered typeahead is preserved.
struct ByteKeySource<'a, R: BufRead> {
    reader: &'a mut R,
}

impl<'a, R: BufRead> ByteKeySource<'a, R> {
    fn new(reader: &'a mut R) -> Self {
        Self { reader }
    }

    fn read_byte(&mut self) -> io::Result<Option<u8>> {
        let mut byte = [0u8; 1];
        loop {
            match self.reader.read_exact(&mut byte) {
                Ok(()) => return Ok(Some(byte[0])),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e),
            }
        }
    }

    fn read_escape(&mut self) -> io::Result<Key> {
        // A lone ESC or an unrecognised sequence is an ignored key. Read at
        // most two further bytes so escape state cannot desynchronise.
        let Some(second) = self.read_byte()? else {
            return Ok(Key::Ignored);
        };
        if second != b'[' && second != b'O' {
            return Ok(Key::Ignored);
        }
        match self.read_byte()? {
            Some(b'A') => Ok(Key::Up),
            Some(b'B') => Ok(Key::Down),
            _ => Ok(Key::Ignored),
        }
    }
}

impl<R: BufRead> KeySource for ByteKeySource<'_, R> {
    fn next_key(&mut self) -> io::Result<Key> {
        let Some(byte) = self.read_byte()? else {
            return Ok(Key::Eof);
        };
        Ok(match byte {
            0x0A | 0x0D => Key::Enter,
            0x20 => Key::Toggle,
            b'k' => Key::Up,
            b'j' => Key::Down,
            0x04 => Key::Eof,
            0x03 => Key::Interrupt,
            0x1B => self.read_escape()?,
            _ => Key::Ignored,
        })
    }
}

/// Restores the terminal mode when dropped, so panics and early returns do not
/// leave the terminal in cbreak mode.
trait TerminalModeGuard {
    fn restore(&mut self);
}

/// Injectable terminal-mode seam: the real adapter shells out to `stty`; tests
/// use a fake that can fail without touching a terminal.
trait TerminalMode {
    fn enter(&self) -> io::Result<Box<dyn TerminalModeGuard>>;
}

struct SttyTerminalMode;

fn restore_stty(saved: &str) -> bool {
    Command::new("stty")
        .arg(saved)
        .stdin(Stdio::inherit())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

struct SttyModeGuard {
    saved: Option<String>,
}

impl SttyModeGuard {
    fn restore_inner(&mut self) {
        if let Some(saved) = self.saved.take()
            && !restore_stty(&saved)
        {
            eprintln!("warning: could not restore terminal settings; run `stty sane`");
        }
    }
}

impl TerminalModeGuard for SttyModeGuard {
    fn restore(&mut self) {
        self.restore_inner();
    }
}

impl Drop for SttyModeGuard {
    fn drop(&mut self) {
        self.restore_inner();
    }
}

impl TerminalMode for SttyTerminalMode {
    fn enter(&self) -> io::Result<Box<dyn TerminalModeGuard>> {
        let output = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("stty -g failed"));
        }
        let saved = String::from_utf8(output.stdout)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "stty -g returned invalid UTF-8"))?
            .trim()
            .to_string();
        if saved.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "stty -g returned no settings",
            ));
        }

        let status = Command::new("stty")
            .args([
                "-icanon", "-echo", "-isig", "-ixon", "min", "1", "time", "0",
            ])
            .stdin(Stdio::inherit())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            // Best effort: do not leave a partially modified terminal behind.
            restore_stty(&saved);
            return Err(io::Error::other("stty mode change failed"));
        }

        Ok(Box::new(SttyModeGuard { saved: Some(saved) }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListOutcome {
    Continue,
    Confirm,
    Interrupt,
}

/// Pure list state machine: cursor and activation state only, no I/O.
struct ArtifactList {
    decisions: BTreeMap<String, bool>,
    documents: BTreeMap<String, bool>,
    cursor: usize,
}

impl ArtifactList {
    fn new(decisions: BTreeMap<String, bool>, documents: BTreeMap<String, bool>) -> Self {
        Self {
            decisions,
            documents,
            cursor: 0,
        }
    }

    fn key_at(index: usize) -> &'static str {
        if index < DECISION_ARTIFACTS.len() {
            DECISION_ARTIFACTS[index]
        } else {
            DOCUMENT_ARTIFACTS[index - DECISION_ARTIFACTS.len()]
        }
    }

    fn cursor_key(&self) -> &'static str {
        Self::key_at(self.cursor)
    }

    fn enabled(&self, key: &str) -> bool {
        self.decisions
            .get(key)
            .or_else(|| self.documents.get(key))
            .copied()
            .unwrap_or(true)
    }

    fn toggle_cursor(&mut self) {
        let key = self.cursor_key();
        if let Some(value) = self.decisions.get_mut(key) {
            *value = !*value;
        } else if let Some(value) = self.documents.get_mut(key) {
            *value = !*value;
        }
    }

    fn handle_key(&mut self, key: Key) -> ListOutcome {
        match key {
            Key::Enter | Key::Eof => ListOutcome::Confirm,
            Key::Interrupt => ListOutcome::Interrupt,
            Key::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                ListOutcome::Continue
            }
            Key::Down => {
                if self.cursor + 1 < SELECTABLE_ARTIFACT_COUNT {
                    self.cursor += 1;
                }
                ListOutcome::Continue
            }
            Key::Toggle => {
                self.toggle_cursor();
                ListOutcome::Continue
            }
            Key::Ignored => ListOutcome::Continue,
        }
    }

    /// Append-only redraw: no ANSI cursor movement, clear-line, or colour.
    fn render<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer)?;
        writeln!(
            writer,
            "Artifacts (Up/Down or k/j to move, Space toggles, Enter confirms):"
        )?;
        for key in CORE_ARTIFACTS {
            writeln!(writer, "  [x] {key} (always active)")?;
        }
        for (index, key) in DECISION_ARTIFACTS
            .iter()
            .chain(DOCUMENT_ARTIFACTS.iter())
            .enumerate()
        {
            let cursor = if index == self.cursor { '>' } else { ' ' };
            let mark = if self.enabled(key) { 'x' } else { ' ' };
            writeln!(writer, "{cursor} [{mark}] {key}")?;
        }
        writer.flush()
    }

    fn into_activation(self) -> Activation {
        (self.decisions, self.documents)
    }
}

fn run_artifact_list<W: Write>(
    keys: &mut dyn KeySource,
    writer: &mut W,
    decisions: BTreeMap<String, bool>,
    documents: BTreeMap<String, bool>,
) -> Result<Activation, AppError> {
    let mut list = ArtifactList::new(decisions, documents);
    list.render(writer)
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    loop {
        let key = keys
            .next_key()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let outcome = list.handle_key(key);
        if outcome == ListOutcome::Interrupt {
            return Err(AppError::Interrupted);
        }
        // Every keypress redraws the complete list.
        list.render(writer)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        if outcome == ListOutcome::Confirm {
            return Ok(list.into_activation());
        }
    }
}

fn confirm_degraded<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> Result<(), AppError> {
    loop {
        write!(writer, "Press Enter to continue: ")
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let mut input = String::new();
        let read = reader
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        if read == 0 || input.trim().is_empty() {
            return Ok(());
        }
        writeln!(
            writer,
            "Artifact activation is fixed in this environment; press Enter to continue."
        )
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    }
}

pub fn run_init(project_dir_raw: PathBuf, opts: InitOptions) -> Result<(), AppError> {
    let interactive = std::io::stdin().is_terminal();
    if interactive {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        run_init_with_io(project_dir_raw, opts, true, &mut reader, &mut writer)
    } else {
        let mut reader = std::io::empty();
        let mut writer = std::io::sink();
        run_init_with_io(project_dir_raw, opts, false, &mut reader, &mut writer)
    }
}

pub fn run_init_with_io<R: BufRead, W: Write>(
    project_dir_raw: PathBuf,
    opts: InitOptions,
    interactive: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<(), AppError> {
    let name = sanitize_name(&opts.name).ok_or_else(|| {
        AppError::Validation("project name cannot be normalized into a valid value".to_string())
    })?;

    // Create project dir if absent.
    if !project_dir_raw.exists() {
        fs::create_dir_all(&project_dir_raw).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
    let project_dir =
        fs::canonicalize(&project_dir_raw).map_err(|e| AppError::Runtime(e.to_string()))?;

    let config_path = project_dir.join("papyrus.config.yml");

    // Conflict check: existing config blocks init without --force; writes nothing.
    if config_path.exists() && !opts.force {
        return Err(AppError::Validation(
            "papyrus.config.yml already exists; use --force to replace it".to_string(),
        ));
    }

    // Build effective config.
    let config = if interactive {
        build_config_interactive(name.clone(), reader, writer)?
    } else {
        Config {
            project_name: Some(name.clone()),
            id_padding: 3,
            ..Config::default()
        }
    };

    // Validate before writing.
    config
        .validate(Some(&project_dir))
        .map_err(AppError::Validation)?;

    // Create directories through the shared record descriptors so the layout
    // has one source of truth. Directory creation follows activation.
    for name in ["task", "milestone", "release"] {
        if let Some(desc) = crate::record::descriptor::descriptor_for(name)
            && let Ok(dir) = desc.resolve_directory(&config, &project_dir)
        {
            let _ = fs::create_dir_all(dir);
        }
    }

    // Always ensure the documentation root exists.
    let _ = fs::create_dir_all(project_dir.join(&config.doc_directory));

    let any_decision = config.decision_types.values().any(|v| v.enabled);
    if any_decision
        && let Some(desc) = crate::record::descriptor::descriptor_for("decision")
        && let Ok(dir) = desc.resolve_directory(&config, &project_dir)
    {
        let _ = fs::create_dir_all(dir);
    }

    // Document directories follow the registry: one descriptor per configured
    // document type, enabled or not, with the record facts carried by the same
    // descriptor (ADR-0017 decision 1).
    let registry = crate::registry::ArtifactRegistry::build(&config)?;
    for descriptor in registry.enabled(crate::registry::ArtifactCategory::Document) {
        if let Some(record) = registry.record_descriptor(&descriptor.slug)
            && let Ok(dir) = record.resolve_directory(&config, &project_dir)
        {
            let _ = fs::create_dir_all(&dir);
        }
    }

    // Write config file (full replace under --force or when absent). Each
    // document entry is minimal: `prefix` and `directory`, plus `enabled` only
    // when the walkthrough disables it (REQ-007.002, RE-010 D14). Lists are
    // rendered as flow sequences by the shared config renderer (ADR-0024).
    let value = config.init_value().map_err(AppError::Runtime)?;
    let yaml = crate::config::to_config_yaml(&value).map_err(AppError::Runtime)?;
    crate::config::write_config_file(&config_path, &yaml)?;

    // Git init unless suppressed.
    if !opts.no_git {
        let status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                return Err(AppError::Runtime(format!("git init failed with {}", s)));
            }
            Err(e) => {
                return Err(AppError::Runtime(format!("failed to invoke git: {}", e)));
            }
        }
    }

    Ok(())
}

fn build_config_interactive<R: BufRead, W: Write>(
    name: String,
    reader: &mut R,
    writer: &mut W,
) -> Result<Config, AppError> {
    let mut mode = SttyTerminalMode;
    build_config_interactive_with_mode(name, reader, writer, &mut mode)
}

fn initial_activation() -> Activation {
    let defaults = Config::default();
    (
        defaults
            .decision_types
            .keys()
            .map(|key| (key.clone(), true))
            .collect(),
        defaults
            .document_types
            .keys()
            .map(|key| (key.clone(), true))
            .collect(),
    )
}

fn build_config_interactive_with_mode<R: BufRead, W: Write>(
    name: String,
    reader: &mut R,
    writer: &mut W,
    mode: &mut dyn TerminalMode,
) -> Result<Config, AppError> {
    let (decisions, documents) = initial_activation();

    let (active_decisions, active_docs) = match mode.enter() {
        Ok(mut guard) => {
            let outcome = {
                let mut keys = ByteKeySource::new(&mut *reader);
                run_artifact_list(&mut keys, writer, decisions, documents)
            };
            // Restore before the line-based prefix/folder prompts run.
            guard.restore();
            outcome?
        }
        Err(_) => {
            // One stderr warning: the list still renders and confirms line-based.
            eprintln!(
                "warning: key navigation is unavailable in this environment; using a line-based confirmation with all artifacts active"
            );
            let list = ArtifactList::new(decisions, documents);
            list.render(writer)
                .map_err(|e| AppError::Runtime(e.to_string()))?;
            confirm_degraded(reader, writer)?;
            list.into_activation()
        }
    };

    build_config_with_activation(name, reader, writer, active_decisions, active_docs)
}

fn build_config_with_activation<R: BufRead, W: Write>(
    name: String,
    reader: &mut R,
    writer: &mut W,
    active_decisions: BTreeMap<String, bool>,
    active_docs: BTreeMap<String, bool>,
) -> Result<Config, AppError> {
    let defaults = Config::default();
    let mut config = Config {
        project_name: Some(name),
        id_padding: 3,
        ..Config::default()
    };

    let default_task_prefix = defaults.task_prefix.clone();
    let task_prefix = prompt_value(
        reader,
        writer,
        &format!("Task ID prefix [{}]: ", default_task_prefix),
        &default_task_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_task_prefix),
    )?;
    let default_milestone_prefix = defaults.milestone_prefix.clone();
    let milestone_prefix = prompt_value(
        reader,
        writer,
        &format!("Milestone ID prefix [{}]: ", default_milestone_prefix),
        &default_milestone_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_milestone_prefix),
    )?;
    let default_release_prefix = defaults.release_prefix.clone();
    let release_prefix = prompt_value(
        reader,
        writer,
        &format!("Release ID prefix [{}]: ", default_release_prefix),
        &default_release_prefix,
        sanitize_name,
        &format!("invalid prefix; use {}", default_release_prefix),
    )?;
    let default_backlog_directory = defaults.backlog_directory.clone();
    let backlog_directory = prompt_value(
        reader,
        writer,
        &format!("Backlog directory [{}]: ", default_backlog_directory),
        &default_backlog_directory,
        sanitize_path_string,
        &format!("invalid folder; use {}", default_backlog_directory),
    )?;

    // Decision types.
    let any_decision = active_decisions.values().any(|v| *v);
    let default_decision_directory = defaults.decision_directory.clone();
    let decision_directory = if any_decision {
        prompt_value(
            reader,
            writer,
            &format!("Decision directory [{}]: ", default_decision_directory),
            &default_decision_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", default_decision_directory),
        )?
    } else {
        default_decision_directory.clone()
    };

    for (key, entry) in &defaults.decision_types {
        let activated = active_decisions.get(key).copied().unwrap_or(true);
        if activated {
            let prefix = prompt_value(
                reader,
                writer,
                &format!("  {} ID prefix [{}]: ", key, entry.prefix),
                &entry.prefix,
                sanitize_name,
                &format!("invalid prefix; use {}", entry.prefix),
            )?;
            if let Some(e) = config.decision_types.get_mut(key) {
                e.prefix = prefix;
                e.enabled = true;
            }
        } else if let Some(e) = config.decision_types.get_mut(key) {
            e.enabled = false;
        }
    }

    // Document types.
    let any_doc = active_docs.values().any(|v| *v);
    let default_doc_directory = defaults.doc_directory.clone();
    let doc_directory = if any_doc {
        prompt_value(
            reader,
            writer,
            &format!("Documentation root [{}]: ", default_doc_directory),
            &default_doc_directory,
            sanitize_path_string,
            &format!("invalid folder; use {}", default_doc_directory),
        )?
    } else {
        default_doc_directory.clone()
    };

    for (key, entry) in &defaults.document_types {
        let activated = active_docs.get(key).copied().unwrap_or(true);
        if activated {
            let prefix = prompt_value(
                reader,
                writer,
                &format!("  {} ID prefix [{}]: ", key, entry.prefix),
                &entry.prefix,
                sanitize_name,
                &format!("invalid prefix; use {}", entry.prefix),
            )?;
            let directory = prompt_value(
                reader,
                writer,
                &format!("  {} directory [{}]: ", key, entry.directory),
                &entry.directory,
                sanitize_path_string,
                &format!("invalid folder; use {}", entry.directory),
            )?;
            if let Some(e) = config.document_types.get_mut(key) {
                e.prefix = prefix;
                e.directory = directory;
                e.enabled = true;
            }
        } else if let Some(e) = config.document_types.get_mut(key) {
            e.enabled = false;
        }
    }

    config.task_prefix = task_prefix;
    config.milestone_prefix = milestone_prefix;
    config.release_prefix = release_prefix;
    config.backlog_directory = backlog_directory;
    config.decision_directory = decision_directory;
    config.doc_directory = doc_directory;

    Ok(config)
}

fn prompt_value<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    default: &str,
    validate: fn(&str) -> Option<String>,
    hint: &str,
) -> Result<String, AppError> {
    loop {
        write!(writer, "{}", prompt).map_err(|e| AppError::Runtime(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        let trimmed = input.trim();
        let candidate = if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        };
        if let Some(v) = validate(&candidate) {
            return Ok(v);
        }
        writeln!(writer, "{}", hint).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::Cursor;
    use std::rc::Rc;

    struct ScriptedKeySource {
        keys: std::vec::IntoIter<Key>,
    }

    impl ScriptedKeySource {
        fn new(keys: Vec<Key>) -> Self {
            Self {
                keys: keys.into_iter(),
            }
        }
    }

    impl KeySource for ScriptedKeySource {
        fn next_key(&mut self) -> io::Result<Key> {
            Ok(self.keys.next().unwrap_or(Key::Eof))
        }
    }

    struct FakeTerminalMode {
        available: bool,
        restored: Rc<Cell<bool>>,
    }

    struct FakeGuard(Rc<Cell<bool>>);

    impl TerminalModeGuard for FakeGuard {
        fn restore(&mut self) {
            self.0.set(true);
        }
    }

    impl TerminalMode for FakeTerminalMode {
        fn enter(&self) -> io::Result<Box<dyn TerminalModeGuard>> {
            if self.available {
                Ok(Box::new(FakeGuard(self.restored.clone())))
            } else {
                Err(io::Error::other("no stty in test"))
            }
        }
    }

    fn list() -> ArtifactList {
        let (decisions, documents) = initial_activation();
        ArtifactList::new(decisions, documents)
    }

    fn run_scripted(keys: Vec<Key>) -> (String, Activation) {
        let mut writer = Vec::new();
        let (decisions, documents) = initial_activation();
        let mut source = ScriptedKeySource::new(keys);
        let activation = run_artifact_list(&mut source, &mut writer, decisions, documents).unwrap();
        (String::from_utf8(writer).unwrap(), activation)
    }

    fn header_count(output: &str) -> usize {
        output.matches("Artifacts (Up/Down").count()
    }

    #[test]
    fn initial_cursor_is_adr_and_every_row_is_active() {
        let list = list();
        assert_eq!(list.cursor_key(), "adr");
        let mut output = Vec::new();
        list.render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("  [x] task (always active)\n"));
        assert!(output.contains("  [x] milestone (always active)\n"));
        assert!(output.contains("  [x] release (always active)\n"));
        assert!(output.contains("> [x] adr\n"));
        assert!(!output.contains("> [x] task"));
        assert!(!output.contains("> [x] milestone"));
        assert!(!output.contains("> [x] release"));
    }

    #[test]
    fn rows_follow_the_required_artifact_order() {
        let mut output = Vec::new();
        list().render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        let expected = [
            "task",
            "milestone",
            "release",
            "adr",
            "bdr",
            "req",
            "nfr",
            "glossary",
            "rule",
            "ui-spec",
            "ui-mockup",
            "audit",
            "tech-spec",
            "use-case",
            "test-case",
            "test-report",
        ];
        let mut offset = 0;
        for key in expected {
            let needle = format!("] {key}");
            let position = output[offset..]
                .find(&needle)
                .unwrap_or_else(|| panic!("missing row {key}"))
                + offset;
            assert!(position >= offset);
            offset = position + needle.len();
        }
    }

    #[test]
    fn selectable_document_types_are_the_eleven_defaults_without_deploy() {
        assert_eq!(DOCUMENT_ARTIFACTS.len(), 11);
        assert_eq!(SELECTABLE_ARTIFACT_COUNT, 13);
        assert!(!DOCUMENT_ARTIFACTS.contains(&"deploy"));
        let mut output = Vec::new();
        list().render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains("deploy"), "{}", output);
    }

    #[test]
    fn document_artifacts_match_config_default_document_types() {
        let seeded = crate::config::SEED_DOCUMENT_TYPES;
        let seeded_slugs: Vec<&str> = seeded.iter().map(|entry| entry.slug).collect();

        // Position-by-position equality against the canonical seeding order,
        // so an add, remove, reorder, or rename on either side fails.
        assert_eq!(
            DOCUMENT_ARTIFACTS.as_slice(),
            seeded_slugs.as_slice(),
            "init order must match the config seeding order"
        );

        // Set equality against the live default map, so a config-only add or
        // loss fails even when the seeding constant stays aligned.
        let defaults = Config::default();
        let mut config_slugs: Vec<&str> =
            defaults.document_types.keys().map(String::as_str).collect();
        config_slugs.sort_unstable();
        let mut init_slugs = DOCUMENT_ARTIFACTS.to_vec();
        init_slugs.sort_unstable();
        assert_eq!(
            init_slugs, config_slugs,
            "init default slugs must equal Config::default().document_types keys"
        );

        // Exactly eleven unique slugs on each side; no deploy.
        assert_eq!(DOCUMENT_ARTIFACTS.len(), 11);
        let unique: std::collections::BTreeSet<&str> = DOCUMENT_ARTIFACTS.iter().copied().collect();
        assert_eq!(unique.len(), 11, "init slugs must be unique");
        assert_eq!(defaults.document_types.len(), 11);
        assert!(!DOCUMENT_ARTIFACTS.contains(&"deploy"));
        assert!(!defaults.document_types.contains_key("deploy"));

        // Seeded shape: ten front-matter types declare [active, archived],
        // ui-mockup declares no status, and every default type is enabled.
        for (slug, entry) in &defaults.document_types {
            assert!(entry.enabled, "{slug} must be enabled by default");
            if slug == "ui-mockup" {
                assert!(entry.statuses.is_none(), "ui-mockup has no statuses");
            } else {
                assert_eq!(
                    entry.statuses,
                    Some(vec!["active".to_string(), "archived".to_string()]),
                    "{slug} must seed [active, archived]"
                );
            }
        }
    }

    fn tmpdir(name: &str) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("init-{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn non_interactive_init_writes_the_eleven_types_and_flow_statuses() {
        let dir = tmpdir("generated");
        let opts = InitOptions {
            name: "Proj".to_string(),
            no_git: true,
            force: false,
        };
        let mut reader = Cursor::new(Vec::new());
        let mut writer = Vec::new();
        run_init_with_io(dir.clone(), opts, false, &mut reader, &mut writer).unwrap();

        let raw = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
        for slug in [
            "req",
            "nfr",
            "glossary",
            "rule",
            "ui-spec",
            "ui-mockup",
            "audit",
            "tech-spec",
            "use-case",
            "test-case",
            "test-report",
        ] {
            assert!(raw.contains(&format!("\n  {}:\n", slug)), "{}", raw);
        }
        assert!(!raw.contains("\n  deploy:\n"), "{}", raw);
        assert!(
            !raw.contains("statuses: [active, archived]"),
            "init must not write seed-filled collections: {}",
            raw
        );
        assert!(!raw.contains("    default_status"), "{}", raw);
        assert!(!raw.contains("    fields:"), "{}", raw);

        let config: Config = yaml_serde::from_str(&raw).unwrap();
        assert_eq!(config.document_types["req"].statuses, None);
        assert_eq!(config.document_types["ui-mockup"].statuses, None);
        config.validate(Some(&dir)).unwrap();

        let effective =
            crate::commands::load_effective_config(&dir.join("papyrus.config.yml"), &dir).unwrap();
        assert_eq!(
            effective.document_types["req"].statuses,
            Some(vec!["active".to_string(), "archived".to_string()])
        );
    }

    #[test]
    fn injected_write_failure_keeps_the_previous_configuration() {
        let dir = tmpdir("write-failure");
        let config_path = dir.join("papyrus.config.yml");
        let previous = "project_name: Before\n";
        fs::write(&config_path, previous).unwrap();
        let opts = InitOptions {
            name: "After".to_string(),
            no_git: true,
            force: true,
        };
        crate::config::fail_next_config_write_for_test();
        let mut reader = Cursor::new(Vec::new());
        let mut writer = Vec::new();
        let error = run_init_with_io(dir.clone(), opts, false, &mut reader, &mut writer)
            .expect_err("the injected write failure must fail init");
        assert!(matches!(error, AppError::Runtime(_)), "{:?}", error);
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            previous,
            "the previous configuration must stay intact"
        );
        let leftovers: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.contains(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "leftover temporary files: {leftovers:?}"
        );
    }

    #[test]
    fn walkthrough_disabling_a_document_type_keeps_its_seeded_shape() {
        // Cursor: adr -> bdr -> req.
        let (_, (decisions, documents)) =
            run_scripted(vec![Key::Down, Key::Down, Key::Toggle, Key::Enter]);
        let mut reader = Cursor::new(vec![b'\n'; 40]);
        let mut writer = Vec::new();
        let config = build_config_with_activation(
            "Proj".into(),
            &mut reader,
            &mut writer,
            decisions,
            documents,
        )
        .unwrap();

        let req = &config.document_types["req"];
        assert!(!req.enabled);
        assert_eq!(req.prefix, "REQ");
        assert_eq!(req.directory, "requirements/business/");
        assert_eq!(
            req.statuses,
            Some(vec!["active".to_string(), "archived".to_string()])
        );
        assert_eq!(req.default_status, None);
        assert!(config.validate(None).is_ok());
    }

    #[test]
    fn down_and_j_move_to_the_next_selectable_row() {
        let mut list = list();
        assert_eq!(list.handle_key(Key::Down), ListOutcome::Continue);
        assert_eq!(list.cursor_key(), "bdr");
        list.handle_key(Key::Down);
        assert_eq!(list.cursor_key(), "req");
        list.handle_key(Key::Up);
        assert_eq!(list.cursor_key(), "bdr");
        list.handle_key(Key::Up);
        assert_eq!(list.cursor_key(), "adr");
    }

    #[test]
    fn cursor_clamps_at_both_ends_without_wrapping() {
        let mut list = list();
        list.handle_key(Key::Up);
        assert_eq!(list.cursor_key(), "adr");
        for _ in 0..100 {
            list.handle_key(Key::Down);
        }
        assert_eq!(list.cursor_key(), "test-report");
        list.handle_key(Key::Down);
        assert_eq!(list.cursor_key(), "test-report");
        list.handle_key(Key::Up);
        assert_eq!(list.cursor_key(), "test-case");
    }

    #[test]
    fn space_toggles_only_the_highlighted_row() {
        let mut list = list();
        list.handle_key(Key::Toggle);
        assert!(!list.enabled("adr"));
        assert!(list.enabled("bdr"));

        let mut output = Vec::new();
        list.render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("> [ ] adr\n"));
        assert!(output.contains("  [x] bdr\n"));

        list.handle_key(Key::Toggle);
        assert!(list.enabled("adr"));
    }

    #[test]
    fn core_rows_are_never_reachable_or_toggleable() {
        let mut list = list();
        let mut keys = vec![Key::Up, Key::Up, Key::Toggle];
        keys.extend(std::iter::repeat_n(Key::Down, 30));
        keys.extend(std::iter::repeat_n(Key::Toggle, 3));
        for key in keys {
            list.handle_key(key);
            assert!(!CORE_ARTIFACTS.contains(&list.cursor_key()));
        }
        assert!(!list.decisions.contains_key("task"));
        assert!(!list.documents.contains_key("task"));
    }

    #[test]
    fn unrelated_keys_do_not_change_activation() {
        let mut list = list();
        let before = (list.decisions.clone(), list.documents.clone());
        list.handle_key(Key::Ignored);
        assert_eq!((list.decisions.clone(), list.documents.clone()), before);
    }

    #[test]
    fn every_keypress_redraws_the_whole_list() {
        let (output, _) = run_scripted(vec![Key::Ignored, Key::Down, Key::Toggle, Key::Enter]);
        assert_eq!(header_count(&output), 5);
        let last = output.rsplit("Artifacts (Up/Down").next().unwrap();
        assert!(last.contains("> [ ] bdr"));
        assert!(last.contains("  [x] adr"));
        assert!(last.contains("  [x] test-report"));
    }

    #[test]
    fn enter_confirms_the_state_shown_on_screen() {
        let (_, (decisions, documents)) = run_scripted(vec![Key::Down, Key::Toggle, Key::Enter]);
        assert!(decisions["adr"]);
        assert!(!decisions["bdr"]);
        assert!(documents["req"]);
    }

    #[test]
    fn end_of_input_confirms_the_current_state() {
        let (_, (decisions, documents)) = run_scripted(vec![Key::Toggle]);
        assert!(!decisions["adr"]);
        assert!(decisions["bdr"]);
        assert!(documents["req"]);

        let mut empty = Cursor::new(Vec::new());
        let mut source = ByteKeySource::new(&mut empty);
        assert_eq!(source.next_key().unwrap(), Key::Eof);
    }

    #[test]
    fn byte_key_source_parses_keys_and_escape_sequences() {
        let bytes = b"\x1b[A\x1bOBjk \x04\x03\n\r".to_vec();
        let mut reader = Cursor::new(bytes);
        let mut source = ByteKeySource::new(&mut reader);
        assert_eq!(source.next_key().unwrap(), Key::Up);
        assert_eq!(source.next_key().unwrap(), Key::Down);
        assert_eq!(source.next_key().unwrap(), Key::Down);
        assert_eq!(source.next_key().unwrap(), Key::Up);
        assert_eq!(source.next_key().unwrap(), Key::Toggle);
        assert_eq!(source.next_key().unwrap(), Key::Eof);
        assert_eq!(source.next_key().unwrap(), Key::Interrupt);
        assert_eq!(source.next_key().unwrap(), Key::Enter);
        assert_eq!(source.next_key().unwrap(), Key::Enter);
    }

    #[test]
    fn lone_escape_and_truncated_sequences_are_ignored() {
        let mut reader = Cursor::new(b"\x1b\x1b[Z".to_vec());
        let mut source = ByteKeySource::new(&mut reader);
        assert_eq!(source.next_key().unwrap(), Key::Ignored);
        assert_eq!(source.next_key().unwrap(), Key::Ignored);
        assert_eq!(source.next_key().unwrap(), Key::Ignored);
        assert_eq!(source.next_key().unwrap(), Key::Eof);
    }

    #[test]
    fn interrupted_reads_are_retried() {
        struct InterruptOnceReader {
            inner: Cursor<Vec<u8>>,
            interrupted: bool,
        }

        impl io::Read for InterruptOnceReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(io::Error::new(ErrorKind::Interrupted, "eintr"));
                }
                self.inner.read(buf)
            }
        }

        impl BufRead for InterruptOnceReader {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                self.inner.fill_buf()
            }

            fn consume(&mut self, amount: usize) {
                self.inner.consume(amount);
            }
        }

        let mut reader = InterruptOnceReader {
            inner: Cursor::new(b"\n".to_vec()),
            interrupted: false,
        };
        let mut source = ByteKeySource::new(&mut reader);
        assert_eq!(source.next_key().unwrap(), Key::Enter);
    }

    #[test]
    fn ctrl_c_maps_to_interrupted_exit_code_130() {
        let (decisions, documents) = initial_activation();
        let mut source = ScriptedKeySource::new(vec![Key::Interrupt]);
        let mut writer = Vec::new();
        let error = run_artifact_list(&mut source, &mut writer, decisions, documents).unwrap_err();
        assert!(matches!(error, AppError::Interrupted));
        assert_eq!(error.code(), 130);
        assert_eq!(error.message(), "interrupted");
    }

    #[test]
    fn typed_tokens_do_not_change_activation() {
        let mut reader = Cursor::new(b"req\n".to_vec());
        let mut source = ByteKeySource::new(&mut reader);
        let (decisions, documents) = initial_activation();
        let mut writer = Vec::new();
        let (decisions, documents) =
            run_artifact_list(&mut source, &mut writer, decisions, documents).unwrap();
        assert!(decisions.values().all(|enabled| *enabled));
        assert!(documents.values().all(|enabled| *enabled));
    }

    #[test]
    fn fake_mode_success_relies_on_the_real_key_source_and_restores() {
        let restored = Rc::new(Cell::new(false));
        let mut mode = FakeTerminalMode {
            available: true,
            restored: restored.clone(),
        };
        let mut reader = Cursor::new(vec![b'\n'; 40]);
        let mut writer = Vec::new();
        let config =
            build_config_interactive_with_mode("Proj".into(), &mut reader, &mut writer, &mut mode)
                .unwrap();
        assert!(restored.get());
        assert!(config.decision_types.values().all(|entry| entry.enabled));
        assert!(config.document_types.values().all(|entry| entry.enabled));
    }

    #[test]
    fn stty_failure_degrades_to_one_render_and_line_confirmation() {
        let restored = Rc::new(Cell::new(false));
        let mut mode = FakeTerminalMode {
            available: false,
            restored: restored.clone(),
        };
        let mut reader = Cursor::new(vec![b'\n'; 40]);
        let mut writer = Vec::new();
        let config =
            build_config_interactive_with_mode("Proj".into(), &mut reader, &mut writer, &mut mode)
                .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert_eq!(header_count(&output), 1);
        assert!(!output.contains("> [ ]"));
        assert!(!restored.get());
        assert!(config.decision_types.values().all(|entry| entry.enabled));
        assert!(config.document_types.values().all(|entry| entry.enabled));
    }

    #[test]
    fn degraded_path_never_activates_from_typed_tokens() {
        let restored = Rc::new(Cell::new(false));
        let mut mode = FakeTerminalMode {
            available: false,
            restored,
        };
        let mut bytes = b"req\n\n".to_vec();
        bytes.extend(std::iter::repeat_n(b'\n', 40));
        let mut reader = Cursor::new(bytes);
        let mut writer = Vec::new();
        let config =
            build_config_interactive_with_mode("Proj".into(), &mut reader, &mut writer, &mut mode)
                .unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Artifact activation is fixed in this environment"));
        assert!(config.decision_types.values().all(|entry| entry.enabled));
        assert!(config.document_types.values().all(|entry| entry.enabled));
    }

    #[test]
    fn activation_from_the_list_reaches_the_generated_config() {
        let (_, (decisions, documents)) =
            run_scripted(vec![Key::Down, Key::Down, Key::Toggle, Key::Enter]);
        let mut reader = Cursor::new(vec![b'\n'; 40]);
        let mut writer = Vec::new();
        let config = build_config_with_activation(
            "Proj".into(),
            &mut reader,
            &mut writer,
            decisions,
            documents,
        )
        .unwrap();
        assert!(!config.document_types["req"].enabled);
        assert!(config.document_types["nfr"].enabled);
        assert!(config.decision_types["adr"].enabled);
    }

    #[test]
    fn document_artifacts_derive_from_the_seed_table_without_a_second_slug_list() {
        // ADR-0020 decision 1 clarification (2026-11-23): the interactive-init
        // list derives from `SEED_DOCUMENT_TYPES`; no production line in this
        // module may restate a document slug.
        let production = include_str!("init.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default();
        for seed in crate::config::SEED_DOCUMENT_TYPES {
            assert!(
                !production.contains(&format!("\"{}\"", seed.slug)),
                "init production code must not restate the document slug {}",
                seed.slug
            );
        }
    }

    #[test]
    fn legacy_typed_token_prompt_text_is_absent() {
        let source = include_str!("init.rs");
        assert!(!source.contains(concat!("Toggle", " artifacts")));
        assert!(!source.contains(concat!("space", "-separated")));
    }
}
