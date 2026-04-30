use crate::common::{AppError, DiffTarget};

pub fn svn_diff(
    path: &str,
    target: &DiffTarget,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let rev_range;

    let mut args: Vec<&str> = vec!["diff", "--xml", path];

    match target {
        DiffTarget::File { path: file_path, revision } => {
            args.push(file_path);
            if let Some(rev) = revision {
                args.push("-r");
                args.push(rev);
            }
        }
        DiffTarget::Revisions { old_rev, new_rev } => {
            rev_range = format!("{}:{}", old_rev, new_rev);
            args.push("-r");
            args.push(&rev_range);
        }
    }

    crate::svn::run_svn(&args, timeout_secs)
}
