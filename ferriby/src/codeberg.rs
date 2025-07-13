use std::cell::LazyCell;

use chrono::{DateTime, offset::Utc};
use http::{HeaderMap, header};
use regex::Regex;
use reqwest::Url;

use crate::app::ActivitySource;
use crate::githoster::get_with_headers;

#[derive(Debug, Clone, PartialEq)]
pub struct CodebergSource {
    pub owner: String,
    pub repo: String,
    pub pat: Option<String>,
}

impl Default for CodebergSource {
    fn default() -> Self {
        Self {
            owner: "rust-lang".into(),
            repo: "rust".into(),
            pat: None,
        }
    }
}

impl ActivitySource for CodebergSource {
    async fn get_last_activity(self) -> Option<DateTime<Utc>> {
        let url = format!(
            "https://codeberg.org/api/v1/repos/{}/{}",
            self.owner, self.repo
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
            let cb_pat = header::HeaderValue::from_str(format!("token {token}").as_str())
                .expect("bad codeberg pat");
            headers.insert(header::AUTHORIZATION, cb_pat);
        }

        match get_with_headers(url, headers).await {
            Some(body) => {
                let timestamps = CodebergSource::parse_timestamps(body.as_str());
                timestamps.into_iter().max()
            }
            None => None,
        }
    }
}

impl CodebergSource {
    fn parse_timestamps(response: &str) -> Vec<DateTime<Utc>> {
        let re: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new("\"updated_at\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d[+-]\\d\\d:\\d\\d)\"").unwrap()
        });

        re.captures_iter(response)
            .map(|m| {
                let s = m.get(1).unwrap().as_str();
                let dt = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
                    .expect("unexpected timestamp format");
                let secs = dt.timestamp();
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
    fn codeberg_parse_positive_offset() {
        let s = "\"updated_at\":\"2025-07-11T12:30:20+02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22+02:00\"";
        let parsed = CodebergSource::parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 7);
        assert_eq!(parsed[0].day(), 11);
        assert_eq!(parsed[0].hour(), 10);
        assert_eq!(parsed[0].minute(), 30);
        assert_eq!(parsed[0].second(), 20);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 11);
        assert_eq!(parsed[1].hour(), 11);
        assert_eq!(parsed[1].minute(), 31);
        assert_eq!(parsed[1].second(), 22);
    }

    #[test]
    fn codeberg_parse_negative_offset() {
        let s = "\"updated_at\":\"2025-07-11T12:30:20-02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22-02:00\"";
        let parsed = CodebergSource::parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 7);
        assert_eq!(parsed[0].day(), 11);
        assert_eq!(parsed[0].hour(), 14);
        assert_eq!(parsed[0].minute(), 30);
        assert_eq!(parsed[0].second(), 20);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 11);
        assert_eq!(parsed[1].hour(), 15);
        assert_eq!(parsed[1].minute(), 31);
        assert_eq!(parsed[1].second(), 22);
    }
}
