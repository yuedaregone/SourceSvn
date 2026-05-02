use sourcesvn_lib::svn;

const REPO_PATH: &str = r"D:\Study\Test\trunk";
const TIMEOUT: u64 = 30;

/// Run `svn cleanup` on the working copy to release stale locks.
async fn cleanup_wc() {
    let _ = svn::run_svn_async_in_dir(&["cleanup"], TIMEOUT, Some(REPO_PATH)).await;
}

#[tokio::test]
async fn test_find_svn_executable() {
    let result = svn::find_svn_executable();
    assert!(result.is_ok(), "find_svn_executable failed: {:?}", result);
    let path = result.unwrap();
    println!("svn found at: {}", path);
    assert!(!path.is_empty());
}

#[tokio::test]
async fn test_svn_status() {
    let result = svn::status::svn_status(REPO_PATH, TIMEOUT).await;
    assert!(result.is_ok(), "svn_status failed: {:?}", result);
    let statuses = result.unwrap();
    println!("status entries: {}", statuses.len());
    for s in &statuses {
        println!("  {} -> {:?}", s.path, s.status);
    }
}

#[tokio::test]
async fn test_svn_info() {
    let result = svn::info::svn_info(REPO_PATH, TIMEOUT).await;
    assert!(result.is_ok(), "svn_info failed: {:?}", result);
    let info = result.unwrap();
    println!("url: {}", info.url);
    println!("root: {}", info.root);
    println!("revision: {}", info.revision);
    assert!(!info.url.is_empty());
    assert!(!info.root.is_empty());
}

#[tokio::test]
async fn test_svn_log() {
    let result = svn::log::svn_log(REPO_PATH, Some(5), None, TIMEOUT).await;
    assert!(result.is_ok(), "svn_log failed: {:?}", result);
    let entries = result.unwrap();
    println!("log entries: {}", entries.len());
    for e in &entries {
        println!("  r{} by {}: {}", e.revision, e.author, e.message);
    }
}

#[tokio::test]
async fn test_svn_diff_no_changes() {
    let target = sourcesvn_lib::svn::models::DiffTarget::File {
        path: "README.txt".to_string(),
        revision: None,
    };
    let result = svn::diff::svn_diff(REPO_PATH, &target, TIMEOUT).await;
    assert!(result.is_ok(), "svn_diff failed: {:?}", result);
    let diff = result.unwrap();
    println!("diff output length: {}", diff.len());
}

#[tokio::test]
async fn test_svn_list() {
    let result = svn::list::svn_list(REPO_PATH, None, false, TIMEOUT).await;
    assert!(result.is_ok(), "svn_list failed: {:?}", result);
    let entries = result.unwrap();
    println!("list entries: {}", entries.len());
    for e in &entries {
        println!("  {} ({:?})", e.name, e.kind);
    }
}

#[tokio::test]
async fn test_svn_cat() {
    let result = svn::cat::svn_cat(REPO_PATH, Some("1"), TIMEOUT).await;
    if let Err(e) = &result {
        println!("svn_cat error (may be expected for binary or missing): {}", e);
    }
    if let Ok(content) = &result {
        println!("cat content length: {}", content.len());
        println!("cat preview: {}", &content[..content.len().min(100)]);
    }
}

#[tokio::test]
async fn test_full_commit_flow() {
    cleanup_wc().await;
    // 1. Get status first
    let statuses = svn::status::svn_status(REPO_PATH, TIMEOUT).await.unwrap();
    println!("=== pre-commit status ===");
    let files_to_commit: Vec<String> = statuses.iter().map(|s| s.path.clone()).collect();
    for s in &statuses {
        println!("  {} -> {:?}", s.path, s.status);
    }

    if files_to_commit.is_empty() {
        println!("No changes to commit, skipping commit flow test");
        return;
    }

    // 2. Commit (should auto-add unversioned files)
    let result = svn::commit::svn_commit(
        REPO_PATH,
        "integration test: auto commit from test",
        &files_to_commit,
        TIMEOUT,
    )
    .await;
    assert!(result.is_ok(), "svn_commit failed: {:?}", result);
    let commit_result = result.unwrap();
    println!("commit result: rev={}, success={}", commit_result.revision, commit_result.success);
    assert!(commit_result.revision > 0, "revision should be > 0");

    // 3. Verify status after commit
    let statuses_after = svn::status::svn_status(REPO_PATH, TIMEOUT).await.unwrap();
    println!("=== post-commit status ===");
    for s in &statuses_after {
        println!("  {} -> {:?}", s.path, s.status);
    }
    assert!(
        statuses_after.is_empty() || statuses_after.iter().all(|s| s.status != sourcesvn_lib::svn::models::FileStatusType::Unversioned),
        "all previously unversioned files should now be committed"
    );
}

#[tokio::test]
async fn test_svn_update() {
    let result = svn::update::svn_update(REPO_PATH, TIMEOUT).await;
    assert!(result.is_ok(), "svn_update failed: {:?}", result);
    let update_result = result.unwrap();
    println!("update: rev={}, files={}",
        update_result.revision,
        update_result.files.len()
    );
}

#[tokio::test]
async fn test_modify_and_commit() {
    cleanup_wc().await;
    // Use a dedicated subdirectory to avoid conflicting with test_full_commit_flow
    let subdir = format!(r"{}\test_dir", REPO_PATH);
    let _ = std::fs::create_dir(&subdir);

    let test_file = format!(r"{}\test_modify_new.txt", subdir);
    std::fs::write(&test_file, "line 1\n").unwrap();

    // Auto-add handles unversioned files, but SVN requires parent dirs to be committed too
    // So pass both the dir and file together
    let commit_result = svn::commit::svn_commit(
        REPO_PATH,
        "test: add test_dir and test_modify_new.txt",
        &["test_dir".to_string(), "test_dir/test_modify_new.txt".to_string()],
        TIMEOUT,
    )
    .await;
    assert!(commit_result.is_ok(), "commit after add failed: {:?}", commit_result);
    let cr = commit_result.unwrap();
    println!("committed r{}", cr.revision);

    // Modify the file
    std::fs::write(&test_file, "line 1\nline 2\n").unwrap();

    // Commit the modification
    let commit_result2 = svn::commit::svn_commit(
        REPO_PATH,
        "test: modify test_modify_new.txt",
        &["test_dir/test_modify_new.txt".to_string()],
        TIMEOUT,
    )
    .await;
    assert!(commit_result2.is_ok(), "commit after modify failed: {:?}", commit_result2);
    let cr2 = commit_result2.unwrap();
    println!("committed r{}", cr2.revision);

    // Delete the file
    std::fs::remove_file(&test_file).unwrap();

    // Commit the deletion
    let commit_result3 = svn::commit::svn_commit(
        REPO_PATH,
        "test: delete test_modify_new.txt",
        &["test_dir/test_modify_new.txt".to_string()],
        TIMEOUT,
    )
    .await;
    assert!(commit_result3.is_ok(), "commit after delete failed: {:?}", commit_result3);
    let cr3 = commit_result3.unwrap();
    println!("committed r{}", cr3.revision);

    // Cleanup: remove the subdirectory from version control
    svn::run_svn_async_in_dir(
        &["delete", "test_dir", "--force"],
        TIMEOUT,
        Some(REPO_PATH),
    ).await.ok();
    svn::commit::svn_commit(
        REPO_PATH,
        "test: cleanup test_dir",
        &["test_dir".to_string()],
        TIMEOUT,
    ).await.ok();
}
