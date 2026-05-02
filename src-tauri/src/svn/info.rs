use crate::common::AppError;
use crate::svn::models::RepoInfo;
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct InfoXml {
    #[serde(rename = "entry")]
    entry: Option<InfoDetail>,
}

#[derive(Deserialize)]
struct InfoDetail {
    url: Option<String>,
    revision: Option<String>,
    repository: Option<InfoRepository>,
    #[serde(rename = "wc-info")]
    wc_info: Option<InfoWcInfo>,
    commit: Option<InfoCommit>,
}

#[derive(Deserialize)]
struct InfoRepository {
    root: Option<String>,
}

#[derive(Deserialize)]
struct InfoWcInfo {
    #[serde(rename = "revision")]
    revision: Option<String>,
}

#[derive(Deserialize)]
struct InfoCommit {
    #[serde(rename = "@revision")]
    revision: Option<u64>,
    author: Option<String>,
    date: Option<String>,
}

fn parse_info_xml(xml: &str) -> Result<RepoInfo, AppError> {
    let info_xml: InfoXml =
        from_str(xml).map_err(|e| AppError::Svn(format!("Failed to parse info XML: {}", e)))?;

    let entry = info_xml
        .entry
        .ok_or_else(|| AppError::Svn("No entry found in info XML".to_string()))?;

    let url = entry.url.unwrap_or_default();
    let revision = entry
        .revision
        .and_then(|r| r.parse::<u64>().ok())
        .unwrap_or(0);
    let root = entry
        .repository
        .and_then(|r| r.root)
        .unwrap_or_default();

    let last_changed_rev = entry
        .commit
        .as_ref()
        .and_then(|c| c.revision)
        .unwrap_or(revision);
    let last_changed_date = entry
        .commit
        .as_ref()
        .and_then(|c| c.date.clone())
        .unwrap_or_default();
    let last_changed_author = entry
        .commit
        .as_ref()
        .and_then(|c| c.author.clone())
        .unwrap_or_default();

    Ok(RepoInfo {
        url,
        root,
        revision,
        last_changed_rev,
        last_changed_date,
        last_changed_author,
    })
}

pub async fn svn_info(path: &str, timeout_secs: u64) -> Result<RepoInfo, AppError> {
    let xml = crate::svn::run_svn_utf8_async(&["info", "--xml", path], timeout_secs).await?;
    parse_info_xml(&xml)
}

pub struct RepoInfoParsed {
    pub url: String,
    pub wc_revision: u64,
}

pub fn parse_info_for_log(xml: &str) -> Result<RepoInfoParsed, AppError> {
    let info_xml: InfoXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Svn(format!("Failed to parse info XML: {}", e)))?;
    let entry = info_xml
        .entry
        .ok_or_else(|| AppError::Svn("No entry found in info XML".to_string()))?;
    let url = entry.url.unwrap_or_default();
    let wc_revision = entry
        .wc_info
        .and_then(|w| w.revision)
        .and_then(|r| r.parse::<u64>().ok())
        .unwrap_or(0);
    Ok(RepoInfoParsed { url, wc_revision })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_info_xml() {
        let xml = r#"<?xml version="1.0"?>
<info>
  <entry kind="dir" path="/trunk" revision="1">
    <url>svn://repo/trunk</url>
    <repository>
      <root>svn://repo</root>
    </repository>
    <wc-info>
      <revision>1</revision>
    </wc-info>
    <commit revision="1">
      <author>alice</author>
      <date>2026-04-30T10:00:00Z</date>
    </commit>
  </entry>
</info>"#;
        let result = parse_info_xml(xml).unwrap();
        assert_eq!(result.url, "svn://repo/trunk");
        assert_eq!(result.root, "svn://repo");
        assert_eq!(result.revision, 1);
        assert_eq!(result.last_changed_rev, 1);
        assert_eq!(result.last_changed_author, "alice");
    }

    #[test]
    fn test_parse_info_xml_no_entry() {
        let xml = r#"<?xml version="1.0"?>
<info/>"#;
        let result = parse_info_xml(xml);
        assert!(result.is_err());
    }
}
