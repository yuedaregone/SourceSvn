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
    #[serde(rename = "revision")]
    revision: Option<u64>,
}

fn parse_info_xml(xml: &str) -> Result<RepoInfo, AppError> {
    let info_xml: InfoXml =
        from_str(xml).map_err(|e| AppError::Svn(format!("Failed to parse info XML: {}", e)))?;

    let entry = info_xml
        .entry
        .ok_or_else(|| AppError::Svn("No entry found in info XML".to_string()))?;

    let root = entry
        .repository
        .and_then(|r| r.root)
        .unwrap_or_default();

    let last_changed_rev = entry
        .wc_info
        .as_ref()
        .and_then(|w| w.revision)
        .unwrap_or(entry.revision);

    Ok(RepoInfo {
        url: entry.url,
        root,
        revision: entry.revision,
        last_changed_rev,
        last_changed_date: String::new(),
        last_changed_author: String::new(),
    })
}

pub async fn svn_info(path: &str, timeout_secs: u64) -> Result<RepoInfo, AppError> {
    let xml = crate::svn::run_svn_async(&["info", "--xml", path], timeout_secs).await?;
    parse_info_xml(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_info_xml() {
        let xml = r#"<?xml version="1.0"?>
<info>
  <entry kind="dir" url="svn://repo/trunk" revision="42">
    <repository>
      <root>svn://repo</root>
    </repository>
    <wc-info>
      <revision>30</revision>
    </wc-info>
  </entry>
</info>"#;
        let result = parse_info_xml(xml).unwrap();
        assert_eq!(result.url, "svn://repo/trunk");
        assert_eq!(result.root, "svn://repo");
        assert_eq!(result.revision, 42);
        assert_eq!(result.last_changed_rev, 30);
    }

    #[test]
    fn test_parse_info_xml_no_entry() {
        let xml = r#"<?xml version="1.0"?>
<info/>"#;
        let result = parse_info_xml(xml);
        assert!(result.is_err());
    }
}
