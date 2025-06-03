use crate::error::ConfigParseError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub anki: AnkiConfig,
    pub rate_limit: u32,
    pub web_driver: WebDriverConfig,
    pub cookie: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnkiConfig {
    #[serde(default)]
    pub model: AnkiModelConfig,
    #[serde(default)]
    pub deck: AnkiDeckConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnkiModelConfig {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnkiDeckConfig {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverConfig {
    pub endpoint: Url,
    #[serde(default)]
    pub headless: bool,
}

#[derive(Debug)]
pub struct Config {
    pub anki: AnkiConfig,
    pub rate_limit: u32,
    pub web_driver: WebDriverConfig,
    pub cookie: LeetCodeCookies,
}

#[derive(Debug, Clone)]
pub struct LeetCodeCookies {
    pub csrf_token: String,
    pub leet_code_token: String,
}

impl FromStr for LeetCodeCookies {
    type Err = ConfigParseError;

    fn from_str(cookie_str: &str) -> Result<Self, Self::Err> {
        let mut csrf_token = None;
        let mut leet_code_token = None;

        for part in cookie_str.split(';').map(str::trim) {
            if let Some((k, v)) = part.split_once('=') {
                let key = k.trim();
                match key {
                    "csrftoken" => {
                        csrf_token = Some(v.trim().to_string());
                    }
                    "LEETCODE_SESSION" => {
                        leet_code_token = Some(v.trim().to_string());
                    }
                    _ => {}
                }
            }
        }

        let csrf_token = csrf_token.ok_or(ConfigParseError::CookieParseError("csrftoken"))?;
        let leet_code_token =
            leet_code_token.ok_or(ConfigParseError::CookieParseError("LEETCODE_SESSION"))?;

        Ok(Self {
            csrf_token,
            leet_code_token,
        })
    }
}

impl fmt::Display for LeetCodeCookies {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "csrftoken={}; LEETCODE_SESSION={}",
            self.csrf_token, self.leet_code_token
        )
    }
}

impl Config {
    pub async fn load_from_file(path: &str) -> Result<Config, ConfigParseError> {
        let content = tokio::fs::read_to_string(path).await?;
        let config_file: ConfigFile = serde_json::from_str(&content)?;

        Config::try_from(config_file)
    }
}

impl Default for AnkiModelConfig {
    fn default() -> Self {
        Self {
            id: 1307111927,
            name: String::from("LeetCode"),
        }
    }
}

impl Default for AnkiDeckConfig {
    fn default() -> Self {
        Self {
            id: 2084543157,
            name: String::from("LeetCode"),
        }
    }
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigParseError;

    fn try_from(config_file: ConfigFile) -> Result<Self, Self::Error> {
        Ok(Self {
            anki: config_file.anki,
            rate_limit: config_file.rate_limit,
            web_driver: config_file.web_driver,
            cookie: LeetCodeCookies::from_str(&config_file.cookie)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_leetcode_cookies_from_str() {
        // Test valid cookie string
        let cookie_str = "csrftoken=abc123; LEETCODE_SESSION=xyz789";
        let cookies = LeetCodeCookies::from_str(cookie_str).unwrap();
        assert_eq!(cookies.csrf_token, "abc123");
        assert_eq!(cookies.leet_code_token, "xyz789");
        assert_eq!(cookies.to_string(), cookie_str);
    }

    #[test]
    fn test_parse_leetcode_cookies_without_csrf_token() {
        let cookie_str = "LEETCODE_SESSION=xyz789";
        let result = LeetCodeCookies::from_str(cookie_str);
        assert!(result.is_err());

        if let Err(ConfigParseError::CookieParseError(field)) = result {
            assert_eq!(field, "csrftoken");
        }
    }

    #[test]
    fn test_parse_leetcode_cookies_without_leet_code_session() {
        let cookie_str = "csrftoken=abc123; INVALID_COOKIE=xyz789";
        let result = LeetCodeCookies::from_str(cookie_str);
        assert!(result.is_err());

        if let Err(ConfigParseError::CookieParseError(field)) = result {
            assert_eq!(field, "LEETCODE_SESSION");
        }
    }

    #[test]
    fn test_parse_leetcode_without_any_required_fieds() {
        let cookie_str = "INVALID_COOKIE=xyz789";
        let result = LeetCodeCookies::from_str(cookie_str);
        assert!(result.is_err());

        if let Err(ConfigParseError::CookieParseError(field)) = result {
            assert_eq!(field, "csrftoken");
        }
    }
}
