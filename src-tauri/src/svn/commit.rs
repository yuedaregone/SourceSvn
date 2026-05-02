use crate::common::AppError;
use crate::svn::models::{CommitResult, FileStatusType};
use std::collections::HashSet;

fn debug_log(msg: &str) {
    let path = std::env::temp_dir().join("sourcesvn_debug.log");
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", msg);
    }
}

fn debug_log_bytes(label: &str, bytes: &[u8]) {
    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    debug_log(&format!("{}: [hex] {}  [len] {}", label, hex.join(" "), bytes.len()));
}

/// Write the commit message to a temp file and commit with -F,
/// avoiding command-line argument encoding issues on Windows.
///
/// On Chinese Windows, `svn commit -m "中文"` can corrupt the message because
/// SVN's internal handling may re-encode UTF-16 args through the system code page.
/// Writing to a file and using `-F` bypasses this: the bytes are read from disk
/// directly by SVN using the system code page, avoiding the argv encoding round-trip.
async fn commit_with_message_file(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<String, AppError> {
    let msg_file = std::path::PathBuf::from(path).join(".svn_commit_msg");

    // Write message as UTF-8 bytes.
    // svn.exe on Windows reads -F files as UTF-8, so the bytes must be UTF-8.
    #[cfg(target_os = "windows")]
    {
        debug_log_bytes("temp file UTF-8 bytes", message.as_bytes());
        std::fs::write(&msg_file, message.as_bytes())
            .map_err(|e| AppError::Fs(format!("Failed to write commit message file: {}", e)))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::write(&msg_file, message.as_bytes())
            .map_err(|e| AppError::Fs(format!("Failed to write commit message file: {}", e)))?;
    }

    let msg_file_str = msg_file.to_str().unwrap_or("");
    debug_log(&format!("svn args: commit -F {} {}", msg_file_str, files.join(" ")));
    let mut args = vec!["commit", "-F", msg_file_str];
    for f in files {
        args.push(f);
    }
    let result = crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await;

    // Clean up temp file
    let _ = std::fs::remove_file(&msg_file);

    result
}

pub async fn svn_commit(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<CommitResult, AppError> {
    // ===== DIAGNOSTIC: log message encoding at every stage =====
    debug_log("====== svn_commit called ======");
    debug_log(&format!("message (Rust String): {:?}", message));
    debug_log_bytes("message UTF-8 bytes", message.as_bytes());
    #[cfg(target_os = "windows")]
    {
        let (gbk, _, _) = encoding_rs::GBK.encode(message);
        debug_log_bytes("message GBK bytes", &gbk);
    }
    // Check what the system code page is
    debug_log(&format!("FILES: {:?}", files));
    // ===== END DIAGNOSTIC =====

    auto_add_unversioned(path, files, timeout_secs).await?;

    let output = commit_with_message_file(path, message, files, timeout_secs).await?;

    // ===== DIAGNOSTIC: log subprocess output =====
    debug_log(&format!("svn output: {}", output));
    debug_log_bytes("svn output UTF-8 bytes", output.as_bytes());
    // ===== END DIAGNOSTIC =====

    let revision = extract_revision_from_output(&output);
    debug_log(&format!("committed revision: {}", revision));
    debug_log("====== svn_commit done ======\n");

    Ok(CommitResult {
        revision,
        success: true,
        errors: None,
    })
}

async fn auto_add_unversioned(
    path: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<(), AppError> {
    let status_xml =
        crate::svn::run_svn_utf8_async_in_dir(&["status", "--xml"], timeout_secs, Some(path)).await?;
    let statuses = crate::svn::status::parse_status_xml(&status_xml)?;

    let unversioned: HashSet<&str> = statuses
        .iter()
        .filter(|s| s.status == FileStatusType::Unversioned)
        .map(|s| s.path.as_str())
        .collect();

    let to_add: Vec<&String> = files
        .iter()
        .filter(|f| {
            // Match both absolute and relative paths
            if unversioned.contains(f.as_str()) {
                return true;
            }
            // Strip repo path prefix to get relative path
            std::path::Path::new(f)
                .strip_prefix(path)
                .ok()
                .and_then(|rel| rel.to_str())
                .map(|rel| {
                    // "D:\repo\trunk\file.txt" -> "trunk/file.txt" or "file.txt" depending on strip
                    let rel_normalized = rel.trim_start_matches('\\').trim_start_matches('/');
                    unversioned.contains(rel_normalized)
                        || rel_normalized.split(['\\', '/']).last().map_or(false, |name| {
                            unversioned.iter().any(|u| u.ends_with(name))
                        })
                })
                .unwrap_or(false)
        })
        .collect();

    if to_add.is_empty() {
        return Ok(());
    }

    let mut add_args = vec!["add"];
    for f in &to_add {
        add_args.push(f);
    }
    crate::svn::run_svn_async_in_dir(&add_args, timeout_secs, Some(path)).await?;

    Ok(())
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
}
