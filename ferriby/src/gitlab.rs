use std::cell::LazyCell;

use chrono::NaiveDateTime;
use chrono::{DateTime, offset::Utc};
use http::{HeaderMap, header};
use regex::Regex;
use reqwest::Url;

use crate::app::ActivitySource;
use crate::githoster::get_with_headers;

#[derive(Debug, Clone, PartialEq)]
pub struct GitLabSource {
    pub hostname: String,
    pub project_id: String,
    pub project_name: String,
    pub pat: Option<String>,
}

impl ActivitySource for GitLabSource {
    async fn get_last_activity(self) -> Option<DateTime<Utc>> {
        let url = format!(
            "https://{}/api/v4/projects/{}/events",
            self.hostname, self.project_id
        );
        let url = Url::parse(url.as_str()).expect("Url creation failed");

        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("ferriby"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        if let Some(token) = &self.pat {
            let pat = header::HeaderValue::from_str(token.as_str()).expect("bad gitlab pat");
            headers.insert("PRIVATE-TOKEN", pat);
        }

        match get_with_headers(url, headers).await {
            Some(body) => {
                let timestamps = GitLabSource::parse_timestamps(body.as_str());
                timestamps.into_iter().max()
            }
            None => None,
        }
    }
}

impl GitLabSource {
    fn parse_timestamps(response: &str) -> Vec<DateTime<Utc>> {
        let re: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(
                "\"created_at\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d.\\d\\d\\dZ)\"",
            )
            .unwrap()
        });

        re.captures_iter(response)
            .map(|m| {
                let s = m.get(1).unwrap().as_str();
                let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ")
                    .expect("unexpected timestamp format");
                let secs = dt.and_utc().timestamp();
                DateTime::from_timestamp(secs, 0).expect("from_timestamp failed")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::*;

    #[test]
    fn github_parse() {
        let s = "\"created_at\":\"2025-07-14T21:12:15.564Z\" bla foo\
            \"created_at\":\"2025-07-14T21:12:15.137Z\"";
        let parsed = GitLabSource::parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 7);
        assert_eq!(parsed[0].day(), 14);
        assert_eq!(parsed[0].hour(), 21);
        assert_eq!(parsed[0].minute(), 12);
        assert_eq!(parsed[0].second(), 15);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 14);
        assert_eq!(parsed[1].hour(), 21);
        assert_eq!(parsed[1].minute(), 12);
        assert_eq!(parsed[1].second(), 15);
    }
}
