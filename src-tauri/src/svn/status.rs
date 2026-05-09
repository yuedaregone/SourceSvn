use crate::common::AppError;
use crate::svn::models::{FileStatus, FileStatusType};
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct StatusXml {
    #[serde(rename = "target")]
    targets: Option<Vec<StatusTarget>>,
}

#[derive(Deserialize)]
struct StatusTarget {
    #[serde(rename = "entry")]
    entries: Option<Vec<StatusEntry>>,
}

#[derive(Deserialize)]
struct StatusEntry {
    #[serde(rename = "@path")]
    path: String,
    #[serde(rename = "wc-status")]
    wc_status: WcStatus,
}

#[derive(Deserialize)]
struct WcStatus {
    #[serde(rename = "@item")]
    item: String,
    #[serde(rename = "@props")]
    _props: Option<String>,
    #[serde(rename = "@copy-from-url")]
    copy_from_url: Option<String>,
    #[serde(rename = "@kind")]
    kind: Option<String>,
    #[serde(rename = "@tree-conflicted")]
    tree_conflicted: Option<String>,
}

pub fn parse_status_xml(xml: &str) -> Result<Vec<FileStatus>, AppError> {
    let status: StatusXml = from_str(xml)
        .map_err(|e| AppError::svn_parse(format!("Failed to parse status XML: {}", e)))?;

    let targets = status.targets.unwrap_or_default();

    let mut all_entries = Vec::new();
    for target in targets {
        let entries = target.entries.unwrap_or_default();
        all_entries.extend(entries);
    }

    Ok(all_entries
        .into_iter()
        .map(|entry| {
            let status_type = if entry.wc_status.tree_conflicted.as_deref() == Some("true") {
                FileStatusType::Conflicted
            } else {
                match entry.wc_status.item.as_str() {
                    "modified" => FileStatusType::Modified,
                    "added" => FileStatusType::Added,
                    "deleted" => FileStatusType::Deleted,
                    "unversioned" => FileStatusType::Unversioned,
                    "missing" => FileStatusType::Missing,
                    "conflicted" => FileStatusType::Conflicted,
                    _ => FileStatusType::Unversioned,
                }
            };

            FileStatus {
                path: entry.path,
                status: status_type,
                is_directory: entry.wc_status.kind.as_deref() == Some("dir"),
                copied: entry.wc_status.copy_from_url.map(|_| true),
            }
        })
        .collect())
}

pub async fn svn_status(path: &str, timeout_secs: u64) -> Result<Vec<FileStatus>, AppError> {
    let xml = crate::svn::run_svn_async(&["status", "--xml", "--depth", "infinity", path], timeout_secs).await?;
    parse_status_xml(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_xml_modified() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target>
    <entry path="src/main.rs">
      <wc-status item="modified" revision="100"/>
    </entry>
  </target>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "src/main.rs");
        assert_eq!(result[0].status, FileStatusType::Modified);
    }

    #[test]
    fn test_parse_status_xml_multiple() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target>
    <entry path="a.txt">
      <wc-status item="modified"/>
    </entry>
    <entry path="b.txt">
      <wc-status item="added"/>
    </entry>
    <entry path="c.txt">
      <wc-status item="deleted"/>
    </entry>
  </target>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].status, FileStatusType::Modified);
        assert_eq!(result[1].status, FileStatusType::Added);
        assert_eq!(result[2].status, FileStatusType::Deleted);
    }

    #[test]
    fn test_parse_status_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target/>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 0);
    }
}
