use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use rustyline::{
    Helper,
    completion::Completer as RustylineCompleter,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    completion::Pair,
    Context,
    Result,
};
use crate::builtins::BUILTINS;

pub struct Completer {
    commands: Vec<String>,
}

impl Completer {
    pub fn new() -> Self {
        Self {
            commands: Self::find_commands(),
        }
    }

    fn find_commands() -> Vec<String> {
        let mut commands = Vec::new();
        
        // Add builtin commands
        commands.extend(BUILTINS.iter().map(|&cmd| cmd.to_string()));

        // Add PATH commands
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.filter_map(|r: std::io::Result<_>| r.ok()) {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        commands.sort();
        commands.dedup();
        commands
    }

    pub fn complete(&self, line: &str) -> Vec<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.len() {
            0 => self.commands.clone(),
            1 => self.complete_command(words[0]),
            _ => self.complete_argument(words.last().unwrap()),
        }
    }

    fn complete_command(&self, prefix: &str) -> Vec<String> {
        self.commands.iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .cloned()
            .collect()
    }

    fn complete_argument(&self, prefix: &str) -> Vec<String> {
        let path = Path::new(prefix);
        let (dir, prefix) = if let Some(parent) = path.parent() {
            (parent.to_path_buf(), path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
        } else {
            (PathBuf::from("."), prefix)
        };

        let mut completions = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|r| r.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(prefix) {
                        let mut full_path = dir.clone().join(name);
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            full_path.push("");  // Add trailing slash for directories
                        }
                        completions.push(full_path.to_string_lossy().into_owned());
                    }
                }
            }
        }
        completions.sort();
        completions
    }
}

// Implement the required traits for rustyline
impl Helper for Completer {}

impl RustylineCompleter for Completer {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        // Find the word start position
        let start = line[..pos].rfind(char::is_whitespace).map_or(0, |i| i + 1);
        
        // Get completions
        let completions = self.complete(&line[start..pos]);
        
        // Convert to Pairs
        let pairs: Vec<Pair> = completions
            .into_iter()
            .map(|s| Pair {
                display: s.clone(),
                replacement: s,
            })
            .collect();

        Ok((start, pairs))
    }
}

// Implement stubs for required traits
impl Highlighter for Completer {}
impl Hinter for Completer {
    type Hint = String;
}
impl Validator for Completer {}
