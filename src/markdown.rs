use std::io;
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// Trait for converting types to Markdown format
pub trait ToMarkdown {
    type Err;

    fn to_markdown(&self) -> Result<String, Self::Err>;
}

/// Represents a Markdown code block with a specified language and code content
#[derive(Debug)]
pub struct MarkdownCodeBlock {
    pub language: String,
    pub code: String,
}

impl ToMarkdown for MarkdownCodeBlock {
    type Err = ();

    fn to_markdown(&self) -> Result<String, Self::Err> {
        let language = match self.language.to_lowercase().as_str() {
            "python3" | "pythondata" => "python",
            "postgresql" | "mysql" | "mssql" | "oraclesql" => "sql",
            _ => &self.language,
        };
        Ok(format!("```{}\n{}\n```", language, self.code))
    }
}

pub async fn save_markdown_to_file(filename: impl AsRef<Path>, markdown: &str) -> io::Result<()> {
    let mut file = tokio::fs::File::create(filename).await?;
    file.write_all(markdown.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}
