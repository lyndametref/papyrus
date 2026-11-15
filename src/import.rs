//! One-way import of Backlog.md task and milestone records into Papyrus.

use crate::commands::load_effective_config;
use crate::config::Config;
use crate::context::Context;
use crate::error::AppError;
use crate::record::descriptor::descriptor_for;
use crate::record::document;
use crate::record::frontmatter;
use crate::record::frontmatter::FrontMatter;
use crate::record::frontmatter::FrontMatterValue;
use crate::record::identity;
use crate::record::patch::PatchOp;
use crate::record::patch::apply_patch as apply_patch_fn;
use crate::record::store;
use clap::Subcommand;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug)]
pub enum ImportCmd {
    /// Import tasks and milestones from a Backlog.md store.
    Backlog {
        /// Source project directory.
        #[arg(long = "from", value_name = "source-project-dir")]
        from: PathBuf,
        /// Source backlog directory (default: <from>/backlog).
        #[arg(long = "source-backlog-dir", value_name = "dir")]
        source_backlog_dir: Option<PathBuf>,
        /// Plan and report without writing anything.
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// Allow import into a target that already holds records of an imported family.
        #[arg(long = "allow-existing")]
        allow_existing: bool,
        /// Write the report to a path inside the project directory.
        #[arg(long = "report", value_name = "path")]
        report: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Task,
    Milestone,
}

#[derive(Clone)]
struct SrcRecord {
    family: Family,
    archived: bool,
    path: PathBuf,
    stem: String,
    source_number: i64,
    source_id: String,
    fm: FrontMatter,
    title: String,
    body: String,

    // raw parsed fields
    status_raw: Option<String>,
    assignee: Vec<String>,
    created_raw: Option<String>,
    updated_raw: Option<String>,
    milestone_raw: Option<String>,
    dependencies: Vec<String>,
    type_raw: Option<String>,
    ordinal_raw: Option<i64>,

    // computed
    target_id: String,
    target_number: i64,
    normalized_status: String,
    status_mismatch: bool,
    created_final: String,
    created_approx: bool,
    updated_final: String,
    updated_approx: bool,
    milestone_target: Option<String>,
    milestone_dropped: Option<String>,
    dependencies_target: Vec<String>,
    dropped_deps: Vec<String>,
    ordinal_final: i64,
    dest_path: PathBuf,
    warnings: Vec<String>,
}

#[derive(Default)]
struct ExcludedCounts {
    docs: usize,
    drafts: usize,
    decisions: usize,
    config: usize,
    locks: usize,
}

pub fn run(ctx: &Context, cmd: ImportCmd) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        ImportCmd::Backlog {
            from,
            source_backlog_dir,
            dry_run,
            allow_existing,
            report,
        } => run_backlog(
            ctx,
            from,
            source_backlog_dir,
            dry_run,
            allow_existing,
            report,
        ),
    }
}

#[allow(clippy::vec_init_then_push)]
fn run_backlog(
    ctx: &Context,
    from: PathBuf,
    source_backlog_dir: Option<PathBuf>,
    dry_run: bool,
    allow_existing: bool,
    report: Option<PathBuf>,
) -> Result<crate::CommandOutput, AppError> {
    let cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;

    // Source resolution.
    let from_canon = fs::canonicalize(&from)
        .map_err(|e| AppError::Runtime(format!("source project directory: {}", e)))?;
    let backlog_dir = source_backlog_dir
        .map(|p| {
            if p.is_absolute() {
                p
            } else {
                from_canon.join(p)
            }
        })
        .unwrap_or_else(|| from_canon.join("backlog"));
    let backlog_canon = fs::canonicalize(&backlog_dir)
        .map_err(|e| AppError::Runtime(format!("source backlog directory: {}", e)))?;
    if !backlog_canon.is_dir() {
        return Err(AppError::Runtime(format!(
            "source backlog directory is not a directory: {}",
            backlog_canon.display()
        )));
    }

    // Report path containment. Resolve relative paths against the project dir.
    let report_resolved = report.as_ref().map(|rp| {
        if rp.is_absolute() {
            rp.clone()
        } else {
            ctx.project_dir.join(rp)
        }
    });
    let report_safe = if let Some(rp) = &report_resolved {
        Some(Context::ensure_within(&ctx.project_dir, rp).map_err(|_| {
            AppError::Validation(format!(
                "report path resolves outside project directory: {}",
                rp.display()
            ))
        })?)
    } else {
        None
    };

    // Excluded counts.
    let excluded = count_excluded(&backlog_canon);

    // Enumerate source records.
    let mut task_records: Vec<SrcRecord> = Vec::new();
    let mut milestone_records: Vec<SrcRecord> = Vec::new();
    let mut skipped: Vec<(PathBuf, String)> = Vec::new();
    let mut anomalies: Vec<String> = Vec::new();

    let locations: Vec<(Family, bool, PathBuf)> = vec![
        (Family::Task, false, backlog_canon.join("tasks")),
        (
            Family::Task,
            true,
            backlog_canon.join("archive").join("tasks"),
        ),
        (Family::Milestone, false, backlog_canon.join("milestones")),
        (
            Family::Milestone,
            true,
            backlog_canon.join("archive").join("milestones"),
        ),
    ];

    for (family, archived, dir) in locations {
        if !dir.exists() {
            continue;
        }
        let mut entries: Vec<fs::DirEntry> = match fs::read_dir(&dir) {
            Ok(rd) => rd.flatten().collect(),
            Err(e) => {
                return Err(AppError::Runtime(format!(
                    "cannot read source directory {}: {}",
                    dir.display(),
                    e
                )));
            }
        };
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Non-markdown or non-matching stem -> skipped.
            if ext != "md" {
                skipped.push((path.clone(), "not a markdown record".to_string()));
                continue;
            }

            let source_number = match parse_source_number(family, &stem) {
                Some(n) => n,
                None => {
                    skipped.push((path.clone(), "unparseable source number".to_string()));
                    continue;
                }
            };

            // Read content.
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    skipped.push((path.clone(), format!("cannot read file: {}", e)));
                    continue;
                }
            };

            // Parse front matter + document.
            let fm = match frontmatter::parse(&content) {
                Ok(Some(f)) => f,
                Ok(None) => {
                    skipped.push((path.clone(), "missing front matter".to_string()));
                    continue;
                }
                Err(e) => {
                    skipped.push((path.clone(), format!("malformed front matter: {}", e)));
                    continue;
                }
            };
            if document::parse(&content).is_err() {
                skipped.push((path.clone(), "malformed document".to_string()));
                continue;
            }

            let title = match yaml_string(&fm, "title") {
                Some(t) if !t.trim().is_empty() => t,
                _ => {
                    skipped.push((path.clone(), "missing title".to_string()));
                    continue;
                }
            };

            let status_raw = yaml_string(&fm, "status");
            let assignee = yaml_seq_strings(&fm, "assignee");
            let created_raw = yaml_string(&fm, "created_date");
            let updated_raw = yaml_string(&fm, "updated_date");
            let milestone_raw = yaml_string(&fm, "milestone");
            let dependencies = yaml_seq_strings(&fm, "dependencies");
            let type_raw = yaml_string(&fm, "type");
            let ordinal_raw =
                yaml_string(&fm, "ordinal").and_then(|s| s.trim().parse::<i64>().ok());
            // require non-negative for ordinal
            let ordinal_raw = ordinal_raw.filter(|n| *n >= 0);

            let source_id = yaml_string(&fm, "id").unwrap_or_else(|| stem.clone());

            let body = if let Some((_, _, bs)) = frontmatter::find_block(&content) {
                content[bs..].to_string()
            } else {
                content.clone()
            };

            let rec = SrcRecord {
                family,
                archived,
                path: path.clone(),
                stem: stem.clone(),
                source_number,
                source_id,
                fm,
                title,
                body,
                status_raw,
                assignee,
                created_raw,
                updated_raw,
                milestone_raw,
                dependencies,
                type_raw,
                ordinal_raw,
                target_id: String::new(),
                target_number: 0,
                normalized_status: String::new(),
                status_mismatch: false,
                created_final: String::new(),
                created_approx: false,
                updated_final: String::new(),
                updated_approx: false,
                milestone_target: None,
                milestone_dropped: None,
                dependencies_target: Vec::new(),
                dropped_deps: Vec::new(),
                ordinal_final: 0,
                dest_path: PathBuf::new(),
                warnings: Vec::new(),
            };

            match family {
                Family::Task => task_records.push(rec),
                Family::Milestone => milestone_records.push(rec),
            }
        }
    }

    // Sort per family by source number then path.
    task_records.sort_by(|a, b| {
        a.source_number
            .cmp(&b.source_number)
            .then(a.path.cmp(&b.path))
    });
    milestone_records.sort_by(|a, b| {
        a.source_number
            .cmp(&b.source_number)
            .then(a.path.cmp(&b.path))
    });

    // Source anomalies: duplicate source numbers.
    let mut seen_task_nums: HashSet<i64> = HashSet::new();
    for rec in &task_records {
        if !seen_task_nums.insert(rec.source_number) {
            anomalies.push(format!(
                "duplicate source number in tasks: {} ({})",
                rec.source_number,
                rec.path.display()
            ));
        }
    }
    let mut seen_ms_nums: HashSet<i64> = HashSet::new();
    for rec in &milestone_records {
        if !seen_ms_nums.insert(rec.source_number) {
            anomalies.push(format!(
                "duplicate source number in milestones: {} ({})",
                rec.source_number,
                rec.path.display()
            ));
        }
    }

    // Stem/id mismatch.
    for rec in task_records.iter().chain(milestone_records.iter()) {
        if let Some(fm_id) = yaml_string(&rec.fm, "id") {
            let fm_num = extract_number(&fm_id);
            if let Some(n) = fm_num
                && n != rec.source_number
            {
                anomalies.push(format!(
                    "source stem/id number mismatch: {} ({}) stem {} vs front matter {}",
                    rec.path.display(),
                    rec.source_id,
                    rec.source_number,
                    n
                ));
            }
        }
    }

    // Phase 1: allocate target IDs.
    let task_desc = descriptor_for("task")
        .ok_or_else(|| AppError::Runtime("task descriptor unavailable".to_string()))?;
    let ms_desc = descriptor_for("milestone")
        .ok_or_else(|| AppError::Runtime("milestone descriptor unavailable".to_string()))?;

    let task_dirs = task_desc.resolve_read_directories(&cfg, &ctx.project_dir);
    let task_dir_refs: Vec<&Path> = task_dirs.iter().map(|p| p.as_path()).collect();
    let task_observed = identity::observed_numbers_in_dirs(&task_dir_refs, &cfg.task_prefix);
    let task_base = identity::allocate_next(&task_observed);

    let ms_dirs = ms_desc.resolve_read_directories(&cfg, &ctx.project_dir);
    let ms_dir_refs: Vec<&Path> = ms_dirs.iter().map(|p| p.as_path()).collect();
    let ms_observed = identity::observed_numbers_in_dirs(&ms_dir_refs, &cfg.milestone_prefix);
    let ms_base = identity::allocate_next(&ms_observed);

    for (i, rec) in task_records.iter_mut().enumerate() {
        rec.target_number = task_base + i as i64;
        rec.target_id =
            identity::format_id(&cfg.task_prefix, rec.target_number, cfg.id_padding as usize);
    }
    for (i, rec) in milestone_records.iter_mut().enumerate() {
        rec.target_number = ms_base + i as i64;
        rec.target_id = identity::format_id(
            &cfg.milestone_prefix,
            rec.target_number,
            cfg.id_padding as usize,
        );
    }

    // Allow-existing check.
    if !task_records.is_empty() && !task_observed.is_empty() && !allow_existing {
        return Err(AppError::Runtime(format!(
            "target already holds {} task record(s); use --allow-existing if duplication is intended",
            task_observed.len()
        )));
    }
    if !milestone_records.is_empty() && !ms_observed.is_empty() && !allow_existing {
        return Err(AppError::Runtime(format!(
            "target already holds {} milestone record(s); use --allow-existing if duplication is intended",
            ms_observed.len()
        )));
    }

    // Build source->target maps.
    let mut task_map: BTreeMap<String, String> = BTreeMap::new();
    for rec in &task_records {
        task_map.insert(rec.source_id.clone(), rec.target_id.clone());
    }
    let mut ms_map: BTreeMap<String, String> = BTreeMap::new();
    for rec in &milestone_records {
        ms_map.insert(rec.source_id.clone(), rec.target_id.clone());
    }

    // Phase 2: normalize statuses, timestamps, translate refs, ordinal, dest paths.
    let now = frontmatter::utc_now();

    // Task ordinal base.
    let task_ordinal_base = identity::scan_max_ordinal_in_dirs(&task_dir_refs);
    let mut task_ordinal_counter = task_ordinal_base;
    let ms_ordinal_base = identity::scan_max_ordinal_in_dirs(&ms_dir_refs);
    let ms_ordinal_counter = ms_ordinal_base;

    // Existing titles for possible-duplicate detection.
    let existing_task_titles = collect_existing_titles(ctx, &cfg, "task", &task_dirs);
    let existing_ms_titles = collect_existing_titles(ctx, &cfg, "milestone", &ms_dirs);

    // Snapshots for translation lookups.
    let ms_snapshot = milestone_records.clone();
    let task_snapshot = task_records.clone();
    // Process tasks.
    for rec in task_records.iter_mut() {
        // status
        let (st, mismatch) = normalize_status(&cfg, rec.status_raw.as_deref());
        rec.normalized_status = st;
        rec.status_mismatch = mismatch;
        if mismatch {
            rec.warnings.push(format!(
                "status '{}' does not match a configured status; preserved verbatim",
                rec.status_raw.clone().unwrap_or_default()
            ));
        }

        // timestamps
        rec.created_final = match rec.created_raw.as_deref() {
            Some(v) if frontmatter::is_timestamp_pub(v) => v.to_string(),
            Some(v) => {
                rec.created_approx = true;
                rec.warnings.push(format!(
                    "created_date '{}' is not a valid ISO-8601 UTC timestamp; approximated",
                    v
                ));
                now.clone()
            }
            None => {
                rec.created_approx = true;
                rec.warnings
                    .push("created_date missing; approximated".to_string());
                now.clone()
            }
        };
        rec.updated_final = match rec.updated_raw.as_deref() {
            Some(v) if frontmatter::is_timestamp_pub(v) => v.to_string(),
            Some(v) => {
                rec.updated_approx = true;
                rec.warnings.push(format!(
                    "updated_date '{}' is not a valid ISO-8601 UTC timestamp; approximated",
                    v
                ));
                now.clone()
            }
            None => {
                rec.updated_approx = true;
                rec.warnings
                    .push("updated_date missing; approximated".to_string());
                now.clone()
            }
        };

        // assignee already parsed as list.
        if rec.assignee.is_empty() {
            // keep empty
        }

        // milestone translation
        if let Some(src_ms) = &rec.milestone_raw {
            if let Some(tid) = translate_milestone(src_ms, &ms_snapshot, &ms_map) {
                rec.milestone_target = Some(tid);
            } else {
                rec.milestone_target = None;
                rec.milestone_dropped = Some(src_ms.clone());
                rec.warnings.push(format!(
                    "milestone reference '{}' could not be resolved; dropped",
                    src_ms
                ));
            }
        }

        // dependencies translation
        for dep in &rec.dependencies {
            if let Some(tid) = translate_task(dep, &task_snapshot, &task_map) {
                rec.dependencies_target.push(tid);
            } else {
                rec.dropped_deps.push(dep.clone());
                rec.warnings.push(format!(
                    "dependency '{}' could not be resolved; dropped",
                    dep
                ));
            }
        }

        // ordinal
        rec.ordinal_final = match rec.ordinal_raw {
            Some(n) => n,
            None => {
                let v = task_ordinal_counter;
                task_ordinal_counter += 1;
                v
            }
        };

        // dest path
        let dir = if rec.archived {
            task_desc
                .resolve_archive_directory(&cfg, &ctx.project_dir)
                .and_then(|r| r.ok())
                .unwrap_or_else(|| {
                    task_desc
                        .resolve_directory(&cfg, &ctx.project_dir)
                        .unwrap_or_else(|_| ctx.project_dir.clone())
                })
        } else {
            task_desc
                .resolve_directory(&cfg, &ctx.project_dir)
                .unwrap_or_else(|_| ctx.project_dir.clone())
        };
        fs::create_dir_all(&dir).map_err(|e| AppError::Runtime(e.to_string()))?;
        let safe_dir = Context::ensure_within(&ctx.project_dir, &dir)?;
        let raw_path = safe_dir.join(format!("{}.md", rec.target_id));
        rec.dest_path = Context::ensure_within(&ctx.project_dir, &raw_path)?;
    }

    // Process milestones.
    for rec in milestone_records.iter_mut() {
        let (st, mismatch) = normalize_status(&cfg, rec.status_raw.as_deref());
        // milestones may not have status in source; default? We'll normalize if present else keep default? Milestone source has no status field. Set default status? Native milestone has no status front matter! Wait milestone front matter keys: id,title,created,updated,labels,references,release. No status. So don't normalize status for milestones. Oops. Milestones have no status. Set normalized_status unused. We'll not add status. But code above sets; skip.
        let _ = (st, mismatch);
        rec.normalized_status = String::new();
        rec.status_mismatch = false;

        // timestamps
        rec.created_final = match rec.created_raw.as_deref() {
            Some(v) if frontmatter::is_timestamp_pub(v) => v.to_string(),
            Some(v) => {
                rec.created_approx = true;
                rec.warnings.push(format!(
                    "created_date '{}' is not a valid ISO-8601 UTC timestamp; approximated",
                    v
                ));
                now.clone()
            }
            None => {
                rec.created_approx = true;
                rec.warnings
                    .push("created_date missing; approximated".to_string());
                now.clone()
            }
        };
        rec.updated_final = match rec.updated_raw.as_deref() {
            Some(v) if frontmatter::is_timestamp_pub(v) => v.to_string(),
            Some(v) => {
                rec.updated_approx = true;
                rec.warnings.push(format!(
                    "updated_date '{}' is not a valid ISO-8601 UTC timestamp; approximated",
                    v
                ));
                now.clone()
            }
            None => {
                rec.updated_approx = true;
                rec.warnings
                    .push("updated_date missing; approximated".to_string());
                now.clone()
            }
        };

        // ordinal not applicable for milestones; use scan? store maybe. We'll set 0.
        rec.ordinal_final = 0;
        let _ = (ms_ordinal_base, ms_ordinal_counter);

        // dest path
        let dir = if rec.archived {
            // Milestones have no lifecycle archive location; the one-way
            // import is the only writer of the read/import archive folder.
            ms_desc
                .resolve_read_import_directory(&cfg, &ctx.project_dir)
                .and_then(|r| r.ok())
                .unwrap_or_else(|| {
                    ms_desc
                        .resolve_directory(&cfg, &ctx.project_dir)
                        .unwrap_or_else(|_| ctx.project_dir.clone())
                })
        } else {
            ms_desc
                .resolve_directory(&cfg, &ctx.project_dir)
                .unwrap_or_else(|_| ctx.project_dir.clone())
        };
        fs::create_dir_all(&dir).map_err(|e| AppError::Runtime(e.to_string()))?;
        let safe_dir = Context::ensure_within(&ctx.project_dir, &dir)?;
        let raw_path = safe_dir.join(format!("{}.md", rec.target_id));
        rec.dest_path = Context::ensure_within(&ctx.project_dir, &raw_path)?;
    }

    // Status mismatch warning already for tasks; milestone none.

    // Possible duplicate title warnings.
    // Tasks.
    let mut task_title_seen: BTreeMap<String, String> = BTreeMap::new();
    for rec in task_records.iter_mut() {
        for (eid, etitle) in &existing_task_titles {
            if *etitle == rec.title {
                rec.warnings.push(format!(
                    "possible duplicate: {} and {} share the title",
                    rec.target_id, eid
                ));
            }
        }
        if let Some(prev) = task_title_seen.get(&rec.title) {
            rec.warnings.push(format!(
                "possible duplicate: {} and {} share the title",
                rec.target_id, prev
            ));
        }
        task_title_seen.insert(rec.title.clone(), rec.target_id.clone());
    }
    let mut ms_title_seen: BTreeMap<String, String> = BTreeMap::new();
    for rec in milestone_records.iter_mut() {
        for (eid, etitle) in &existing_ms_titles {
            if *etitle == rec.title {
                rec.warnings.push(format!(
                    "possible duplicate: {} and {} share the title",
                    rec.target_id, eid
                ));
            }
        }
        if let Some(prev) = ms_title_seen.get(&rec.title) {
            rec.warnings.push(format!(
                "possible duplicate: {} and {} share the title",
                rec.target_id, prev
            ));
        }
        ms_title_seen.insert(rec.title.clone(), rec.target_id.clone());
    }

    // Build report text.
    let mut report_lines: Vec<String> = Vec::new();
    report_lines.push("Papyrus import backlog".to_string());
    report_lines.push(format!("Source project: {}", from_canon.display()));
    report_lines.push(format!("Source backlog: {}", backlog_canon.display()));
    report_lines.push(format!("Target project: {}", ctx.project_dir.display()));
    if dry_run {
        report_lines.push("Mode: dry-run (no writes)".to_string());
    } else {
        report_lines.push("Mode: import".to_string());
    }
    report_lines.push(format!(
        "Excluded: docs={} drafts={} decisions={} config={} locks={}",
        excluded.docs, excluded.drafts, excluded.decisions, excluded.config, excluded.locks
    ));
    report_lines.push(format!(
        "Planned: tasks={} milestones={}",
        task_records.len(),
        milestone_records.len()
    ));
    report_lines.push(format!("Skipped: {}", skipped.len()));
    if !anomalies.is_empty() {
        report_lines.push("Source anomalies:".to_string());
        for a in &anomalies {
            report_lines.push(format!("  - {}", a));
        }
    }

    report_lines.push("Plan:".to_string());
    for rec in &task_records {
        report_lines.push(format!(
            "  {} <- {} {}{}",
            rec.target_id,
            rec.stem,
            rec.title,
            if rec.archived { " [archived]" } else { "" }
        ));
    }
    for rec in &milestone_records {
        report_lines.push(format!(
            "  {} <- {} {}{}",
            rec.target_id,
            rec.stem,
            rec.title,
            if rec.archived { " [archived]" } else { "" }
        ));
    }

    // Warnings.
    let mut all_warnings: Vec<String> = Vec::new();
    for rec in task_records.iter().chain(milestone_records.iter()) {
        for w in &rec.warnings {
            all_warnings.push(format!("{}: {}", rec.target_id, w));
        }
    }
    if !all_warnings.is_empty() {
        report_lines.push("Warnings:".to_string());
        for w in &all_warnings {
            report_lines.push(format!("  - {}", w));
        }
    }
    if !skipped.is_empty() {
        report_lines.push("Skipped records:".to_string());
        for (p, r) in &skipped {
            report_lines.push(format!("  - {} ({})", p.display(), r));
        }
    }

    let report_text = report_lines.join("\n") + "\n";
    println!("{}", report_text.trim_end());

    // Write report file if requested.
    if let Some(safe) = report_safe {
        let mut f = fs::File::create(&safe)
            .map_err(|e| AppError::Runtime(format!("cannot create report: {}", e)))?;
        f.write_all(report_text.as_bytes())
            .map_err(|e| AppError::Runtime(format!("cannot write report: {}", e)))?;
        f.sync_all().ok();
    }

    // Dry run: stop here.
    if dry_run {
        if skipped.is_empty() {
            return Ok(crate::CommandOutput {
                text: String::new(),
                json: None,
                ..Default::default()
            });
        } else {
            return Err(AppError::Runtime(format!(
                "import plan contains {} skipped record(s)",
                skipped.len()
            )));
        }
    }

    // Phase 3: write.
    let mut written: Vec<String> = Vec::new();

    // Helper closure to write one record.
    let write_one =
        |rec: &mut SrcRecord, cfg: &Config, ctx: &Context, now: &str| -> Result<(), AppError> {
            // Build converted body.
            let body_text = match rec.family {
                Family::Task => convert_task_body(&rec.body),
                Family::Milestone => convert_milestone_body(&rec.body),
            };

            // Parse body into document (no front matter).
            let parsed_doc = document::parse(&body_text)
                .map_err(|e| AppError::Runtime(format!("cannot parse converted body: {}", e)))?;

            // Ensure front matter from source is attached.
            let mut doc = parsed_doc;
            doc.front_matter = Some(rec.fm.clone());
            doc.frontmatter = Some(rec.fm.clone());
            doc.source = body_text.clone();

            // Build ops.
            let mut ops: Vec<PatchOp> = Vec::new();
            ops.push(PatchOp::SetFrontMatterValue(
                "id".to_string(),
                FrontMatterValue::String(rec.target_id.clone()),
            ));
            ops.push(PatchOp::SetFrontMatterValue(
                "title".to_string(),
                FrontMatterValue::String(rec.title.clone()),
            ));
            ops.push(PatchOp::SetFrontMatterValue(
                "created_date".to_string(),
                FrontMatterValue::String(rec.created_final.clone()),
            ));
            ops.push(PatchOp::SetFrontMatterValue(
                "updated_date".to_string(),
                FrontMatterValue::String(rec.updated_final.clone()),
            ));

            match rec.family {
                Family::Task => {
                    // labels: patch if present? They are in source raw already; but ensure key exists? Source has labels. Keep.
                    // status
                    ops.push(PatchOp::SetFrontMatterValue(
                        "status".to_string(),
                        FrontMatterValue::String(rec.normalized_status.clone()),
                    ));
                    // assignee list
                    let assignee_val = if rec.assignee.is_empty() {
                        "[]".to_string()
                    } else {
                        rec.assignee.join("\n")
                    };
                    ops.push(PatchOp::SetFrontMatterValue(
                        "assignee".to_string(),
                        FrontMatterValue::String(assignee_val),
                    ));
                    // milestone
                    match &rec.milestone_target {
                        Some(m) => {
                            ops.push(PatchOp::SetFrontMatterValue(
                                "milestone".to_string(),
                                FrontMatterValue::String(m.clone()),
                            ));
                        }
                        None => {
                            ops.push(PatchOp::SetFrontMatterValue(
                                "milestone".to_string(),
                                FrontMatterValue::Null,
                            ));
                        }
                    }
                    // dependencies list
                    let deps_val = if rec.dependencies_target.is_empty() {
                        "[]".to_string()
                    } else {
                        rec.dependencies_target.join("\n")
                    };
                    ops.push(PatchOp::SetFrontMatterValue(
                        "dependencies".to_string(),
                        FrontMatterValue::String(deps_val),
                    ));
                    // ordinal
                    ops.push(PatchOp::SetFrontMatterValue(
                        "ordinal".to_string(),
                        FrontMatterValue::Integer(rec.ordinal_final),
                    ));
                    // type: copy if present
                    if let Some(t) = &rec.type_raw {
                        ops.push(PatchOp::SetFrontMatterValue(
                            "type".to_string(),
                            FrontMatterValue::String(t.clone()),
                        ));
                    }
                    // labels/references/documentation already in source raw; no need to patch.
                    let _ = cfg;
                }
                Family::Milestone => {
                    // release: ensure null (not written). Clear if exists.
                    ops.push(PatchOp::SetFrontMatterValue(
                        "release".to_string(),
                        FrontMatterValue::Null,
                    ));
                    // labels/references already in source raw.
                }
            }

            // Remove source id? id patched already.
            // Clear heading (tasks/milestones no heading).
            ops.push(PatchOp::ClearHeading);

            let file_content = apply_patch_fn(&doc, &ops)
                .map_err(|e| AppError::Runtime(format!("cannot build record content: {}", e)))?;

            // Collision check.
            if rec.dest_path.exists() {
                return Err(AppError::Runtime(format!(
                    "destination already exists: {}",
                    rec.dest_path.display()
                )));
            }

            store::write_atomic(ctx, &rec.dest_path, &file_content)?;
            let _ = now;
            Ok(())
        };

    // Write tasks first.
    for rec in task_records.iter_mut() {
        if let Err(e) = write_one(rec, &cfg, ctx, &now) {
            // abort with summary
            let mut msg = format!("import aborted: {}", e);
            msg.push_str(&format!(
                "\nRecords written before failure: {}",
                written.len()
            ));
            for w in &written {
                msg.push_str(&format!("\n  - {}", w));
            }
            return Err(AppError::Runtime(msg));
        }
        written.push(rec.target_id.clone());
    }
    // Write milestones.
    for rec in milestone_records.iter_mut() {
        if let Err(e) = write_one(rec, &cfg, ctx, &now) {
            let mut msg = format!("import aborted: {}", e);
            msg.push_str(&format!(
                "\nRecords written before failure: {}",
                written.len()
            ));
            for w in &written {
                msg.push_str(&format!("\n  - {}", w));
            }
            return Err(AppError::Runtime(msg));
        }
        written.push(rec.target_id.clone());
    }

    if skipped.is_empty() {
        Ok(crate::CommandOutput {
            text: String::new(),
            json: None,
            ..Default::default()
        })
    } else {
        Err(AppError::Runtime(format!(
            "import completed with {} skipped record(s)",
            skipped.len()
        )))
    }
}

// ---------- helpers ----------

fn count_excluded(backlog: &Path) -> ExcludedCounts {
    let mut c = ExcludedCounts::default();
    let dirs = [
        ("docs", backlog.join("docs")),
        ("drafts", backlog.join("drafts")),
        ("drafts", backlog.join("archive").join("drafts")),
        ("decisions", backlog.join("decisions")),
    ];
    for (kind, dir) in dirs {
        let n = count_files(&dir);
        match kind {
            "docs" => c.docs += n,
            "drafts" => c.drafts += n,
            "decisions" => c.decisions += n,
            _ => {}
        }
    }
    let config_path = backlog.join("config.yml");
    if config_path.exists() {
        c.config += 1;
    }
    let locks_dir = backlog.join(".locks");
    c.locks += count_files(&locks_dir);
    c
}

fn count_files(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                count += 1;
            } else if p.is_dir() {
                count += count_files(&p);
            }
        }
    }
    count
}

fn parse_source_number(family: Family, stem: &str) -> Option<i64> {
    let prefix = match family {
        Family::Task => "task",
        Family::Milestone => "m",
    };
    let base = stem.split(" - ").next().unwrap_or(stem).trim();
    let (p, rest) = base.split_once('-')?;
    if !p.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let remainder = &rest[digits.len()..];
    if !remainder.is_empty() && !remainder.starts_with('-') {
        return None;
    }
    digits.parse::<i64>().ok()
}

fn yaml_string(fm: &FrontMatter, key: &str) -> Option<String> {
    match fm.data.get(yaml_serde::Value::String(key.to_string())) {
        Some(yaml_serde::Value::String(s)) => Some(s.clone()),
        Some(yaml_serde::Value::Number(n)) => Some(n.to_string()),
        Some(yaml_serde::Value::Bool(b)) => Some(b.to_string()),
        Some(yaml_serde::Value::Null) => None,
        _ => None,
    }
}

fn yaml_seq_strings(fm: &FrontMatter, key: &str) -> Vec<String> {
    match fm.data.get(yaml_serde::Value::String(key.to_string())) {
        Some(yaml_serde::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| match v {
                yaml_serde::Value::String(s) => Some(s.clone()),
                yaml_serde::Value::Number(n) => Some(n.to_string()),
                yaml_serde::Value::Bool(b) => Some(b.to_string()),
                _ => None,
            })
            .collect(),
        Some(yaml_serde::Value::String(s)) if !s.trim().is_empty() => {
            vec![s.clone()]
        }
        _ => Vec::new(),
    }
}

fn normalize_status(cfg: &Config, raw: Option<&str>) -> (String, bool) {
    let raw = raw.unwrap_or(&cfg.default_status).trim().to_string();
    let norm = |s: &str| s.to_lowercase().replace([' ', '-', '_'], "");
    for s in &cfg.statuses {
        if norm(s) == norm(&raw) {
            return (s.clone(), false);
        }
    }
    (raw, true)
}

fn extract_number(s: &str) -> Option<i64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i64>().ok()
    }
}

fn translate_task(
    src: &str,
    records: &[SrcRecord],
    map: &BTreeMap<String, String>,
) -> Option<String> {
    if let Some(t) = map.get(src) {
        return Some(t.clone());
    }
    for rec in records {
        if rec.source_id.eq_ignore_ascii_case(src) || rec.stem.eq_ignore_ascii_case(src) {
            return Some(rec.target_id.clone());
        }
    }
    // match by number
    if let Some(n) = extract_number(src) {
        for rec in records {
            if rec.source_number == n {
                return Some(rec.target_id.clone());
            }
        }
    }
    None
}

fn translate_milestone(
    src: &str,
    records: &[SrcRecord],
    map: &BTreeMap<String, String>,
) -> Option<String> {
    if let Some(t) = map.get(src) {
        return Some(t.clone());
    }
    for rec in records {
        if rec.source_id.eq_ignore_ascii_case(src) || rec.stem.eq_ignore_ascii_case(src) {
            return Some(rec.target_id.clone());
        }
    }
    if let Some(n) = extract_number(src) {
        for rec in records {
            if rec.source_number == n {
                return Some(rec.target_id.clone());
            }
        }
    }
    None
}

fn collect_existing_titles(
    _ctx: &Context,
    cfg: &Config,
    kind: &str,
    dirs: &[PathBuf],
) -> Vec<(String, String)> {
    let desc = match kind {
        "task" => descriptor_for("task"),
        "milestone" => descriptor_for("milestone"),
        _ => None,
    };
    let Some(desc) = desc else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let prefix = match kind {
        "task" => cfg.task_prefix.clone(),
        "milestone" => cfg.milestone_prefix.clone(),
        _ => String::new(),
    };
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(rd) = fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };
            if identity::id_number(stem, &prefix).is_none() {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Some(fm) = frontmatter::parse(&content).ok().flatten() else {
                continue;
            };
            let title = yaml_string(&fm, "title").unwrap_or_default();
            if title.is_empty() {
                continue;
            }
            let id = yaml_string(&fm, "id").unwrap_or_else(|| stem.to_string());
            let _ = desc;
            out.push((id, title));
        }
    }
    out
}

// ---------- body conversion ----------

#[derive(Clone)]
enum Block {
    Section { heading: String, body: Vec<String> },
    Verbatim(Vec<String>),
}

fn convert_task_body(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut blocks: Vec<Block> = Vec::new();
    let mut verbatim: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(kind) = begin_marker(trimmed) {
            // flush verbatim
            if !verbatim.is_empty() {
                blocks.push(Block::Verbatim(std::mem::take(&mut verbatim)));
            }
            i += 1;
            let mut buf: Vec<&str> = Vec::new();
            while i < lines.len() && !is_end_marker(lines[i].trim(), kind) {
                buf.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip end marker
            }
            // trim outer blank lines
            while buf.first().is_some_and(|l| l.trim().is_empty()) {
                buf.remove(0);
            }
            while buf.last().is_some_and(|l| l.trim().is_empty()) {
                buf.pop();
            }
            let body_lines: Vec<String> = buf.iter().map(|l| l.to_string()).collect();
            let (heading, converted) = match kind {
                MarkerKind::Description => ("## Description".to_string(), body_lines),
                MarkerKind::Ac => {
                    let mut out = Vec::new();
                    for l in &body_lines {
                        out.push(strip_ac_ordinal(l));
                    }
                    ("## Acceptance Criteria".to_string(), out)
                }
                MarkerKind::Dod => ("## Definition of Done".to_string(), body_lines),
                MarkerKind::Plan => ("## Implementation Notes".to_string(), body_lines),
                MarkerKind::Notes => ("## Notes".to_string(), body_lines),
                MarkerKind::Comments => ("## Comments".to_string(), body_lines),
            };
            blocks.push(Block::Section {
                heading,
                body: converted,
            });
        } else if is_any_end_marker(trimmed) || heading_precedes_marker(&lines, i) {
            i += 1;
        } else {
            verbatim.push(lines[i].to_string());
            i += 1;
        }
    }
    if !verbatim.is_empty() {
        blocks.push(Block::Verbatim(verbatim));
    }

    // Render blocks.
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Section { heading, body } => {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                // ensure blank line before heading if content present
                if !out.is_empty() && !out.ends_with("\n\n") {
                    out.push('\n');
                }
                out.push_str(&heading);
                out.push('\n');
                let body_str = body.join("\n");
                if !body_str.trim().is_empty() {
                    out.push_str(&body_str);
                    out.push('\n');
                }
            }
            Block::Verbatim(lines) => {
                // append verbatim lines
                for l in lines {
                    out.push_str(&l);
                    out.push('\n');
                }
            }
        }
    }
    out
}

fn convert_milestone_body(src: &str) -> String {
    // Remove marker comment lines.
    let mut lines: Vec<String> = src
        .lines()
        .filter(|l| !is_marker_comment(l.trim()))
        .map(|l| l.to_string())
        .collect();
    // Trim leading/trailing blank lines.
    while lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    // Strip a leading "## Description" heading if present, to avoid duplication.
    if lines
        .first()
        .map(|l| l.trim() == "## Description")
        .unwrap_or(false)
    {
        lines.remove(0);
        while lines.first().is_some_and(|l| l.trim().is_empty()) {
            lines.remove(0);
        }
    }
    let text = lines.join(
        "
",
    );
    if text.trim().is_empty() {
        "## Description
"
        .to_string()
    } else {
        format!(
            "## Description

{}
",
            text.trim_end()
        )
    }
}

fn is_marker_comment(line: &str) -> bool {
    line.starts_with("<!--") && line.contains(":BEGIN")
        || line.starts_with("<!--") && line.contains(":END")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerKind {
    Description,
    Ac,
    Dod,
    Plan,
    Notes,
    Comments,
}

fn begin_marker(trimmed: &str) -> Option<MarkerKind> {
    if trimmed.contains("SECTION:DESCRIPTION:BEGIN") {
        Some(MarkerKind::Description)
    } else if trimmed.contains("AC:BEGIN") {
        Some(MarkerKind::Ac)
    } else if trimmed.contains("DOD:BEGIN") {
        Some(MarkerKind::Dod)
    } else if trimmed.contains("SECTION:PLAN:BEGIN") {
        Some(MarkerKind::Plan)
    } else if trimmed.contains("SECTION:NOTES:BEGIN") {
        Some(MarkerKind::Notes)
    } else if trimmed.contains("COMMENTS:BEGIN") {
        Some(MarkerKind::Comments)
    } else {
        None
    }
}

fn is_end_marker(trimmed: &str, kind: MarkerKind) -> bool {
    match kind {
        MarkerKind::Description => trimmed.contains("SECTION:DESCRIPTION:END"),
        MarkerKind::Ac => trimmed.contains("AC:END"),
        MarkerKind::Dod => trimmed.contains("DOD:END"),
        MarkerKind::Plan => trimmed.contains("SECTION:PLAN:END"),
        MarkerKind::Notes => trimmed.contains("SECTION:NOTES:END"),
        MarkerKind::Comments => trimmed.contains("COMMENTS:END"),
    }
}

fn is_any_end_marker(trimmed: &str) -> bool {
    trimmed.contains(":END -->")
        || trimmed.contains("AC:END")
        || trimmed.contains("DOD:END")
        || trimmed.contains("COMMENTS:END")
}

#[allow(unused_assignments)]
fn strip_ac_ordinal(line: &str) -> String {
    let t = line.trim_end();
    // Preserve checkbox prefix.
    let trimmed = t.trim_start();
    let indent = &t[..t.len() - trimmed.len()];
    // find checkbox
    let mut rest = trimmed;
    let mut prefix = String::new();
    if let Some(r) = rest.strip_prefix("- [ ]") {
        prefix = "- [ ]".to_string();
        rest = r.trim_start();
    } else if let Some(r) = rest.strip_prefix("- [x]") {
        prefix = "- [x]".to_string();
        rest = r.trim_start();
    } else if let Some(r) = rest.strip_prefix("- [X]") {
        prefix = "- [X]".to_string();
        rest = r.trim_start();
    } else {
        return t.to_string();
    }
    // remove leading #N
    let mut chars = rest.chars().peekable();
    if chars.peek() == Some(&'#') {
        chars.next();
        // consume digits
        while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            chars.next();
        }
        // consume following space
        if chars.peek() == Some(&' ') {
            chars.next();
        }
    }
    let new_rest: String = chars.collect();
    if prefix.is_empty() {
        t.to_string()
    } else if new_rest.is_empty() {
        format!("{}{}", indent, prefix)
    } else {
        format!("{}{} {}", indent, prefix, new_rest)
    }
}

fn heading_precedes_marker(lines: &[&str], i: usize) -> bool {
    let heading = lines[i].trim();
    if !heading.starts_with("## ") {
        return false;
    }
    let mut j = i + 1;
    while j < lines.len() && lines[j].trim().is_empty() {
        j += 1;
    }
    if j >= lines.len() {
        return false;
    }
    // Drop any heading that immediately precedes a recognized marker block;
    // the converter emits its own target heading for that marker.
    begin_marker(lines[j].trim()).is_some()
}
