use crate::common::AppError;
use crate::svn::models::BlameEntry;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::path::Path;
use tokio::io::AsyncReadExt;

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

pub async fn svn_resolve(
    path: &str,
    paths: &[String],
    accept: &str,
    timeout_secs: u64,
) -> Result<Vec<String>, AppError> {
    let accept_arg = match accept {
        "theirs" => "theirs-conflict",
        "mine" => "mine-conflict",
        "working" => "working",
        _ => return Err(AppError::Svn(format!("Invalid accept strategy: {}", accept))),
    };
    let mut args = vec!["resolve", "--accept", accept_arg, "-R"];
    for p in paths {
        args.push(p);
    }
    crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await?;
    Ok(paths.to_vec())
}

/// 获取文件的工作副本大小和 SVN base 版本大小
pub async fn file_size_diff(
    repo_path: &str,
    file_path: &str,
    timeout_secs: u64,
) -> Result<(u64, u64), AppError> {
    // 当前文件大小
    let current_size = tokio::fs::metadata(file_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    // base 版本大小：svn cat 输出字节数
    let child = tokio::process::Command::new(crate::svn::find_svn_executable()?)
        .args(["cat", file_path])
        .current_dir(repo_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Svn(format!("Failed to spawn svn cat: {}", e)))?;

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        async {
            let mut stdout = child.stdout.ok_or_else(|| AppError::Svn("No stdout".into()))?;
            let mut buf = Vec::new();
            stdout.read_to_end(&mut buf).await.map_err(|e| AppError::Svn(e.to_string()))?;
            Ok::<u64, AppError>(buf.len() as u64)
        },
    )
    .await
    .map_err(|_| AppError::Svn("svn cat timed out".into()))??;

    Ok((output, current_size))
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
