//! registry and starter scaffold for dither-kit-rune
//!
//! `dither list` names the components, `dither add <item>` writes a starter
//! module into your crate, `dither prompt` prints the agent setup prompt and
//! `dither build` emits the registry JSON the site serves.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Item {
    name: String,
    title: String,
    description: String,
    scene: String,
    snippet: String,
    docs: String,
}

#[derive(Deserialize)]
struct Registry {
    name: String,
    version: String,
    license: String,
    source: serde_json::Value,
    items: Vec<Item>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dither: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    let registry_path = registry_path(args)?;
    match command {
        "list" => {
            let registry = load(&registry_path)?;
            for item in &registry.items {
                println!("{:<12} {}", item.name, item.description);
            }
            Ok(())
        }
        "add" => {
            let name = args.get(1).ok_or("usage: dither add <item> [--dir PATH]")?;
            let dir = flag(args, "--dir")
                .map(PathBuf::from)
                .unwrap_or(std::env::current_dir().map_err(io_error)?);
            add(&registry_path, name, &dir)
        }
        "prompt" => {
            print!("{}", prompt(&registry_path)?);
            Ok(())
        }
        "build" => {
            let out = flag(args, "--out")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("www/r"));
            build(&registry_path, &out)
                .map(|count| println!("wrote {count} items to {}", out.display()))
        }
        "help" | "--help" | "-h" => {
            println!(
                "dither <command>\n\n  list                 components in the registry\n  add <item> [--dir]   write a starter module and print the dependency\n  prompt               the agent setup prompt\n  build [--out DIR]    emit registry JSON for the site"
            );
            Ok(())
        }
        other => Err(format!("unknown command {other:?}; try `dither help`")),
    }
}

fn registry_path(args: &[String]) -> Result<PathBuf, String> {
    if let Some(path) = flag(args, "--registry") {
        return Ok(PathBuf::from(path));
    }
    let local = PathBuf::from("registry.json");
    if local.exists() {
        return Ok(local);
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fallback = manifest.join("../../registry.json");
    if fallback.exists() {
        Ok(fallback)
    } else {
        Err("registry.json not found; pass --registry PATH".into())
    }
}

fn load(path: &Path) -> Result<Registry, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn prompt(registry_path: &Path) -> Result<String, String> {
    let path = registry_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("registry/prompt.md");
    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

fn add(registry_path: &Path, name: &str, dir: &Path) -> Result<(), String> {
    let registry = load(registry_path)?;
    let item = registry
        .items
        .iter()
        .find(|item| item.name == name)
        .ok_or_else(|| format!("no item {name:?}; run `dither list`"))?;
    let root = registry_path.parent().unwrap_or(Path::new("."));
    let snippet = fs::read_to_string(root.join(&item.snippet))
        .map_err(|e| format!("{}: {e}", item.snippet))?;

    let target = dir.join("dither");
    fs::create_dir_all(&target).map_err(io_error)?;
    let ident = ident(&item.name);
    fs::write(target.join(format!("{ident}.rs")), snippet).map_err(io_error)?;
    add_module(&target, &ident).map_err(io_error)?;

    let source = registry.source;
    let path = source
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("crates/dither-kit-rune");
    let engine = source
        .get("engine")
        .and_then(|v| v.as_str())
        .unwrap_or("crates/dither-core");
    println!("added dither/{ident}.rs");
    println!("\nadd to Cargo.toml:");
    println!("  dither-kit-rune = {{ path = \"{path}\" }}");
    println!("  dither-core = {{ path = \"{engine}\" }}");
    println!("\nRune is pinned in the workspace; copy its [patch.crates-io] block too.");
    println!("then render with `chart.view(&theme)` and forward pointer input from the view.");
    Ok(())
}

fn add_module(dir: &Path, ident: &str) -> io::Result<()> {
    let path = dir.join("mod.rs");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    let line = format!("pub mod {ident};");
    if !text.lines().any(|existing| existing.trim() == line) {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
        fs::write(path, text)?;
    }
    Ok(())
}

fn build(registry_path: &Path, out: &Path) -> Result<usize, String> {
    let registry = load(registry_path)?;
    let root = registry_path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(out).map_err(io_error)?;

    let index = serde_json::json!({
        "name": registry.name,
        "version": registry.version,
        "license": registry.license,
        "source": registry.source,
        "items": registry.items,
    });
    write_json(&out.join("index.json"), &index)?;

    for item in &registry.items {
        let snippet = fs::read_to_string(root.join(&item.snippet))
            .map_err(|e| format!("{}: {e}", item.snippet))?;
        let entry = serde_json::json!({
            "name": item.name,
            "title": item.title,
            "description": item.description,
            "scene": item.scene,
            "docs": item.docs,
            "source": registry.source,
            "install": {
                "dependency": format!("dither-kit-rune = {{ path = \"{}\" }}", registry.source.get("path").and_then(|v| v.as_str()).unwrap_or("crates/dither-kit-rune")),
                "cli": format!("dither add {}", item.name),
            },
            "snippet": snippet,
        });
        write_json(&out.join(format!("{}.json", item.name)), &entry)?;
    }

    fs::write(out.join("prompt.txt"), prompt(registry_path)?).map_err(io_error)?;
    Ok(registry.items.len())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, text + "\n").map_err(io_error)
}

fn ident(name: &str) -> String {
    name.replace('-', "_")
}

fn flag(args: &[String], name: &str) -> Option<String> {
    let index = args.iter().position(|arg| arg == name)?;
    args.get(index + 1).cloned()
}

fn io_error(error: io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_names_become_module_identifiers() {
        assert_eq!(ident("area-chart"), "area_chart");
        assert_eq!(ident("radar"), "radar");
    }

    #[test]
    fn adding_a_module_twice_keeps_one_line() {
        let dir = std::env::temp_dir().join(format!("dither-cli-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        add_module(&dir, "area_chart").unwrap();
        add_module(&dir, "area_chart").unwrap();
        let text = fs::read_to_string(dir.join("mod.rs")).unwrap();
        assert_eq!(text.matches("pub mod area_chart;").count(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }
}
