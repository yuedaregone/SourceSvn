use crate::common::AppError;
use crate::svn::diff::{is_binary_bytes, make_binary_placeholder};

pub async fn svn_cat(
    path: &str,
    revision: Option<&str>,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["cat", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    crate::svn::run_svn_async(&args, timeout_secs).await
}

/// 在指定工作目录下执行 svn cat（用于本地工作副本的 BASE 版本），二进制文件返回占位符
pub async fn svn_cat_in_dir(
    repo_path: &str,
    file_path: &str,
    revision: Option<&str>,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["cat"];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    args.push(file_path);
    let bytes = crate::svn::run_svn_async_in_dir_bytes(&args, timeout_secs, Some(repo_path)).await?;
    if is_binary_bytes(&bytes) {
        return Ok(make_binary_placeholder(&bytes));
    }
    Ok(crate::svn::decode_bytes(&bytes))
}

/// 通过工作目录和相对路径获取指定版本的文件内容（自动构造完整 URL），二进制文件返回占位符
pub async fn svn_cat_at_revision(
    repo_path: &str,
    file_path: &str,
    revision: &str,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let info_xml = crate::svn::run_svn_async(&["info", "--xml", repo_path], timeout_secs).await?;
    let info = crate::svn::info::parse_info_for_log(&info_xml)?;
    let base_url = if info.root.is_empty() { &info.url } else { &info.root };
    let file_url = if base_url.ends_with('/') {
        format!("{}{}", base_url, file_path)
    } else {
        format!("{}/{}", base_url, file_path)
    };
    let args = vec!["cat", "-r", revision, &file_url];
    let bytes = crate::svn::run_svn_async_bytes(&args, timeout_secs).await?;
    if is_binary_bytes(&bytes) {
        return Ok(make_binary_placeholder(&bytes));
    }
    Ok(crate::svn::decode_bytes(&bytes))
}
