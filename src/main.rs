use papyrus::{CommandOutput, parse, run};
use std::process::exit;

fn main() {
    let cli = parse();
    let json = cli.json;
    match run(cli) {
        Ok(out) => {
            print_output(&out, json);
            exit(0);
        }
        Err(err) => {
            if json {
                let v = serde_json::json!({
                    "error": {
                        "code": err.code(),
                        "message": err.to_string(),
                    }
                });
                eprintln!("{}", v);
            } else {
                eprintln!("{}", err);
            }
            exit(err.code());
        }
    }
}

fn print_output(out: &CommandOutput, json: bool) {
    if json {
        if let Some(v) = &out.json {
            println!("{}", v);
        } else if !out.text.is_empty() {
            // wrap plain text as JSON string? Print as-is for safety.
            println!("{}", out.text);
        }
    } else if !out.text.is_empty() {
        println!("{}", out.text);
    }
}
