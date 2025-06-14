use chrono::{DateTime, offset::Utc};
use git2::{Repository, RepositoryOpenFlags};

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
        Err(e) => panic!("failed to open git repository: {}", e),
    };

    let mut revwalk = match repo.revwalk() {
        Ok(revwalk) => revwalk,
        Err(e) => panic!("failed to get revwalk: {}", e),
    };

    let _ = revwalk
        .push_head()
        .inspect_err(|e| panic!("push_head() failed: {}", e));

    match revwalk.next() {
        Some(Ok(oid)) => match repo.find_commit(oid) {
            Ok(commit) => {
                let secs_since_epoch = commit.time().seconds();
                match DateTime::from_timestamp(secs_since_epoch, 0) {
                    Some(t) => Some(t),
                    None => panic!("DateTime::from_timestamp() failed"),
                }
            }
            Err(e) => panic!("find_commit() failed: {}", e),
        },
        Some(Err(e)) => panic!("revwalk.next() failed: {}", e),
        None => None,
    }
}
