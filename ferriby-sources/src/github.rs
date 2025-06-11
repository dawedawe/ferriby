use std::sync::Arc;

use chrono::{DateTime, offset::Utc};

pub async fn get_last_gh_repo_event(
    owner: impl Into<String>,
    repo: impl Into<String>,
) -> Option<DateTime<Utc>> {
    let instance = octocrab::instance();
    let instance = match std::env::var("FERRIBY_GH_PAT") {
        Ok(token) => Arc::new(instance.user_access_token(token).unwrap()),
        Err(_) => instance,
    };

    let repo = instance.repos(owner, repo);
    let r = repo.list_commits().send().await;
    match r {
        Ok(mut value) => {
            let items = value.take_items();
            if items.is_empty() {
                None
            } else {
                items
                    .iter()
                    .filter_map(|c| c.commit.committer.as_ref().and_then(|c| c.date))
                    .max()
            }
        }
        Err(_) => None,
    }
}
