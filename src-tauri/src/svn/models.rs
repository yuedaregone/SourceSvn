use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatusType {
    Modified,
    Added,
    Deleted,
    Unversioned,
    Missing,
    Conflicted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub revision: u64,
    pub author: String,
    pub date: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_paths: Option<Vec<ChangedPath>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangedPath {
    pub path: String,
    pub action: PathAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_from_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_from_rev: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PathAction {
    A,
    M,
    D,
    R,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    pub url: String,
    pub root: String,
    pub revision: u64,
    pub last_changed_rev: u64,
    pub last_changed_date: String,
    pub last_changed_author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub revision: u64,
    pub author: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    File,
    Dir,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShelveInfo {
    pub name: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum DiffTarget {
    #[serde(rename_all = "camelCase")]
    File {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    FileAtRevision {
        path: String,
        base_revision: String,
        revision: String,
    },
    #[serde(rename_all = "camelCase")]
    Revisions {
        old_rev: String,
        new_rev: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitResult {
    pub revision: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateResult {
    pub revision: u64,
    pub updated_files: Vec<String>,
    pub merged_files: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReviewChunkEvent {
    pub content: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WcLogResult {
    pub entries: Vec<LogEntry>,
    pub wc_revision: u64,
}
