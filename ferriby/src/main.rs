use crate::app::App;
use ferriby_sources::github::GitHubSource;
use std::env;

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <github_owner> <github_repo>", args[0]);
        std::process::exit(1);
    }
    let source = GitHubSource {
        owner: args[1].clone(),
        repo: args[2].clone(),
    };
    let terminal = ratatui::init();
    let result = App::new(source).run(terminal).await;
    ratatui::restore();
    result
}
