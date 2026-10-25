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
/// Returns ports in priority order: explicit .env, explicit package.json scripts, docker-compose, framework default.
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

    // 3. docker-compose.yml — explicit ports
    for compose_file in ["docker-compose.yml", "docker-compose.yaml"] {
        let path = dir.join(compose_file);
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                extract_from_docker_compose(&contents, compose_file, &mut results);
            }
        }
    }

    // 4. Framework default — only if nothing explicit so far
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

fn extract_from_docker_compose(contents: &str, filename: &str, out: &mut Vec<ExpectedPort>) {
    // Parse the YAML into a serde_yaml::Value
    let yaml: serde_yaml::Value = match serde_yaml::from_str(contents) {
        Ok(v) => v,
        Err(_) => return, // If the YAML is invalid, quietly skip it
    };

    // Navigate to services
    let services = match yaml.get("services").and_then(|s| s.as_mapping()) {
        Some(s) => s,
        None => return,
    };

    for (service_name, service_val) in services {
        let name = service_name.as_str().unwrap_or("unknown");
        
        // Look for the "ports" array
        if let Some(ports) = service_val.get("ports").and_then(|p| p.as_sequence()) {
            for port_entry in ports {
                if let Some(port_str) = port_entry.as_str() {
                    // Format can be "8080:80" or "8080:80/tcp" or just "80"
                    // We want the host port (the first part)
                    if let Some(host_port_str) = port_str.split(':').next() {
                        if let Ok(port) = host_port_str.parse::<u16>() {
                            out.push(ExpectedPort {
                                port,
                                source: format!("{} (service: {})", filename, name),
                            });
                        }
                    }
                } else if let Some(port_num) = port_entry.as_u64() {
                    // Sometimes YAML parses unquoted numbers directly (e.g. ports: - 80)
                    if port_num <= 65535 {
                        out.push(ExpectedPort {
                            port: port_num as u16,
                            source: format!("{} (service: {})", filename, name),
                        });
                    }
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