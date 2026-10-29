//! Mechanical checks for the rules in STACK.md and AGENTS.md. A failure names the file and the rule.

#![expect(
    clippy::expect_used,
    reason = "test helpers may panic with a clear message"
)]

use std::fs;
use std::path::{Path, PathBuf};

const HARD_FILE_LINES: usize = 500;
const SOFT_FILE_LINES: usize = 300;
const PURE_CRATES: [&str; 3] = ["voxel-core", "voxel-gen", "voxel-mesh"];
const DUMPING_GROUND_NAMES: [&str; 6] = ["utils", "util", "helpers", "helper", "common", "misc"];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        {
            out.push(path);
        }
    }
}

/// Source files of every crate except this one: its tests quote the forbidden patterns.
fn source_files(crate_filter: impl Fn(&str) -> bool) -> Vec<(String, PathBuf)> {
    let crates_dir = workspace_root().join("crates");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates_dir).expect("crates dir").flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "guard" || !crate_filter(&name) {
            continue;
        }
        let mut found = Vec::new();
        collect_rust_files(&entry.path().join("src"), &mut found);
        files.extend(found.into_iter().map(|path| (name.clone(), path)));
    }
    files
}

fn violations_matching(
    files: &[(String, PathBuf)],
    forbidden: &[&str],
    skip_comments: bool,
) -> Vec<String> {
    let mut found = Vec::new();
    for (crate_name, path) in files {
        let text = fs::read_to_string(path).expect("readable source");
        for (index, line) in text.lines().enumerate() {
            if skip_comments && line.trim_start().starts_with("//") {
                continue;
            }
            if let Some(pattern) = forbidden.iter().find(|pattern| line.contains(**pattern)) {
                found.push(format!(
                    "{crate_name}: {}:{} uses `{pattern}`",
                    path.display(),
                    index + 1
                ));
            }
        }
    }
    found
}

fn assert_none(rule: &str, violations: &[String]) {
    assert!(
        violations.is_empty(),
        "{rule}\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn no_file_exceeds_the_hard_line_limit() {
    let mut too_long = Vec::new();
    for (crate_name, path) in source_files(|_| true) {
        let lines = fs::read_to_string(&path)
            .expect("readable source")
            .lines()
            .count();
        if lines > SOFT_FILE_LINES {
            eprintln!(
                "note: {crate_name}: {} has {lines} lines (soft limit {SOFT_FILE_LINES})",
                path.display()
            );
        }
        if lines > HARD_FILE_LINES {
            too_long.push(format!("{}: {lines} lines", path.display()));
        }
    }
    assert_none(
        &format!("One file, one responsibility: split files over {HARD_FILE_LINES} lines"),
        &too_long,
    );
}

#[test]
fn no_dumping_ground_modules() {
    let mut offenders = Vec::new();
    for (_, path) in source_files(|_| true) {
        let in_dumping_ground = path.components().any(|component| {
            let name = component.as_os_str().to_string_lossy();
            let stem = name.strip_suffix(".rs").unwrap_or(&name);
            DUMPING_GROUND_NAMES.contains(&stem)
        });
        if in_dumping_ground {
            offenders.push(path.display().to_string());
        }
    }
    assert_none(
        "No utils/helpers/common/misc: name modules for what they do",
        &offenders,
    );
}

#[test]
fn pure_crates_do_not_know_the_engine_or_the_outside_world() {
    let files = source_files(|name| PURE_CRATES.contains(&name));
    let forbidden = [
        "bevy",
        "std::fs",
        "std::net",
        "std::env",
        "std::time::SystemTime",
        "Instant::now",
        "thread_rng",
        "rand::random",
    ];
    assert_none(
        "voxel-* crates are pure and deterministic: no engine, IO, clock or ambient randomness (ADR 0002)",
        &violations_matching(&files, &forbidden, true),
    );
}

#[test]
fn nothing_touches_the_network() {
    let forbidden = [
        "std::net",
        "reqwest",
        "hyper::",
        "ureq",
        "tungstenite",
        "tokio::net",
    ];
    assert_none(
        "The game is fully offline: no network code anywhere",
        &violations_matching(&source_files(|_| true), &forbidden, true),
    );
}

#[test]
fn gameplay_never_branches_on_the_platform() {
    let files = source_files(|name| name == "game");
    let forbidden = ["target_os", "target_arch", "target_family"];
    assert_none(
        "Branch on capabilities and tier, not on the platform; per-OS glue lives in `platform`",
        &violations_matching(&files, &forbidden, true),
    );
}

#[test]
fn every_adr_is_indexed_and_has_the_required_sections() {
    let adr_dir = workspace_root().join("docs/adr");
    let index = fs::read_to_string(adr_dir.join("README.md")).expect("adr index");
    let mut problems = Vec::new();
    for entry in fs::read_dir(&adr_dir).expect("adr dir").flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let is_numbered = file_name.len() > 4 && file_name[..4].chars().all(|c| c.is_ascii_digit());
        if !is_numbered
            || file_name.starts_with("0000")
            || !Path::new(&file_name)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        if !index.contains(&format!("| {} ", &file_name[..4])) {
            problems.push(format!("{file_name}: missing from docs/adr/README.md"));
        }
        let text = fs::read_to_string(entry.path()).expect("readable adr");
        for section in [
            "- Status:",
            "- Date:",
            "## Context",
            "## Decision",
            "## Consequences",
        ] {
            if !text.contains(section) {
                problems.push(format!("{file_name}: missing `{section}`"));
            }
        }
    }
    assert_none(
        "ADRs follow docs/adr/0000-template.md and are listed in the index",
        &problems,
    );
}

#[test]
fn player_code_never_depends_on_developer_tools() {
    let allowed = ["lib.rs", "main.rs", "run_mode.rs", "dev.rs"];
    let files: Vec<_> = source_files(|name| name == "game")
        .into_iter()
        .filter(|(_, path)| {
            let in_dev_folder = path.components().any(|part| part.as_os_str() == "dev");
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            !in_dev_folder && !file_name.is_some_and(|name| allowed.contains(&name.as_str()))
        })
        .collect();
    let forbidden = ["dev::", "feature = \"dev\"", "DevSettings", "DevPlugin"];
    assert_none(
        "Developer tools are compiled out of player builds: only lib.rs, main.rs, run_mode.rs and game/src/dev may mention them",
        &violations_matching(&files, &forbidden, true),
    );
}
