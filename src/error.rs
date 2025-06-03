use std::{fmt, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentConversionError {
    #[error(transparent)]
    FormatError(#[from] fmt::Error),

    #[error("Failed to rewrite HTML document: {0}")]
    HtmlRewriteError(#[from] lol_html::errors::RewritingError),
}

#[derive(Error, Debug)]
pub enum ConfigParseError {
    #[error("Failed to read configuration file: {0}")]
    ReadFileError(#[from] io::Error),

    #[error("Failed to parse configuration file: {0}")]
    DeserializeError(#[from] serde_json::Error),

    #[error("Missing required field in cookies: {0}")]
    CookieParseError(&'static str),
}

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("Rate limit can not be zero")]
    ZeroRateLimit,

    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Failed to get slug from URL: {0}")]
    SlugParseError(String),

    #[error("Failed to build reqwest client: {0}")]
    BuildReqwestClientError(reqwest::Error),

    #[error("Reqwest error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to build web driver client: {0}")]
    BuildWebDriverClientError(#[from] fantoccini::error::NewSessionError),

    #[error("Failed to execute web driver command: {0}")]
    WebDriverCommandError(#[from] fantoccini::error::CmdError),

    #[error("Unexpected empty result when fetching `{0}`")]
    EmptyResult(&'static str),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum AnkiError {
    #[error("Failed to load syntax highlighting theme: {0}")]
    LoadThemeError(#[from] syntect::LoadingError),
    #[error("Failed to create Anki note(Flashcard): {0}")]
    CreateNoteError(Box<genanki_rs::Error>),
    #[error("Failed to write Anki deck to file: {0}")]
    WriteDeckError(Box<genanki_rs::Error>),
    #[error("Deck filename is not valid UTF-8")]
    InvalidDeckFilename,
}
