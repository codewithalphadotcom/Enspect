use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::config::Config;
use crate::utils::fs::is_binary_file;

use super::extractor::extract_env_references;
use super::reference::EnvReference;

pub struct ScanResult {
    pub references: Vec<EnvReference>,
    pub files_scanned: usize,
    pub dynamic_warnings: Vec<(PathBuf, usize)>,
}

/// Walk the directory tree and extract all env var references.
pub fn scan_directory(root: &Path, config: &Config) -> Result<ScanResult> {
    let allowed_extensions: HashSet<&str> = config.scan.extensions.iter().map(|s| s.as_str()).collect();

    // Collect files first
    let files: Vec<PathBuf> = build_walker(root, config)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?;
            if !allowed_extensions.contains(ext) {
                return None;
            }
            // Skip binary files
            if is_binary_file(path).unwrap_or(false) {
                return None;
            }
            Some(path.to_path_buf())
        })
        .collect();

    let files_scanned = files.len();

    // Parallel extraction
    let all_refs: Vec<Vec<EnvReference>> = files
        .par_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(path).ok()?;
            let ext = path.extension()?.to_str()?;
            Some(extract_env_references(&content, path, ext))
        })
        .collect();

    let mut references = Vec::new();
    let mut dynamic_warnings = Vec::new();

    for file_refs in all_refs {
        for r in file_refs {
            if r.key == "<dynamic>" {
                dynamic_warnings.push((r.file.clone(), r.line));
            } else {
                references.push(r);
            }
        }
    }

    Ok(ScanResult {
        references,
        files_scanned,
        dynamic_warnings,
    })
}

fn build_walker(root: &Path, config: &Config) -> Result<ignore::Walk> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .follow_links(config.scan.follow_symlinks);

    // Add custom ignore file
    let enspect_ignore = root.join(".Enspectignore");
    if enspect_ignore.exists() {
        builder.add_ignore(enspect_ignore);
    }

    // Add custom ignore directories via overrides
    let mut overrides = ignore::overrides::OverrideBuilder::new(root);
    for dir in &config.scan.ignore_dirs {
        overrides.add(&format!("!{dir}/"))?;
    }
    for file in &config.scan.ignore_files {
        overrides.add(&format!("!{file}"))?;
    }
    builder.overrides(overrides.build()?);

    Ok(builder.build())
}
