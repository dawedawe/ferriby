use chrono::{DateTime, offset::Utc};
use git2::{BranchType, Repository, RepositoryOpenFlags};

#[derive(Debug, Clone)]
pub struct GitSource {
    pub path: String,
}

impl Default for GitSource {
    fn default() -> Self {
        Self { path: ".".into() }
    }
}

pub async fn get_last_event(source: GitSource) -> Option<DateTime<Utc>> {
    let repo = match Repository::open_ext(
        source.path,
        RepositoryOpenFlags::CROSS_FS,
        &[] as &[&std::ffi::OsStr],
    ) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open git repository: {e}"),
    };

    let mut branch_names = vec![];

    match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => branches.for_each(|b| {
            if let Ok(b) = b {
                if let Ok(Some(branch_name)) = b.0.name() {
                    branch_names.push(branch_name.to_string())
                }
            }
        }),
        Err(e) => panic!("failed to get branches {}", e),
    };

    let mut branch_times = vec![];

    for branch_name in branch_names {
        let branch_reference = repo
            .find_reference(&format!("refs/heads/{}", branch_name))
            .unwrap_or_else(|e| panic!("find_reference failed: {e}"));
        let target = branch_reference
            .target()
            .expect("Branch reference must have a target");

        // Resolve the target to get the commit
        let commit = repo
            .find_commit(target)
            .unwrap_or_else(|e| panic!("find_commit failed: {e}"));
        let secs_since_epoch = commit.time().seconds();
        match DateTime::from_timestamp(secs_since_epoch, 0) {
            Some(t) => branch_times.push(t),
            None => panic!("DateTime::from_timestamp() failed"),
        }
    }

    branch_times.into_iter().max()
}
