pub mod question;
pub mod solution;
pub mod submission;

use crate::config::{LeetCodeCookies, WebDriverConfig};
use crate::markdown::{MarkdownCodeBlock, ToMarkdown};
use governor::{DefaultDirectRateLimiter, Jitter, Quota, RateLimiter};
use question::{QuestionDetail, QuestionObj};
use submission::SubmissionMeta;

use fantoccini::cookies::Cookie;
use reqwest::header::{HeaderMap, COOKIE, REFERER};
use serde_json::json;
use submission::SubmissionObj;
use url::Url;

use super::retry;
use crate::error::{CrawlerError, DocumentConversionError};
use fantoccini::error::CmdError;
pub use question::QuestionDescription;
use std::fmt::Write;
use std::num::NonZeroU32;
use std::time::Duration;
use tracing::{debug, info};

pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.98 Safari/537.36";
pub const LEET_CODE_HOST: &str = "https://leetcode.com";
pub const LEET_CODE_API: &str = "https://leetcode.com/graphql";

/// Represents a LeetCode problem with its name, description and accepted submissions.
#[derive(Debug)]
pub struct LeetCodeProblem {
    pub name: String,
    pub description: QuestionDetail,
    pub submissions: Vec<MarkdownCodeBlock>,
}

impl ToMarkdown for LeetCodeProblem {
    type Err = DocumentConversionError;

    fn to_markdown(&self) -> Result<String, Self::Err> {
        let mut markdown = String::with_capacity(1024);

        write!(
            markdown,
            "# Description\n\n{}",
            self.description.to_markdown()?
        )?;

        if !self.submissions.is_empty() {
            write!(markdown, "\n\n# Solution")?;
            for (i, submission) in self.submissions.iter().enumerate() {
                write!(
                    markdown,
                    "\n\n{}. \n\n{}",
                    i + 1,
                    submission.to_markdown().unwrap()
                )?;
            }
        }

        Ok(markdown)
    }
}

/// A crawler for LeetCode problems. Some methods require a web driver to fetch dynamic content, while others use the LeetCode GraphQL API for static data retrieval.
pub struct LeetCodeCrawler {
    web_driver: fantoccini::Client,
    client: reqwest::Client,
    rate_limiter: DefaultDirectRateLimiter,
    jitter: Jitter,
}

impl LeetCodeCrawler {
    /// Creates a new `LeetCodeCrawler` instance with the provided web driver endpoint, headless mode, and cookies.
    pub async fn new(
        rate_limit: u32,
        web_driver_config: &WebDriverConfig,
        cookie: &LeetCodeCookies,
    ) -> Result<Self, CrawlerError> {
        // Set up reqwest client
        let client = Self::new_reqwest_client(cookie)?;
        // Set up fantoccini web driver client
        let web_driver = Self::new_web_driver_client(
            &web_driver_config.endpoint,
            web_driver_config.headless,
            cookie,
        )
        .await?;
        // Create a rate limiter for the crawler
        let quota =
            Quota::per_second(NonZeroU32::new(rate_limit).ok_or(CrawlerError::ZeroRateLimit)?);
        let rate_limiter = RateLimiter::direct(quota);
        let jitter = Jitter::new(Duration::from_millis(200), Duration::from_millis(500));

        Ok(Self {
            web_driver,
            client,
            rate_limiter,
            jitter,
        })
    }

    /// Creates a new reqwest client with the provided cookies and set headers(referer and user-agent).
    fn new_reqwest_client(cookie: &LeetCodeCookies) -> Result<reqwest::Client, CrawlerError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            LEET_CODE_HOST
                .parse()
                .map_err(|e| CrawlerError::Other(format!("referer parse error : {}", e)))?,
        );
        headers.insert(
            COOKIE,
            cookie
                .to_string()
                .parse()
                .map_err(|e| CrawlerError::Other(format!("cookie parse error: {}", e)))?,
        );

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .map_err(CrawlerError::BuildReqwestClientError)?;

        Ok(client)
    }

    /// Creates a new fantoccini web driver client with the provided endpoint, headless mode, and cookies.
    async fn new_web_driver_client(
        web_driver_endpoint: &Url,
        headless: bool,
        cookie: &LeetCodeCookies,
    ) -> Result<fantoccini::Client, CrawlerError> {
        let mut web_driver_builder = fantoccini::ClientBuilder::native();

        if headless {
            let mut caps = serde_json::map::Map::new();
            let opts = json!({ "args": ["-headless"] });
            caps.insert("moz:firefoxOptions".to_string(), opts);

            web_driver_builder.capabilities(caps);
        }

        let web_driver = web_driver_builder
            .connect(web_driver_endpoint.as_str())
            .await?;

        if let Err(e) = set_up_web_driver(&web_driver, cookie).await {
            web_driver.close().await?;
            return Err(e.into());
        }

        Ok(web_driver)
    }

    /// Crawls a LeetCode problem by its slug and returns a `LeetCodeProblem` struct.
    /// It fetches the problem description, solution, and accepted submissions based on the provided `CrawlConfig`.
    pub async fn crawl_problem(&self, slug: &str) -> Result<LeetCodeProblem, CrawlerError> {
        let (question_detail, submissions) = tokio::try_join!(
            self.fetch_problem_detail(slug),
            self.fetch_accepted_submissions(slug)
        )?;
        let name = format!(
            "{}. {}",
            question_detail.question_id, question_detail.question_title
        );

        let problem = LeetCodeProblem {
            name,
            description: question_detail,
            submissions,
        };

        Ok(problem)
    }

    /// Fetches the problem details (description and tags...) from LeetCode by its slug.
    pub async fn fetch_problem_detail(&self, slug: &str) -> Result<QuestionDetail, CrawlerError> {
        info!("Fetching problem detail for slug: {}", slug);

        let question_obj: QuestionObj = self.post_graphql(
            r#"query getQuestionDetail($titleSlug:String!){question(titleSlug:$titleSlug){questionId questionFrontendId questionTitle questionTitleSlug content difficulty stats similarQuestions categoryTitle topicTags{name slug}}}"#,
            json!({ "titleSlug": slug }),
        ).await?;

        Ok(question_obj.data.question)
    }

    /// Fetches all accepted submissions for a given problem slug and returns a vec of `MarkdownCodeBlock`.
    pub async fn fetch_accepted_submissions(
        &self,
        slug: &str,
    ) -> Result<Vec<MarkdownCodeBlock>, CrawlerError> {
        info!("Fetching accepted submissions for slug: {}", slug);

        let submission_metas = self.fetch_submission_metas(slug).await?;
        let mut code_blocks = Vec::with_capacity(submission_metas.len());

        for meta in submission_metas.iter().filter(|meta| meta.is_accepted()) {
            let code_block = self.fetch_submission(meta).await?;
            code_blocks.push(code_block);
        }

        info!(
            "Found {} accepted submissions for slug: {}",
            code_blocks.len(),
            slug
        );

        Ok(code_blocks)
    }

    /// Fetches the submission metadata for a given problem slug, it returns a vector of `SubmissionMeta`.
    async fn fetch_submission_metas(
        &self,
        slug: &str,
    ) -> Result<Vec<SubmissionMeta>, CrawlerError> {
        debug!("Fetching submission metadata for slug: {}", slug);
        let submission_obj: SubmissionObj = self.post_graphql(
            r#"query Submissions($offset:Int! $limit:Int! $lastKey:String $questionSlug:String!){submissionList(offset:$offset limit:$limit lastKey:$lastKey questionSlug:$questionSlug){submissions{id statusDisplay lang runtime timestamp url isPending __typename}__typename}}"#,
            json!({ "offset": 0, "limit": 20, "lastKey": "", "questionSlug": slug }),
        )
        .await?;

        Ok(submission_obj.data.submission_list.submissions)
    }

    /// Fetches the submitted code for a given submission metadata. It returns a `MarkdownCodeBlock` containing the language and code.
    async fn fetch_submission(
        &self,
        submission_meta: &SubmissionMeta,
    ) -> Result<MarkdownCodeBlock, CrawlerError> {
        let host = Url::parse(LEET_CODE_HOST)?;
        let url = host.join(&submission_meta.url)?;

        let code_text = retry(3, || async {
            self.fetch_submitted_code(url.as_str()).await
        })
        .await?;

        let code_block = MarkdownCodeBlock {
            language: submission_meta.lang.clone(),
            code: code_text,
        };

        Ok(code_block)
    }

    /// Fetches the submitted code for a given submission URL. The code is extracted from the page's JavaScript variable `pageData.submissionCode`.
    async fn fetch_submitted_code(&self, url: &str) -> Result<String, CrawlerError> {
        self.rate_limiter.until_ready_with_jitter(self.jitter).await;
        debug!("Fetching submitted code from URL: {}", url);
        self.web_driver.goto(url).await?;

        let js_script = "return pageData.submissionCode;";
        let code_data = self.web_driver.execute(js_script, vec![]).await?;

        let code_text = code_data
            .as_str()
            .ok_or(CrawlerError::EmptyResult("pageData.submissionCode"))?
            .trim();

        Ok(code_text.into())
    }

    /// Sends a GraphQL POST request to the LeetCode API with the provided query and variables.
    async fn post_graphql<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, CrawlerError> {
        self.rate_limiter.until_ready_with_jitter(self.jitter).await;

        let parameters = json!({
            "query": query,
            "variables": variables,
        });

        debug!("Sending GraphQL request: {}", parameters);

        let resp = self
            .client
            .post(LEET_CODE_API)
            .json(&parameters)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;

        Ok(resp)
    }

    /// Closes the web driver client session.
    pub async fn close(self) -> Result<(), CrawlerError> {
        self.web_driver.close().await?;
        Ok(())
    }
}

/// Extracts the problem slug from a given LeetCode URL.
/// The slug should be the first path segment after `/problems/`.
pub fn extract_slug_from_url(url: &Url) -> Result<&str, CrawlerError> {
    let mut path_segments = url
        .path_segments()
        .ok_or_else(|| CrawlerError::SlugParseError(url.to_string()))?;

    path_segments.find(|&s| s == "problems");

    let slug = path_segments
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CrawlerError::SlugParseError(url.to_string()))?;

    Ok(slug)
}

/// Sets up the web driver with the necessary cookies(csrftoken and LEETCODE_SESSION) and user agent.
async fn set_up_web_driver(
    web_driver: &fantoccini::Client,
    cookie: &LeetCodeCookies,
) -> Result<(), CmdError> {
    // Set up user agent
    web_driver.set_ua(USER_AGENT).await?;

    // Set up cookies
    web_driver.goto(LEET_CODE_HOST).await?;
    web_driver
        .add_cookie(Cookie::new("csrftoken", cookie.csrf_token.clone()))
        .await?;
    web_driver
        .add_cookie(Cookie::new(
            "LEETCODE_SESSION",
            cookie.leet_code_token.clone(),
        ))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_slug_from_url() {
        let url = Url::parse("https://leetcode.com/problems/two-sum/").unwrap();
        let slug = extract_slug_from_url(&url).unwrap();
        assert_eq!(slug, "two-sum");
    }

    #[test]
    fn test_extract_slug_from_url_without_path_segments() {
        let url = Url::parse("https://leetcode.com/").unwrap();
        assert!(extract_slug_from_url(&url).is_err());
    }

    #[test]
    fn test_extract_slug_from_url_without_slug() {
        let url = Url::parse("https://leetcode.com/problems/").unwrap();
        assert!(extract_slug_from_url(&url).is_err());

        let url = Url::parse("https://leetcode.com/whatever/slug").unwrap();
        assert!(extract_slug_from_url(&url).is_err());
    }
}
