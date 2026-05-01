use crate::common::AppError;
use crate::svn::models::UpdateResult;
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct UpdateXml {
    _target: Option<String>,
    #[serde(rename = "revision")]
    revision: Option<String>,
    #[serde(rename = "added")]
    added: Option<Vec<UpdatePath>>,
    #[serde(rename = "unversioned")]
    unversioned: Option<Vec<UpdatePath>>,
}

#[derive(Deserialize, Debug)]
struct UpdatePath {
    path: Option<String>,
}

fn parse_update_xml(xml: &str) -> Result<UpdateResult, AppError> {
    let report: UpdateXml =
        from_str(xml).map_err(|e| AppError::Svn(format!("Failed to parse update XML: {}", e)))?;

    let revision = report
        .revision
        .as_ref()
        .and_then(|r| r.parse::<u64>().ok())
        .unwrap_or(0);

    let updated_files = report
        .added
        .as_ref()
        .map(|paths| paths.iter().filter_map(|p| p.path.clone()).collect())
        .unwrap_or_default();

    let conflicts = report
        .unversioned
        .as_ref()
        .map(|paths| paths.iter().filter_map(|p| p.path.clone()).collect())
        .unwrap_or_default();

    Ok(UpdateResult {
        revision,
        updated_files,
        merged_files: vec![],
        conflicts,
    })
}

pub async fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn_async(&["update", "--xml", path], timeout_secs).await?;
    parse_update_xml(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_update_xml_with_revision() {
        let xml = r#"<?xml version="1.0"?>
<update-report>
  <target>.</target>
  <revision>105</revision>
  <added>
    <path>new_file.rs</path>
  </added>
  <added>
    <path>src/lib.rs</path>
  </added>
</update-report>"#;
        let result = parse_update_xml(xml).unwrap();
        assert_eq!(result.revision, 105);
        assert_eq!(result.updated_files.len(), 2);
        assert_eq!(result.updated_files[0], "new_file.rs");
    }

    #[test]
    fn test_parse_update_xml_no_changes() {
        let xml = r#"<?xml version="1.0"?>
<update-report>
  <target>.</target>
  <revision>100</revision>
</update-report>"#;
        let result = parse_update_xml(xml).unwrap();
        assert_eq!(result.revision, 100);
        assert!(result.updated_files.is_empty());
    }
}
