use crate::app::{App, Source};
use ferriby_sources::{git::GitSource, github::GitHubSource};
use std::env;

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        eprintln!(
            "Usage: {} [<github_owner> <github_repo> | <path_to_repo>]",
            args[0]
        );
        std::process::exit(1);
    }
    let source = {
        if args.len() == 3 {
            let github_source = GitHubSource {
                owner: args[1].clone(),
                repo: args[2].clone(),
            };
            Source::GitHub(github_source)
        } else {
            let git_source = GitSource {
                path: args[1].clone(),
            };
            Source::Git(git_source)
        }
    };
    let terminal = ratatui::init();
    let result = App::new(source).run(terminal).await;
    ratatui::restore();
    result
}
