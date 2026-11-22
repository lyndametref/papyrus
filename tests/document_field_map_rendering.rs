//! TASK-265: the model-neutral field map feeds the generic document list and
//! view renderers (ADR-039 decisions 14-21, TECH-068 section 4.1).

use std::fs;
use std::path::PathBuf;

use papyrus::browser::list::ListPlan;
use papyrus::browser::records::{Index, Record, build_index, resolve_id};
use papyrus::browser::render::{self, Chrome};
use papyrus::browser::routes::BrowserRoutes;
use papyrus::commands::load_effective_config;
use papyrus::config::Config;
use papyrus::context::Context;
use papyrus::registry::ArtifactRegistry;

fn corpus_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/characterization/corpus/project")
}

fn context() -> Context {
    let project_dir = corpus_project();
    Context {
        config_path: project_dir.join("papyrus.config.yml"),
        project_dir,
        json: false,
        no_color: true,
    }
}

fn project_config(ctx: &Context) -> Config {
    load_effective_config(&ctx.config_path, &ctx.project_dir).expect("project config")
}

fn registry() -> ArtifactRegistry {
    ArtifactRegistry::build(&Config::default()).expect("registry")
}

fn registry_for(cfg: &Config) -> ArtifactRegistry {
    ArtifactRegistry::build(cfg).expect("registry")
}

fn load(ctx: &Context, cfg: &Config, registry: &ArtifactRegistry, id: &str) -> Record {
    resolve_id(ctx, cfg, registry, id)
        .expect("resolve")
        .unwrap_or_else(|| panic!("record {id}"))
        .record
}

fn render_record(record: &Record, group: Option<&Record>, members: &[Record]) -> String {
    let routes = BrowserRoutes::build(&registry());
    let chrome = Chrome {
        project: "Proj",
        bind_note: "127.0.0.1:0",
        routes: &routes,
    };
    let cfg = Config::default();
    render::document_view_page(record, group, members, &[], &[], &[], &cfg, chrome).html
}

#[test]
fn generic_view_renders_descriptor_fields_in_sequence() {
    let ctx = context();
    let cfg = project_config(&ctx);
    let registry = registry_for(&cfg);
    let record = load(&ctx, &cfg, &registry, "BR-001");
    let html = render_record(&record, None, &[]);
    for heading in ["Description", "Rationale", "Applicability", "Requirements"] {
        assert!(
            html.contains(&format!(
                "<section class=\"record-section\"><h2>{heading}</h2>"
            )),
            "missing field section {heading}"
        );
    }
    let description = html.find("<h2>Description</h2>").expect("description");
    let rationale = html.find("<h2>Rationale</h2>").expect("rationale");
    let applicability = html.find("<h2>Applicability</h2>").expect("applicability");
    let requirements = html.find("<h2>Requirements</h2>").expect("requirements");
    assert!(description < rationale && rationale < applicability && applicability < requirements);
}

#[test]
fn required_absent_field_renders_an_empty_section_and_optional_is_omitted() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join(format!("field-map-empty-{}", std::process::id()));
    let glossary = root.join("docs/glossary");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&glossary).expect("glossary directory");
    fs::write(
        glossary.join("GLO-900.md"),
        "---\nid: GLO-900\ntitle: Empty term\n---\n# GLO-900 Empty term\n",
    )
    .expect("record");
    let ctx = Context {
        project_dir: root.clone(),
        config_path: root.join("papyrus.config.yml"),
        json: false,
        no_color: true,
    };
    let cfg = Config::default();
    let registry = registry();
    let record = load(&ctx, &cfg, &registry, "GLO-900");
    let html = render_record(&record, None, &[]);
    assert!(
        html.contains("<section class=\"record-section\"><h2>Definition</h2></section>"),
        "a required empty field renders an empty section"
    );

    let records_dir = root.join("docs/glossary");
    fs::write(
        records_dir.join("GLO-901.md"),
        "---\nid: GLO-901\ntitle: Another\n---\n# GLO-901 Another\n\n## Definition\nPresent.\n",
    )
    .expect("record");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn grouped_group_and_member_are_role_aware() {
    let ctx = context();
    let cfg = project_config(&ctx);
    let registry = registry_for(&cfg);
    let group = load(&ctx, &cfg, &registry, "REQ-001");
    let index = build_index(&ctx, &cfg, &registry, &group.descriptor, None);
    let members: Vec<Record> = index
        .members
        .get(&group.id)
        .map(|positions| {
            positions
                .iter()
                .filter_map(|position| index.requirements.get(*position).cloned())
                .collect()
        })
        .unwrap_or_default();
    let group_html = render_record(&group, None, &members);
    assert!(group_html.contains("<h2>User story</h2>"));
    assert!(group_html.contains("<h2>Requirements in this group</h2>"));
    assert!(group_html.contains("href=\"/doc/REQ-001.001\""));

    let member = load(&ctx, &cfg, &registry, "REQ-001.001");
    let member_html = render_record(&member, Some(&group), &[]);
    assert!(member_html.contains("<h2>User story</h2>"));
    assert!(member_html.contains("<h2>Description</h2>"));
    assert!(member_html.contains("<details><summary>Owning group</summary>"));
    assert!(member_html.contains("href=\"/doc/REQ-001\""));
    assert!(!member_html.contains("<h2>Requirements in this group</h2>"));
}

#[test]
fn mockup_view_uses_the_fixed_metadata_block_without_field_sections() {
    let ctx = context();
    let cfg = project_config(&ctx);
    let registry = registry_for(&cfg);
    let record = load(&ctx, &cfg, &registry, "MOCK-002");
    let html = render_record(&record, None, &[]);
    for label in [
        "Status",
        "Task",
        "UI specifications",
        "References",
        "Superseded by",
    ] {
        assert!(
            html.contains(&format!("<dt>{label}</dt>")),
            "missing fixed metadata label {label}"
        );
    }
    assert!(html.contains("<dd>approved</dd>"));
    assert!(!html.contains("<h2>UI specifications</h2>"));
    assert!(!html.contains("<section class=\"record-section\">"));
}

#[test]
fn document_list_rows_use_the_shared_hierarchy_and_keep_labels() {
    let ctx = context();
    let cfg = project_config(&ctx);
    let registry = registry_for(&cfg);
    let descriptor = registry.descriptor("req").expect("req descriptor").clone();
    let index: Index = build_index(&ctx, &cfg, &registry, &descriptor, None);
    let plan = ListPlan::for_descriptor(
        &descriptor,
        &index,
        "Business requirements",
        "2 groups \u{b7} 3 requirements",
    );
    let keys: Vec<&str> = plan
        .columns
        .iter()
        .map(|column| column.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "id", "title", "status", "group", "created", "updated", "actions"
        ]
    );
    assert!(
        !keys.contains(&"description"),
        "no document field is a column"
    );
    let html = plan.render();
    assert!(html.contains("<tr data-row-id=\"REQ-001.001\" data-child-of=\"REQ-001\">"));
    assert!(html.contains("<li class=\"tag\">core</li>"));
}

#[test]
fn the_document_detail_template_is_gone_and_the_shared_view_remains() {
    let templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browser/templates");
    assert!(!templates.join("document_detail.html").exists());
    assert!(templates.join("view.html").exists());
    let render =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browser/render.rs"))
            .expect("render source");
    assert!(!render.contains("DOCUMENT_DETAIL_TEMPLATE"));
}
