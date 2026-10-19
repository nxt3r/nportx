use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExpectedPort {
    pub port: u16,
    pub source: String,
}

/// Detect expected ports for the project in `dir`.
/// Returns ports in priority order: explicit .env, explicit package.json scripts, framework default.
pub fn detect_expected_ports(dir: &Path) -> Result<Vec<ExpectedPort>> {
    let mut results = Vec::new();

    // 1. .env family — explicit
    for env_file in [".env", ".env.local", ".env.development"] {
        let path = dir.join(env_file);
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                extract_from_env(&contents, env_file, &mut results);
            }
        }
    }

    // 2. package.json — explicit scripts
    let pkg_path = dir.join("package.json");
    let mut pkg_json: Option<Value> = None;
    if pkg_path.exists() {
        if let Ok(contents) = fs::read_to_string(&pkg_path) {
            if let Ok(v) = serde_json::from_str::<Value>(&contents) {
                extract_from_package_json(&v, &mut results);
                pkg_json = Some(v);
            }
        }
    }

    // 3. Framework default — only if nothing explicit so far
    if results.is_empty() {
        if let Some(v) = &pkg_json {
            if let Some((port, framework)) = framework_default_port(v) {
                results.push(ExpectedPort {
                    port,
                    source: format!("{} default", framework),
                });
            }
        }
    }

    // Deduplicate by port (keep first occurrence — highest priority wins)
    let mut seen = std::collections::HashSet::new();
    results.retain(|e| seen.insert(e.port));

    Ok(results)
}

fn extract_from_env(contents: &str, filename: &str, out: &mut Vec<ExpectedPort>) {
    // Match lines like: PORT=3000, VITE_PORT=5173, export API_PORT="8000"
    let re = Regex::new(r#"(?m)^\s*(?:export\s+)?([A-Z0-9_]*PORT[A-Z0-9_]*)\s*=\s*["']?(\d+)["']?\s*$"#).unwrap();
    for cap in re.captures_iter(contents) {
        let var = &cap[1];
        if let Ok(port) = cap[2].parse::<u16>() {
            out.push(ExpectedPort {
                port,
                source: format!("{} ({})", filename, var),
            });
        }
    }
}

fn extract_from_package_json(v: &Value, out: &mut Vec<ExpectedPort>) {
    let scripts = match v.get("scripts").and_then(|s| s.as_object()) {
        Some(s) => s,
        None => return,
    };

    // Patterns: -p 3001, --port 3001, --port=3001, -p3001
    let re = Regex::new(r#"(?:-p|--port)[\s=]+?(\d+)"#).unwrap();

    for key in ["start", "dev"] {
        if let Some(script) = scripts.get(key).and_then(|s| s.as_str()) {
            if let Some(cap) = re.captures(script) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    out.push(ExpectedPort {
                        port,
                        source: format!("package.json (scripts.{})", key),
                    });
                }
            }
        }
    }
}

fn framework_default_port(v: &Value) -> Option<(u16, &'static str)> {
    let deps = v.get("dependencies").and_then(|d| d.as_object());
    let dev_deps = v.get("devDependencies").and_then(|d| d.as_object());

    let has = |name: &str| -> bool {
        deps.map(|d| d.contains_key(name)).unwrap_or(false)
            || dev_deps.map(|d| d.contains_key(name)).unwrap_or(false)
    };

    if has("vite") {
        return Some((5173, "Vite"));
    }
    if has("next") {
        return Some((3000, "Next.js"));
    }
    if has("react-scripts") {
        return Some((3000, "React (CRA)"));
    }
    if has("nuxt") {
        return Some((3000, "Nuxt"));
    }
    if has("gatsby") {
        return Some((8000, "Gatsby"));
    }
    None
}