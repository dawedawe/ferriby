use std::sync::Arc;

use chrono::{DateTime, offset::Utc};

pub async fn get_last_gh_repo_event(
    owner: impl Into<String>,
    repo: impl Into<String>,
) -> Option<DateTime<Utc>> {
    let instance = octocrab::instance();
    let instance = match std::env::var("FERRIBY_GH_PAT") {
        Ok(token) if !token.is_empty() => Arc::new(instance.user_access_token(token).unwrap()),
        _ => instance,
    };

    let owner: String = owner.into();
    let repo: String = repo.into();
    let repos = instance.repos(owner.clone(), repo.clone());
    let r = repos.list_commits().send().await;
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
        Err(_) => {
            panic!(
                "Failed to list commits of github repo {}/{}. \
                    This could mean we don't have access to it or there are no commits so far.",
                owner, repo
            );
        }
    }
}
