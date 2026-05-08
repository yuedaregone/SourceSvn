use crate::common::AppError;
use crate::svn::models::{ChangedPath, LogEntry, PathAction, WcLogResult};
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct LogXml {
    #[serde(rename = "logentry")]
    entries: Option<Vec<LogEntryXml>>,
}

#[derive(Deserialize)]
struct LogEntryXml {
    #[serde(rename = "@revision")]
    revision: u64,
    author: String,
    date: String,
    msg: String,
    #[serde(rename = "paths")]
    paths: Option<PathsXml>,
}

#[derive(Deserialize)]
struct PathsXml {
    #[serde(rename = "path")]
    entries: Option<Vec<PathEntryXml>>,
}

#[derive(Deserialize)]
struct PathEntryXml {
    #[serde(rename = "@action")]
    action: String,
    #[serde(rename = "@copyfrom-path")]
    copy_from_path: Option<String>,
    #[serde(rename = "@copyfrom-rev")]
    copy_from_rev: Option<u64>,
    #[serde(rename = "$text")]
    text: String,
}

pub fn parse_log_xml(xml: &str) -> Result<Vec<LogEntry>, AppError> {
    let log: LogXml = from_str(xml)
        .map_err(|e| AppError::svn_parse(format!("Failed to parse log XML: {}", e)))?;

    let entries = log.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .map(|entry| {
            let changed_paths = entry.paths.and_then(|p| p.entries).map(|paths| {
                paths
                    .into_iter()
                    .map(|p| {
                        let action = match p.action.as_str() {
                            "A" => PathAction::A,
                            "M" => PathAction::M,
                            "D" => PathAction::D,
                            "R" => PathAction::R,
                            _ => PathAction::M,
                        };
                        ChangedPath {
                            path: p.text,
                            action,
                            copy_from_path: p.copy_from_path,
                            copy_from_rev: p.copy_from_rev,
                        }
                    })
                    .collect()
            });

            LogEntry {
                revision: entry.revision,
                author: entry.author,
                date: entry.date,
                message: entry.msg,
                changed_paths,
            }
        })
        .collect())
}

pub async fn svn_log(
    path: &str,
    limit: Option<u32>,
    from_rev: Option<&str>,
    timeout_secs: u64,
) -> Result<Vec<LogEntry>, AppError> {
    let mut args = vec!["log", "--xml", "-v", path];
    let limit_str;
    if let Some(l) = limit {
        args.push("-l");
        limit_str = l.to_string();
        args.push(&limit_str);
    }
    if let Some(rev) = from_rev {
        args.push("-r");
        args.push(rev);
    }
    let xml = crate::svn::run_svn_async(&args, timeout_secs).await?;
    parse_log_xml(&xml)
}

pub async fn svn_log_server(
    path: &str,
    limit: Option<u32>,
    timeout_secs: u64,
) -> Result<WcLogResult, AppError> {
    // Step 1: svn info
    let info_xml = crate::svn::run_svn_async(&["info", "--xml", path], timeout_secs).await?;
    let info = crate::svn::info::parse_info_for_log(&info_xml)?;
    let repo_url = info.url;
    let wc_rev = info.wc_revision;
    
    // Step 2: svn log via URL
    let mut args = vec!["log", "--xml", "-v", &repo_url];
    let limit_str;
    if let Some(l) = limit {
        args.push("-l");
        limit_str = l.to_string();
        args.push(&limit_str);
    }
    let xml = crate::svn::run_svn_async(&args, timeout_secs).await?;
    let entries = parse_log_xml(&xml)?;

    // Step 3: classify local vs non-local
    let entries: Vec<LogEntry> = entries
        .into_iter()
        .map(|e| {
            if e.revision <= wc_rev {
                e
            } else {
                LogEntry {
                    revision: e.revision,
                    author: e.author,
                    date: e.date,
                    message: e.message,
                    changed_paths: None,
                }
            }
        })
        .collect();

    Ok(WcLogResult {
        entries,
        wc_revision: wc_rev,
        root: info.root,
    })
}

pub async fn svn_log_changed_paths(
    path: &str,
    revision: u64,
    timeout_secs: u64,
) -> Result<Vec<ChangedPath>, AppError> {
    let rev_str = revision.to_string();
    let args = vec!["log", "--xml", "-v", "-r", &rev_str, path];
    let xml = crate::svn::run_svn_async(&args, timeout_secs).await?;
    let entries = parse_log_xml(&xml)?;
    Ok(entries
        .into_iter()
        .next()
        .and_then(|e| e.changed_paths)
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_xml_single() {
        let xml = r#"<?xml version="1.0"?>
<log>
  <logentry revision="100">
    <author>alice</author>
    <date>2026-04-30T10:30:00Z</date>
    <msg>Fix login bug</msg>
    <paths>
      <path action="M">src/main.rs</path>
      <path action="A">tests/test.rs</path>
    </paths>
  </logentry>
</log>"#;
        let result = parse_log_xml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].revision, 100);
        assert_eq!(result[0].author, "alice");
        assert_eq!(result[0].message, "Fix login bug");
        let paths = result[0].changed_paths.as_ref().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].action, PathAction::M);
        assert_eq!(paths[1].action, PathAction::A);
    }

    #[test]
    fn test_parse_log_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
<log/>"#;
        let result = parse_log_xml(xml).unwrap();
        assert_eq!(result.len(), 0);
    }
}
