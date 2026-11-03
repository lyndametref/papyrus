//! `papyrus browser`: a project-local, offline HTTP interface over the
//! requirement content implemented so far.
//!
//! The behavior contract is `docs/specifications/browser-requirements.md`.

pub mod http;
pub mod records;
pub mod render;

use crate::CommandOutput;
use crate::commands::load_effective_config;
use crate::config::{Config, is_loopback_host, validate_browser_host};
use crate::context::Context;
use crate::error::AppError;
use crate::requirement::Kind;
use http::{HttpError, Method, Request, Response};
use records::{FieldChange, Form, Record};
use std::io::Write;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const MAX_CONNECTIONS: usize = 16;
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const APP_CSS: &str = include_str!("app.css");
const APP_JS: &str = include_str!("app.js");

/// Effective browser settings after applying the precedence chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserOptions {
    pub host: String,
    pub port: u16,
    pub open: bool,
}

/// Environment tier values. Empty values are treated as unset.
#[derive(Clone, Debug, Default)]
pub struct EnvValues {
    pub host: Option<String>,
    pub port: Option<String>,
    pub open: Option<String>,
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn read_env() -> EnvValues {
    EnvValues {
        host: env_value("PAPYRUS_BROWSER_HOST"),
        port: env_value("PAPYRUS_BROWSER_PORT"),
        open: env_value("PAPYRUS_BROWSER_OPEN"),
    }
}

/// Resolve the effective host, port, and open setting.
///
/// Precedence: command line, then environment, then configuration, then the
/// built-in defaults already present in [`Config`]. Each setting resolves
/// independently.
pub fn resolve_options(
    cli_host: Option<&str>,
    cli_port: Option<u16>,
    no_open: bool,
    env: &EnvValues,
    cfg: &Config,
) -> Result<BrowserOptions, AppError> {
    let host = match cli_host {
        Some(host) => host.to_string(),
        None => match &env.host {
            Some(host) => host.clone(),
            None => cfg.browser_host.clone(),
        },
    };
    validate_browser_host(&host)
        .map_err(|reason| AppError::Validation(format!("invalid host {}: {}", host, reason)))?;

    let port = match cli_port {
        Some(port) => port,
        None => match &env.port {
            Some(raw) => raw.parse::<u16>().map_err(|_| {
                AppError::Validation(format!("invalid PAPYRUS_BROWSER_PORT: {}", raw))
            })?,
            None => cfg.browser_port,
        },
    };
    if port == 0 {
        return Err(AppError::Validation(
            "invalid PAPYRUS_BROWSER_PORT: 0".to_string(),
        ));
    }

    let open = if no_open {
        false
    } else {
        match &env.open {
            Some(raw) if raw.eq_ignore_ascii_case("true") => true,
            Some(raw) if raw.eq_ignore_ascii_case("false") => false,
            Some(raw) => {
                return Err(AppError::Validation(format!(
                    "invalid PAPYRUS_BROWSER_OPEN: {}",
                    raw
                )));
            }
            None => cfg.browser_open,
        }
    };

    Ok(BrowserOptions { host, port, open })
}

fn display_host(host: &str) -> String {
    if host == "0.0.0.0" || host == "::" {
        return "127.0.0.1".to_string();
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V6(_)) => format!("[{}]", host),
        _ => host.to_string(),
    }
}

struct ServerState {
    project_dir: PathBuf,
    config_path: PathBuf,
    host: String,
    port: u16,
}

impl ServerState {
    fn context(&self) -> Context {
        Context {
            project_dir: self.project_dir.clone(),
            config_path: self.config_path.clone(),
            json: false,
            no_color: true,
        }
    }

    fn bind_note(&self) -> String {
        format!("{}:{}", display_host(&self.host), self.port)
    }

    fn config(&self) -> Result<Config, AppError> {
        load_effective_config(&self.config_path, &self.project_dir)
    }
}

/// Run the browser command until the process is interrupted.
pub fn run(
    ctx: &Context,
    cli_host: Option<&str>,
    cli_port: Option<u16>,
    no_open: bool,
) -> Result<CommandOutput, AppError> {
    let cfg = load_effective_config(&ctx.config_path, &ctx.project_dir)?;
    let options = resolve_options(cli_host, cli_port, no_open, &read_env(), &cfg)?;

    if !is_loopback_host(&options.host) {
        eprintln!(
            "warning: binding to non-loopback address {}; the interface is reachable from the network",
            options.host
        );
    }

    let listener = TcpListener::bind((options.host.as_str(), options.port)).map_err(|error| {
        AppError::Runtime(format!(
            "cannot bind {}:{}: {}",
            options.host, options.port, error
        ))
    })?;

    let url = format!("http://{}:{}/", display_host(&options.host), options.port);
    if ctx.json {
        let startup = serde_json::json!({
            "status": "serving",
            "url": url,
            "host": options.host,
            "port": options.port,
            "open": options.open,
        });
        println!("{}", startup);
    } else {
        println!("Serving {}", url);
    }
    std::io::stdout()
        .flush()
        .map_err(|error| AppError::Runtime(error.to_string()))?;

    if options.open
        && let Err(reason) = open_browser(&url)
    {
        eprintln!("warning: could not open the default browser: {}", reason);
    }

    let state = Arc::new(ServerState {
        project_dir: ctx.project_dir.clone(),
        config_path: ctx.config_path.clone(),
        host: options.host.clone(),
        port: options.port,
    });
    serve(listener, state)?;
    Ok(CommandOutput {
        text: String::new(),
        json: None,
    })
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let (program, arguments): (&str, Vec<&str>) = ("cmd", vec!["/C", "start", "", url]);
    #[cfg(target_os = "macos")]
    let (program, arguments): (&str, Vec<&str>) = ("open", vec![url]);
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let (program, arguments): (&str, Vec<&str>) = ("xdg-open", vec![url]);

    std::process::Command::new(program)
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

struct ActiveGuard(Arc<AtomicUsize>);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn serve(listener: TcpListener, state: Arc<ServerState>) -> Result<(), AppError> {
    let active = Arc::new(AtomicUsize::new(0));
    loop {
        let (stream, _) = listener
            .accept()
            .map_err(|error| AppError::Runtime(format!("cannot accept connection: {}", error)))?;
        let current = active.fetch_add(1, Ordering::SeqCst);
        if current >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::SeqCst);
            reject_over_capacity(stream);
            continue;
        }
        let state = Arc::clone(&state);
        let active = Arc::clone(&active);
        std::thread::spawn(move || {
            let _guard = ActiveGuard(active);
            handle_connection(stream, &state);
        });
    }
}

/// Reject a connection accepted beyond the concurrency cap.
///
/// The request is never read, so the accept loop cannot be blocked by a slow
/// or partial client. The connection immediately receives `503` with
/// `Connection: close`, is closed, and is logged as `- - 503` (specification
/// sections 2.12 and 3.2). Host and Origin validation do not apply because no
/// request has been parsed.
fn reject_over_capacity(mut stream: TcpStream) {
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let response = Response::text(503, "text/plain; charset=utf-8", "Service Unavailable\n");
    let _ = http::write_response(&mut stream, &response, false);
    eprintln!("- - 503");
}

fn handle_connection(mut stream: TcpStream, state: &ServerState) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));

    let request = match http::read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("- - {}", error.status);
            let response = error_response(error.status, false, state);
            let _ = http::write_response(&mut stream, &response, false);
            return;
        }
    };

    let method = request.method.as_str().to_string();
    let log_path: String = request.target.path.chars().take(200).collect();
    let head_only = request.method == Method::Head;
    let api = request.target.path.starts_with("/api/");

    let response = match validate_host(&request, state) {
        Ok(host_header) => match validate_origin(&request, &host_header) {
            Ok(()) => handle_request(request, state),
            Err(error) => error_response(error.status, api, state),
        },
        Err(error) => error_response(error.status, api, state),
    };

    eprintln!("{} {} {}", method, log_path, response.status);
    let _ = http::write_response(&mut stream, &response, head_only);
}

fn allowed_host_values(state: &ServerState) -> Vec<String> {
    let effective = state.host.to_ascii_lowercase();
    let port = state.port;
    let mut values = vec![
        format!("{}:{}", effective, port),
        format!("localhost:{}", port),
        format!("127.0.0.1:{}", port),
        format!("[::1]:{}", port),
    ];
    if effective
        .parse::<IpAddr>()
        .map(|ip| ip.is_ipv6())
        .unwrap_or(false)
    {
        values.push(format!("[{}]:{}", effective, port));
    }
    if port == 80 {
        values.push(effective);
        values.push("localhost".to_string());
        values.push("127.0.0.1".to_string());
        values.push("[::1]".to_string());
    }
    values
}

fn validate_host(request: &Request, state: &ServerState) -> Result<String, HttpError> {
    if request.header_count("host") != 1 {
        return Err(HttpError::new(400));
    }
    let host = request.header("host").unwrap_or("").trim().to_string();
    if host.is_empty() {
        return Err(HttpError::new(400));
    }
    let lower = host.to_ascii_lowercase();
    if allowed_host_values(state).contains(&lower) {
        Ok(host)
    } else {
        Err(HttpError::new(400))
    }
}

fn validate_origin(request: &Request, host_header: &str) -> Result<(), HttpError> {
    if request.method != Method::Post {
        return Ok(());
    }
    let Some(origin) = request.header("origin") else {
        return Ok(());
    };
    let origin = origin.to_ascii_lowercase();
    let Some(rest) = origin.strip_prefix("http://") else {
        return Err(HttpError::new(403));
    };
    let host = host_header.to_ascii_lowercase();
    if rest == host {
        return Ok(());
    }
    if request.version == http::Version::Http10 || request.version == http::Version::Http11 {
        // The same host value that passed the Host check, allowing an omitted
        // or explicit `:80` suffix for the default HTTP port.
        if rest == format!("{}:80", host) {
            return Ok(());
        }
        if let Some(stripped) = host.strip_suffix(":80")
            && rest == stripped
        {
            return Ok(());
        }
    }
    Err(HttpError::new(403))
}

fn error_message(status: u16) -> &'static str {
    match status {
        409 => "record changed on disk; reload the form",
        other => render::error_copy(other).1,
    }
}

fn error_response(status: u16, api: bool, state: &ServerState) -> Response {
    let cfg = state.config().unwrap_or_default();
    let project = cfg.effective_project_name(&state.project_dir);
    let bind_note = state.bind_note();
    let chrome = render::Chrome {
        project: &project,
        bind_note: &bind_note,
    };
    if api {
        let body = serde_json::json!({
            "error": {
                "code": status,
                "message": error_message(status),
            }
        });
        let mut response = Response::json(status, body);
        if status == 405 {
            response = response.with_header("Allow", "GET, HEAD, POST");
        }
        response
    } else {
        let mut response = Response::html(status, render::error_page(status, &cfg, chrome, None));
        if status == 405 {
            response = response.with_header("Allow", "GET, HEAD, POST");
        }
        response
    }
}

fn json_error(status: u16, message: &str) -> Response {
    Response::json(
        status,
        serde_json::json!({
            "error": {
                "code": status,
                "message": message,
            }
        }),
    )
}

fn category_from_query(request: &Request) -> Result<Kind, HttpError> {
    match http::query_param(&request.target.query, "type")? {
        None => Ok(Kind::Req),
        Some(value) if value == "req" => Ok(Kind::Req),
        Some(value) if value == "nfr" => Ok(Kind::Nfr),
        Some(_) => Err(HttpError::new(400)),
    }
}

fn chrome_for(cfg: &Config, state: &ServerState) -> (String, String) {
    (
        cfg.effective_project_name(&state.project_dir),
        state.bind_note(),
    )
}

fn handle_request(request: Request, state: &ServerState) -> Response {
    let api = request.target.path.starts_with("/api/");
    let result = route(request, state);
    match result {
        Ok(response) => response,
        Err(error) => error_response(error.status, api, state),
    }
}

fn route(request: Request, state: &ServerState) -> Result<Response, HttpError> {
    if request.method == Method::Other {
        return Err(HttpError::new(405));
    }
    let path = request.target.path.clone();
    let method = request.method;

    if path == "/assets/app.css" {
        if method == Method::Post {
            return Err(HttpError::new(405));
        }
        return Ok(Response::text(200, "text/css; charset=utf-8", APP_CSS));
    }
    if path == "/assets/app.js" {
        if method == Method::Post {
            return Err(HttpError::new(405));
        }
        return Ok(Response::text(
            200,
            "text/javascript; charset=utf-8",
            APP_JS,
        ));
    }

    if path == "/api/requirements" {
        if method == Method::Post {
            return Err(HttpError::new(405));
        }
        return api_requirements(&request, state);
    }
    if let Some(id) = path.strip_prefix("/api/doc/") {
        if id.is_empty() || id.contains('/') {
            return Err(HttpError::new(404));
        }
        return match method {
            Method::Get | Method::Head => api_get(id, state),
            Method::Post => api_post(id, request, state),
            Method::Other => Err(HttpError::new(405)),
        };
    }

    if path == "/" {
        if method == Method::Post {
            return Err(HttpError::new(405));
        }
        return list_page(&request, state);
    }
    if let Some(rest) = path.strip_prefix("/doc/") {
        if let Some(id) = rest.strip_suffix("/edit") {
            if id.is_empty() || id.contains('/') {
                return Err(HttpError::new(404));
            }
            if method == Method::Post {
                return Err(HttpError::new(405));
            }
            return edit_page(id, state);
        }
        if rest.is_empty() || rest.contains('/') {
            return Err(HttpError::new(404));
        }
        return match method {
            Method::Get | Method::Head => view_page(rest, state),
            Method::Post => form_post(rest, request, state),
            Method::Other => Err(HttpError::new(405)),
        };
    }

    Err(HttpError::new(404))
}

fn list_page(request: &Request, state: &ServerState) -> Result<Response, HttpError> {
    let kind = category_from_query(request)?;
    let cfg = state.config().map_err(|_| HttpError::new(500))?;
    let ctx = state.context();
    let index = records::build_index(&ctx, &cfg, kind);
    let other_kind = match kind {
        Kind::Req => Kind::Nfr,
        Kind::Nfr => Kind::Req,
    };
    let other_has_records = !records::build_index(&ctx, &cfg, other_kind).is_empty();
    let (project, bind_note) = chrome_for(&cfg, state);
    let chrome = render::Chrome {
        project: &project,
        bind_note: &bind_note,
    };
    Ok(Response::html(
        200,
        render::list_page(kind, &index, other_has_records, &cfg, chrome),
    ))
}

fn resolve_record(id: &str, state: &ServerState) -> Result<(Record, Config), HttpError> {
    let cfg = state.config().map_err(|_| HttpError::new(500))?;
    let ctx = state.context();
    match records::resolve_id(&ctx, &cfg, id) {
        Ok(Some(resolved)) => Ok((resolved.record, cfg)),
        Ok(None) => Err(HttpError::new(404)),
        Err(_) => Err(HttpError::new(500)),
    }
}

fn resolved_group<'a>(record: &Record, index: &'a records::Index) -> Option<&'a Record> {
    if record.form != Form::Requirement {
        return None;
    }
    let group_id = record.group_id.as_deref()?;
    index.groups.iter().find(|group| group.id == group_id)
}

fn view_page(id: &str, state: &ServerState) -> Result<Response, HttpError> {
    let (record, cfg) = resolve_record(id, state)?;
    let ctx = state.context();
    let index = records::build_index(&ctx, &cfg, record.kind);
    let group = resolved_group(&record, &index);
    let members: Vec<Record> = if record.form == Form::Group {
        index
            .members
            .get(&record.id)
            .map(|positions| {
                positions
                    .iter()
                    .filter_map(|position| index.requirements.get(*position).cloned())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let (project, bind_note) = chrome_for(&cfg, state);
    let chrome = render::Chrome {
        project: &project,
        bind_note: &bind_note,
    };
    Ok(Response::html(
        200,
        render::view_page(&record, group, &members, &cfg, chrome),
    ))
}

fn edit_page(id: &str, state: &ServerState) -> Result<Response, HttpError> {
    let (record, cfg) = resolve_record(id, state)?;
    let ctx = state.context();
    let index = records::build_index(&ctx, &cfg, record.kind);
    let group = resolved_group(&record, &index);
    let values = render::EditValues {
        title: record.title.clone(),
        story: record.story.clone().unwrap_or_default(),
        description: record.description.clone().unwrap_or_default(),
    };
    let (project, bind_note) = chrome_for(&cfg, state);
    let chrome = render::Chrome {
        project: &project,
        bind_note: &bind_note,
    };
    Ok(Response::html(
        200,
        render::edit_page(
            &record,
            group,
            &values,
            &render::FieldErrors::default(),
            &record.version,
            &cfg,
            chrome,
        ),
    ))
}

fn media_type(request: &Request) -> String {
    request
        .header("content-type")
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

fn parse_form(body: &[u8]) -> Result<Vec<(String, String)>, HttpError> {
    let text = std::str::from_utf8(body).map_err(|_| HttpError::new(400))?;
    let mut fields: Vec<(String, String)> = Vec::new();
    for pair in text.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };
        let key = http::decode_component(key)?;
        let value = http::decode_component(value)?;
        fields.push((key, value));
    }
    Ok(fields)
}

fn form_field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .rev()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn form_post(id: &str, request: Request, state: &ServerState) -> Result<Response, HttpError> {
    if media_type(&request) != "application/x-www-form-urlencoded" {
        return Err(HttpError::new(415));
    }
    let fields = parse_form(&request.body)?;
    let version = form_field(&fields, "version").unwrap_or("");
    if !records::valid_version_token(version) {
        return Err(HttpError::new(400));
    }
    let (record, cfg) = resolve_record(id, state)?;
    let submitted = render::EditValues {
        title: form_field(&fields, "title").unwrap_or("").to_string(),
        story: form_field(&fields, "story").unwrap_or("").to_string(),
        description: form_field(&fields, "description").unwrap_or("").to_string(),
    };
    let (project, bind_note) = chrome_for(&cfg, state);
    let chrome = render::Chrome {
        project: &project,
        bind_note: &bind_note,
    };

    if version != record.version {
        return Ok(Response::html(
            409,
            render::conflict_page(&record, &submitted, &cfg, chrome),
        ));
    }

    let mut errors = render::FieldErrors::default();
    let title = match records::validate_title(&submitted.title) {
        Ok(title) => Some(title),
        Err(message) => {
            errors.title = Some(message);
            None
        }
    };

    let mut story_change = FieldChange::Keep;
    if record.kind.supports_story()
        && let Some(raw) = form_field(&fields, "story")
    {
        match records::validate_multiline("User story", raw) {
            Ok(value) => story_change = render::change(value),
            Err(message) => errors.story = Some(message),
        }
    }
    let mut description_change = FieldChange::Keep;
    if let Some(raw) = form_field(&fields, "description") {
        match records::validate_multiline("Description", raw) {
            Ok(value) => description_change = render::change(value),
            Err(message) => errors.description = Some(message),
        }
    }

    if errors.any() {
        let index = records::build_index(&state.context(), &cfg, record.kind);
        return Ok(Response::html(
            422,
            render::edit_page(
                &record,
                resolved_group(&record, &index),
                &submitted,
                &errors,
                &record.version,
                &cfg,
                chrome,
            ),
        ));
    }

    let title = title.unwrap_or_default();
    let content = records::apply_edit(
        &record.doc,
        &record.id,
        &title,
        story_change,
        description_change,
        record.kind.supports_story(),
    );
    records::write_atomic(&state.context(), &record.path, &content)
        .map_err(|_| HttpError::new(500))?;
    Ok(Response::redirect(&format!("/doc/{}", record.id)))
}

fn record_json(record: &Record, group: Option<&Record>) -> serde_json::Value {
    let group_value = match (record.form, record.group_id.as_deref(), group) {
        (Form::Requirement, Some(id), Some(group)) => {
            serde_json::json!({"id": id, "title": group.title})
        }
        _ => serde_json::Value::Null,
    };
    let story = if record.kind.supports_story() {
        serde_json::json!(record.story)
    } else {
        serde_json::Value::Null
    };
    serde_json::json!({
        "id": record.id,
        "kind": record.kind.key(),
        "form": match record.form { Form::Group => "group", Form::Requirement => "requirement" },
        "title": record.title,
        "group": group_value,
        "story": story,
        "description": record.description,
        "version": record.version,
        "path": record.rel_path,
    })
}

fn api_requirements(request: &Request, state: &ServerState) -> Result<Response, HttpError> {
    let kind = category_from_query(request)?;
    let cfg = state.config().map_err(|_| HttpError::new(500))?;
    let index = records::build_index(&state.context(), &cfg, kind);
    let groups: Vec<serde_json::Value> = index
        .groups
        .iter()
        .map(|group| {
            serde_json::json!({
                "id": group.id,
                "kind": kind.key(),
                "form": "group",
                "title": group.title,
            })
        })
        .collect();
    let requirements: Vec<serde_json::Value> = index
        .requirements
        .iter()
        .map(|requirement| {
            serde_json::json!({
                "id": requirement.id,
                "kind": kind.key(),
                "form": "requirement",
                "title": requirement.title,
                "group": requirement.group_id,
            })
        })
        .collect();
    Ok(Response::json(
        200,
        serde_json::json!({"groups": groups, "requirements": requirements}),
    ))
}

fn api_get(id: &str, state: &ServerState) -> Result<Response, HttpError> {
    let (record, cfg) = resolve_record(id, state)?;
    let index = records::build_index(&state.context(), &cfg, record.kind);
    let group = resolved_group(&record, &index);
    Ok(Response::json(200, record_json(&record, group)))
}

fn api_post(id: &str, request: Request, state: &ServerState) -> Result<Response, HttpError> {
    if media_type(&request) != "application/json" {
        return Err(HttpError::new(415));
    }
    let value: serde_json::Value =
        serde_json::from_slice(&request.body).map_err(|_| HttpError::new(400))?;
    let object = value.as_object().ok_or_else(|| HttpError::new(400))?;

    let version = object
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or_else(|| HttpError::new(400))?;
    if !records::valid_version_token(version) {
        return Err(HttpError::new(400));
    }
    let (record, _cfg) = resolve_record(id, state)?;
    if version != record.version {
        return Ok(json_error(409, "record changed on disk; reload the form"));
    }

    let title = object
        .get("title")
        .and_then(|value| value.as_str())
        .ok_or_else(|| HttpError::new(400))?;
    let title = match records::validate_title(title) {
        Ok(title) => title,
        Err(message) => return Ok(json_error(422, &message)),
    };

    let mut story_change = FieldChange::Keep;
    if record.kind.supports_story() {
        match object.get("story") {
            None => {}
            Some(serde_json::Value::Null) => story_change = FieldChange::Clear,
            Some(serde_json::Value::String(raw)) => {
                match records::validate_multiline("User story", raw) {
                    Ok(value) => story_change = render::change(value),
                    Err(message) => return Ok(json_error(422, &message)),
                }
            }
            Some(_) => return Err(HttpError::new(400)),
        }
    }

    let mut description_change = FieldChange::Keep;
    match object.get("description") {
        None => {}
        Some(serde_json::Value::Null) => description_change = FieldChange::Clear,
        Some(serde_json::Value::String(raw)) => {
            match records::validate_multiline("Description", raw) {
                Ok(value) => description_change = render::change(value),
                Err(message) => return Ok(json_error(422, &message)),
            }
        }
        Some(_) => return Err(HttpError::new(400)),
    }

    let content = records::apply_edit(
        &record.doc,
        &record.id,
        &title,
        story_change,
        description_change,
        record.kind.supports_story(),
    );
    records::write_atomic(&state.context(), &record.path, &content)
        .map_err(|_| HttpError::new(500))?;

    let (updated, updated_cfg) = resolve_record(id, state)?;
    let index = records::build_index(&state.context(), &updated_cfg, updated.kind);
    let group = resolved_group(&updated, &index);
    Ok(Response::json(200, record_json(&updated, group)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::default()
    }

    #[test]
    fn defaults_resolve() {
        let options = resolve_options(None, None, false, &EnvValues::default(), &config()).unwrap();
        assert_eq!(
            options,
            BrowserOptions {
                host: "127.0.0.1".to_string(),
                port: 6420,
                open: true
            }
        );
    }

    #[test]
    fn precedence_is_per_setting() {
        let mut cfg = config();
        cfg.browser_host = "localhost".to_string();
        cfg.browser_port = 7000;
        cfg.browser_open = true;
        let env = EnvValues {
            host: Some("0.0.0.0".to_string()),
            port: Some("7001".to_string()),
            open: Some("false".to_string()),
        };
        let options = resolve_options(None, None, false, &env, &cfg).unwrap();
        assert_eq!(options.host, "0.0.0.0");
        assert_eq!(options.port, 7001);
        assert!(!options.open);

        let options = resolve_options(Some("::1"), Some(7002), false, &env, &cfg).unwrap();
        assert_eq!(options.host, "::1");
        assert_eq!(options.port, 7002);

        let options = resolve_options(None, None, true, &env, &cfg).unwrap();
        assert!(!options.open);
    }

    #[test]
    fn invalid_environment_values_are_validation_errors() {
        let env = EnvValues {
            port: Some("abc".to_string()),
            ..EnvValues::default()
        };
        assert!(resolve_options(None, None, false, &env, &config()).is_err());
        let env = EnvValues {
            open: Some("maybe".to_string()),
            ..EnvValues::default()
        };
        assert!(resolve_options(None, None, false, &env, &config()).is_err());
        let env = EnvValues {
            host: Some("bad/host".to_string()),
            ..EnvValues::default()
        };
        assert!(resolve_options(None, None, false, &env, &config()).is_err());
    }

    #[test]
    fn loopback_classification() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.5.5.5"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("LOCALHOST"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("example.test"));
    }

    #[test]
    fn display_hosts() {
        assert_eq!(display_host("0.0.0.0"), "127.0.0.1");
        assert_eq!(display_host("::"), "127.0.0.1");
        assert_eq!(display_host("::1"), "[::1]");
        assert_eq!(display_host("localhost"), "localhost");
    }
}
