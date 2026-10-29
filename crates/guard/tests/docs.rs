//! Keeps the documentation indexed and its references alive. A failure names the file and the rule.

use std::fs;
use std::path::{Path, PathBuf};

const INDEXED_FOLDERS: [&str; 2] = ["docs/stack", "docs/principles"];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn is_markdown(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir()
            && !matches!(
                name.as_str(),
                "target" | ".git" | ".serena" | "node_modules"
            )
        {
            markdown_files(&path, out);
        } else if is_markdown(&name) {
            out.push(path);
        }
    }
}

/// Backticked tokens that name a markdown file, e.g. `docs/stack/layout.md`.
fn referenced_markdown_paths(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .filter(|token| {
            is_markdown(token)
                && !token.contains(' ')
                && !token.contains('*')
                && !token.contains("NNNN")
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_topic_file_is_listed_in_its_folder_index() {
    let mut problems = Vec::new();
    for folder in INDEXED_FOLDERS {
        let dir = workspace_root().join(folder);
        let index = fs::read_to_string(dir.join("README.md")).expect("folder index");
        for entry in fs::read_dir(&dir).expect("docs folder").flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != "README.md" && is_markdown(&name) && !index.contains(&format!("`{name}`")) {
                problems.push(format!("{folder}/{name}: missing from {folder}/README.md"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "Every topic file is indexed\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn referenced_markdown_files_exist() {
    let root = workspace_root();
    let mut files = Vec::new();
    markdown_files(&root, &mut files);
    let mut problems = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("readable markdown");
        let directory = file.parent().expect("file has a parent");
        for reference in referenced_markdown_paths(&text) {
            let exists = directory.join(&reference).exists() || root.join(&reference).exists();
            if !exists {
                problems.push(format!("{}: `{reference}` does not exist", file.display()));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "Documentation references must resolve\n  {}",
        problems.join("\n  ")
    );
}
