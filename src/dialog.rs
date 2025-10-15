use std::fs;

use crate::config::Config;

const DEFAULT_UTTERANCE: &str = "hello liminal";

pub fn default_utterance() -> &'static str {
    DEFAULT_UTTERANCE
}

pub fn load_inputs(cfg: &Config) -> Vec<String> {
    if let Some(path) = cfg.inputs_path.as_deref() {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let lines: Vec<String> = contents
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();
                if !lines.is_empty() {
                    return lines;
                }
            }
            Err(err) => {
                eprintln!("[dialog] failed to read inputs file '{}': {}", path, err);
            }
        }
    }

    if let Some(script) = cfg.script.as_ref() {
        let parts: Vec<String> = script
            .split(';')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect();
        if !parts.is_empty() {
            return parts;
        }
    }

    let cycles = cfg.cycles.max(1);
    vec![DEFAULT_UTTERANCE.to_string(); cycles]
}
