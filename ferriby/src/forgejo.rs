use std::cell::LazyCell;

use chrono::NaiveDateTime;
use chrono::{DateTime, offset::Utc};
use http::{HeaderMap, header};
use regex::Regex;
use reqwest::Url;

use crate::app::ActivitySource;
use crate::githoster::get_with_headers;

#[derive(Debug, Clone, PartialEq)]
pub struct ForgejoSource {
    pub base_url: Url,
    pub owner: String,
    pub repo: String,
    pub pat: Option<String>,
}

impl ActivitySource for ForgejoSource {
    async fn get_last_activity(self) -> Option<DateTime<Utc>> {
        let url = self
            .base_url
            .join(format!("api/v1/repos/{}/{}", self.owner, self.repo).as_str())
            .unwrap();

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
            let pat = header::HeaderValue::from_str(format!("token {token}").as_str())
                .expect("bad forgejo pat");
            headers.insert(header::AUTHORIZATION, pat);
        }

        match get_with_headers(url, headers).await {
            Some(body) => {
                let timestamps = ForgejoSource::parse_timestamps(body.as_str());
                timestamps.into_iter().max()
            }
            None => None,
        }
    }
}

impl ForgejoSource {
    // forgejo on sqlite:     "updated_at":"2025-08-04T20:26:36Z",
    // forgejo on postgres:  "updated_at":"2025-08-09T11:51:12+02:00"
    fn parse_timestamps(response: &str) -> Vec<DateTime<Utc>> {
        let re: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new("\"updated_at\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\dZ)\"|\"updated_at\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d[+-]\\d\\d:\\d\\d)\"")
                .unwrap()
        });

        re.captures_iter(response)
            .map(|m| {
                if m.get(1).is_some() {
                    let s = m.get(1).unwrap().as_str();
                    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                        .expect("unexpected timestamp format");
                    let secs = dt.and_utc().timestamp();
                    DateTime::from_timestamp(secs, 0).expect("from_timestamp failed")
                } else {
                    let s = m.get(2).unwrap().as_str();
                    let dt = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
                        .expect("unexpected timestamp format");
                    let secs = dt.timestamp();
                    DateTime::from_timestamp(secs, 0).expect("from_timestamp failed")
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::*;

    #[test]
    fn forgejo_parse_positive_offset() {
        let s = "\"updated_at\":\"2025-07-11T12:30:20+02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22+02:00\"";
        let parsed = ForgejoSource::parse_timestamps(s);

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
    fn forgejo_parse_negative_offset() {
        let s = "\"updated_at\":\"2025-07-11T12:30:20-02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22-02:00\"";
        let parsed = ForgejoSource::parse_timestamps(s);

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

    #[test]
    fn forgejo_parse_mixed_tz_info() {
        let s = "\"updated_at\":\"2025-08-04T20:26:36Z\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22-02:00\"";
        let parsed = ForgejoSource::parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 8);
        assert_eq!(parsed[0].day(), 4);
        assert_eq!(parsed[0].hour(), 20);
        assert_eq!(parsed[0].minute(), 26);
        assert_eq!(parsed[0].second(), 36);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 11);
        assert_eq!(parsed[1].hour(), 15);
        assert_eq!(parsed[1].minute(), 31);
        assert_eq!(parsed[1].second(), 22);
    }
}
