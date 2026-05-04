use crate::common::AppError;
use crate::svn::models::{DirEntry, EntryKind};
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct ListXml {
    #[serde(rename = "list")]
    lists: Option<Vec<ListContainer>>,
}

#[derive(Deserialize)]
struct ListContainer {
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
    let list_xml: ListXml =
        from_str(xml).map_err(|e| AppError::Svn(format!("Failed to parse list XML: {}", e)))?;

    let entries: Vec<ListItem> = list_xml
        .lists
        .unwrap_or_default()
        .into_iter()
        .flat_map(|c| c.entries.unwrap_or_default())
        .collect();

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

pub async fn svn_list(
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
    let xml = crate::svn::run_svn_async(&args, timeout_secs).await?;
    parse_list_xml(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_xml() {
        let xml = r#"<?xml version="1.0"?>
<lists>
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
</list>
</lists>"#;
        let result = parse_list_xml(xml).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].kind, EntryKind::Dir);
        assert_eq!(result[0].name, "src");
        assert_eq!(result[1].kind, EntryKind::File);
        assert_eq!(result[1].size, Some(1234));
    }
}
