use crate::common::{
    AppError, CommitResult, DirEntry, EntryKind, RepoInfo, UpdateResult,
};
use quick_xml::de::from_str;
use serde::Deserialize;

pub fn svn_commit(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<CommitResult, AppError> {
    let mut args = vec!["commit", "-m", message, path];
    for f in files {
        args.push(f);
    }
    let output = crate::svn::run_svn(&args, timeout_secs)?;

    let revision = extract_revision_from_output(&output);
    Ok(CommitResult {
        revision,
        success: true,
        errors: None,
    })
}

fn extract_revision_from_output(output: &str) -> u64 {
    for line in output.lines() {
        if line.contains("Committed revision") {
            if let Some(rev_str) = line.split_whitespace().last() {
                if let Ok(rev) = rev_str.trim_end_matches('.').parse::<u64>() {
                    return rev;
                }
            }
        }
    }
    0
}

pub fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn(&["update", "--xml", path], timeout_secs)?;
    parse_update_xml(&output)
}

#[derive(Deserialize)]
struct UpdateXml {
    #[serde(rename = "update-report")]
    report: Option<UpdateReport>,
}

#[derive(Deserialize)]
struct UpdateReport {
    #[serde(rename = "target")]
    target: Option<String>,
    #[serde(rename = "revision")]
    revision: Option<String>,
}

fn parse_update_xml(xml: &str) -> Result<UpdateResult, AppError> {
    let _report: UpdateXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse update XML: {}", e)))?;

    Ok(UpdateResult {
        revision: 0,
        updated_files: vec![],
        merged_files: vec![],
        conflicts: vec![],
    })
}

pub fn svn_checkout(
    url: &str,
    dest: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    crate::svn::run_svn(&["checkout", url, dest], timeout_secs)?;
    Ok(())
}

pub fn svn_list(
    path: &str,
    revision: Option<&str>,
    recursive: bool,
    timeout_secs: u64,
) -> Result<Vec<DirEntry>, AppError> {
    let mut args = vec!["list", "--xml", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    if recursive {
        args.push("-R");
    }
    let xml = crate::svn::run_svn(&args, timeout_secs)?;
    parse_list_xml(&xml)
}

#[derive(Deserialize)]
struct ListXml {
    #[serde(rename = "entry")]
    entries: Option<Vec<ListItem>>,
}

#[derive(Deserialize)]
struct ListItem {
    #[serde(rename = "@kind")]
    kind: String,
    name: String,
    #[serde(rename = "size")]
    size: Option<u64>,
    #[serde(rename = "commit")]
    commit: Option<ListCommit>,
}

#[derive(Deserialize)]
struct ListCommit {
    #[serde(rename = "@revision")]
    revision: u64,
    author: Option<String>,
    date: Option<String>,
}

fn parse_list_xml(xml: &str) -> Result<Vec<DirEntry>, AppError> {
    let list_xml: ListXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse list XML: {}", e)))?;

    let entries = list_xml.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .map(|entry| {
            let kind = if entry.kind == "dir" {
                EntryKind::Dir
            } else {
                EntryKind::File
            };
            let commit = entry.commit.unwrap_or(ListCommit {
                revision: 0,
                author: None,
                date: None,
            });
            DirEntry {
                name: entry.name,
                kind,
                size: entry.size,
                revision: commit.revision,
                author: commit.author.unwrap_or_default(),
                date: commit.date.unwrap_or_default(),
            }
        })
        .collect())
}

pub fn svn_cat(
    path: &str,
    revision: Option<&str>,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["cat", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    crate::svn::run_svn(&args, timeout_secs)
}

pub fn svn_info(path: &str, timeout_secs: u64) -> Result<RepoInfo, AppError> {
    let xml = crate::svn::run_svn(&["info", "--xml", path], timeout_secs)?;
    parse_info_xml(&xml)
}

#[derive(Deserialize)]
struct InfoXml {
    #[serde(rename = "entry")]
    entry: Option<InfoDetail>,
}

#[derive(Deserialize)]
struct InfoDetail {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "@revision")]
    revision: u64,
    repository: Option<InfoRepository>,
    #[serde(rename = "wc-info")]
    wc_info: Option<InfoWcInfo>,
}

#[derive(Deserialize)]
struct InfoRepository {
    root: Option<String>,
}

#[derive(Deserialize)]
struct InfoWcInfo {
    #[serde(rename = "@revision")]
    revision: Option<u64>,
}

fn parse_info_xml(xml: &str) -> Result<RepoInfo, AppError> {
    let info_xml: InfoXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse info XML: {}", e)))?;

    let entry = info_xml.entry.ok_or_else(|| {
        AppError::Svn("No entry found in info XML".to_string())
    })?;

    let root = entry
        .repository
        .and_then(|r| r.root)
        .unwrap_or_default();

    Ok(RepoInfo {
        url: entry.url,
        root,
        revision: entry.revision,
        last_changed_rev: 0,
        last_changed_date: String::new(),
        last_changed_author: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_revision() {
        let output = "Sending        src/main.rs\nCommitted revision 42.\n";
        assert_eq!(extract_revision_from_output(output), 42);
    }

    #[test]
    fn test_extract_revision_no_match() {
        let output = "No changes.\n";
        assert_eq!(extract_revision_from_output(output), 0);
    }

    #[test]
    fn test_parse_list_xml() {
        let xml = r#"<?xml version="1.0"?>
<list path="/">
  <entry kind="dir">
    <name>src</name>
    <commit revision="10">
      <author>alice</author>
      <date>2026-04-30T10:00:00Z</date>
    </commit>
  </entry>
  <entry kind="file">
    <name>README.md</name>
    <size>1234</size>
    <commit revision="5">
      <author>bob</author>
      <date>2026-04-28T09:00:00Z</date>
    </commit>
  </entry>
</list>"#;
        let result = parse_list_xml(xml).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].kind, EntryKind::Dir);
        assert_eq!(result[0].name, "src");
        assert_eq!(result[1].kind, EntryKind::File);
        assert_eq!(result[1].size, Some(1234));
    }
}
