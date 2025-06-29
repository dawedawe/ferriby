use crate::app::{App, Source};
use ferriby_sources::{git::GitSource, github::GitHubSource};
use std::env;

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let sources = parse_args();
    let terminal = ratatui::init();
    let result = App::new(sources).run(terminal).await;
    ratatui::restore();
    result
}

fn parse_args() -> Vec<Source> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage(args[0].clone())
    }

    let chunks = args[1..].chunks(2);
    let mut sources = vec![];
    for chunk in chunks {
        if chunk.len() != 2 {
            usage(args[0].clone());
        }

        let source = if chunk[0] == "-gh" {
            let pat = match std::env::var("FERRIBY_GH_PAT") {
                Ok(token) if !token.is_empty() => Some(token),
                _ => None,
            };
            let gh_arg: Vec<&str> = chunk[1].split("/").collect();
            let github_source = GitHubSource {
                owner: gh_arg[0].to_string(),
                repo: gh_arg[1].to_string(),
                pat,
            };
            Source::GitHub(github_source)
        } else if chunk[0] == "-g" {
            let git_source = GitSource {
                path: chunk[1].clone(),
            };
            Source::Git(git_source)
        } else {
            usage(args[0].clone());
        };
        sources.push(source);
    }

    sources
}

fn usage(name: String) -> ! {
    eprintln!(
        "Usage: {} [-gh <owner>/<repository> | -g <path_to_repo>]",
        name
    );
    std::process::exit(1);
}
