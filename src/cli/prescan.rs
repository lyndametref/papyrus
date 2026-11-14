use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Prescan {
    pub dir: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub first_positional: Option<String>,
    pub help: bool,
    pub version: bool,
}

pub fn scan(args: &[String]) -> Prescan {
    let mut out = Prescan::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--" {
            break;
        }
        if a == "--dir" {
            if i + 1 < args.len() {
                out.dir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
                continue;
            }
        } else if let Some(v) = a.strip_prefix("--dir=") {
            out.dir = Some(PathBuf::from(v));
            i += 1;
            continue;
        } else if a == "--config" {
            if i + 1 < args.len() {
                out.config = Some(PathBuf::from(&args[i + 1]));
                i += 2;
                continue;
            }
        } else if let Some(v) = a.strip_prefix("--config=") {
            out.config = Some(PathBuf::from(v));
            i += 1;
            continue;
        } else {
            if a == "--help" || a == "-h" {
                out.help = true;
            } else if a == "--version" || a == "-V" {
                out.version = true;
            }
            if !a.starts_with('-') && out.first_positional.is_none() {
                out.first_positional = Some(a.clone());
            }
        }
        i += 1;
    }
    out
}
