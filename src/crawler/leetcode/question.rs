use crate::markdown::ToMarkdown;
use std::fmt;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
pub struct QuestionObj {
    pub data: QuestionData,
}

#[derive(Debug, Deserialize)]
pub struct QuestionData {
    pub question: QuestionDetail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDetail {
    pub question_frontend_id: String,
    pub question_title: String,
    pub question_title_slug: String,
    pub content: String,
    pub difficulty: QuestionDifficulty,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub topic_tags: Vec<TopicTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QuestionDifficulty {
    Easy,
    Medium,
    Hard,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicTag {
    pub name: String,
    pub slug: String,
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

impl ToMarkdown for QuestionDetail {
    type Err = lol_html::errors::RewritingError;

    fn to_markdown(&self) -> Result<String, Self::Err> {
        let description = QuestionDescription::from(&self.content);
        description.to_markdown()
    }
}

pub struct QuestionDescription<'a>(&'a str);

impl<'a> From<&'a str> for QuestionDescription<'a> {
    fn from(description: &'a str) -> Self {
        QuestionDescription(description)
    }
}

impl<'a> From<&'a String> for QuestionDescription<'a> {
    fn from(s: &'a String) -> Self {
        QuestionDescription::from(s.as_str())
    }
}

impl<'a> ToMarkdown for QuestionDescription<'a> {
    type Err = lol_html::errors::RewritingError;

    fn to_markdown(&self) -> Result<String, Self::Err> {
        // Process the HTML content.
        // Here we do two things:
        // 1. Convert <sup> tags to '^'.
        // 2. Convert <sub> tags to '_'.
        // For example, if the HTML contains: <code>n<sup>th</sup></code>
        // it will be converted to: <code>n^th</code>
        let element_content_handlers = vec![
            // <sup> tags -> '^'
            element!("sup", |el| {
                el.prepend("^", ContentType::Text);
                el.remove_and_keep_content();

                Ok(())
            }),
            // <sub> tags -> '_'
            element!("sub", |el| {
                el.prepend("_", ContentType::Text);
                el.remove_and_keep_content();

                Ok(())
            }),
        ];

        let processed = rewrite_str(
            self.0,
            RewriteStrSettings {
                element_content_handlers,
                ..RewriteStrSettings::new()
            },
        )?;

        // Convert the processed HTML to Markdown.
        let md = html2md::parse_html(&processed);

        Ok(md)
    }
}

impl fmt::Display for QuestionDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuestionDifficulty::Easy => write!(f, "Easy"),
            QuestionDifficulty::Medium => write!(f, "Medium"),
            QuestionDifficulty::Hard => write!(f, "Hard"),
            QuestionDifficulty::Unknown => write!(f, "Unknown"),
        }
    }
}
