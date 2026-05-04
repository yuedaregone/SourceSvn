use crate::common::AppError;
use crate::svn::models::BlameEntry;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::path::Path;

/// Walk up from `path` to find the nearest directory containing `.svn`.
pub fn find_svn_root(path: &str) -> Result<String, AppError> {
    let mut dir = Path::new(path);
    if dir.join(".svn").exists() {
        return Ok(dir.to_string_lossy().into_owned());
    }
    while let Some(parent) = dir.parent() {
        if parent.join(".svn").exists() {
            return Ok(parent.to_string_lossy().into_owned());
        }
        dir = parent;
    }
    Err(AppError::Svn(format!(
        "No .svn directory found from {}",
        path
    )))
}

pub async fn svn_revert(
    path: &str,
    paths: &[String],
    timeout_secs: u64,
) -> Result<Vec<String>, AppError> {
    let mut args = vec!["revert"];
    for p in paths {
        args.push(p);
    }
    let output = crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await?;
    Ok(parse_reverted_paths(&output))
}

pub async fn svn_add(
    path: &str,
    paths: &[String],
    timeout_secs: u64,
) -> Result<Vec<String>, AppError> {
    let mut args = vec!["add"];
    for p in paths {
        args.push(p);
    }
    crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await?;
    Ok(paths.to_vec())
}

pub async fn svn_delete(
    path: &str,
    paths: &[String],
    keep_local: bool,
    timeout_secs: u64,
) -> Result<Vec<String>, AppError> {
    let mut args = vec!["delete"];
    if keep_local {
        args.push("--keep-local");
    }
    for p in paths {
        args.push(p);
    }
    crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await?;
    Ok(paths.to_vec())
}

pub async fn svn_blame(
    path: &str,
    revision: Option<i32>,
    timeout_secs: u64,
) -> Result<Vec<BlameEntry>, AppError> {
    let mut args = vec!["blame", "--xml"];
    let rev_str = revision.map(|r| format!("{}", r));
    if let Some(ref s) = rev_str {
        args.push("-r");
        args.push(s);
    }
    args.push(path);
    let xml = crate::svn::run_svn_async_in_dir(&args, timeout_secs, None).await?;
    parse_blame_xml(&xml)
}

pub async fn svn_update_to_revision(
    path: &str,
    revision: i32,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let rev_str = format!("-r{}", revision);
    let args = vec!["update", &rev_str];
    crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await
}

fn parse_reverted_paths(output: &str) -> Vec<String> {
    let quote = "'";
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // svn revert outputs: Reverted 'path/to/file'
            if let Some(path) = line.strip_prefix("Reverted '").and_then(|s| s.strip_suffix(quote)) {
                return Some(path.to_string());
            }
            // svn delete outputs: D         path/to/file
            if let Some(path) = line.strip_prefix("D         ") {
                return Some(path.trim().to_string());
            }
            None
        })
        .collect()
}

#[derive(Deserialize)]
struct BlameXml {
    #[serde(rename = "target")]
    target: Option<BlameTarget>,
}

#[derive(Deserialize)]
struct BlameTarget {
    #[serde(rename = "entry")]
    entries: Option<Vec<BlameEntryRaw>>,
}

#[derive(Deserialize)]
struct BlameEntryRaw {
    #[serde(rename = "@line")]
    _line: String,
    #[serde(rename = "@revision")]
    revision: String,
    author: Option<String>,
}

fn parse_blame_xml(xml: &str) -> Result<Vec<BlameEntry>, AppError> {
    let blame: BlameXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse blame XML: {}", e)))?;

    let target = blame.target.ok_or_else(|| {
        AppError::Svn("No target found in blame XML".to_string())
    })?;

    let entries = target.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let revision = entry.revision.parse::<i32>().unwrap_or(0);
            let author = entry.author.unwrap_or_default();
            BlameEntry {
                revision,
                author,
                line_number: (idx as u32) + 1,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reverted_paths() {
        let output = "Reverted 'src/main.rs'\nReverted 'src/lib.rs'\n";
        let paths = parse_reverted_paths(output);
        assert_eq!(paths, vec!["src/main.rs", "src/lib.rs"]);
    }

    #[test]
    fn test_parse_blame_xml() {
        let xml = r#"<?xml version="1.0"?>
<blame>
  <target path="file.txt">
    <entry line="1" revision="10">
      <author>alice</author>
    </entry>
    <entry line="2" revision="10">
      <author>alice</author>
    </entry>
  </target>
</blame>"#;
        let entries = parse_blame_xml(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].revision, 10);
        assert_eq!(entries[0].author, "alice");
        assert_eq!(entries[0].line_number, 1);
    }
}
