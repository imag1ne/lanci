use lanci::anki::{set_up_comrak_syntect_adapter, AnkiDeckManager};
use lanci::cli::{self, Cli};
use lanci::config::Config;
use lanci::crawler::leetcode::{extract_slug_from_url, LeetCodeCrawler};
use lanci::markdown::{save_markdown_to_file, ToMarkdown};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let cli_args = cli::parse_args();

    init_tracing_subscriber();
    if let Err(e) = run(&cli_args).await {
        error!("{}", e);
        std::process::exit(1);
    }
}

async fn run(cli_args: &Cli) -> anyhow::Result<()> {
    let config_path = cli_args
        .config
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid config file path"))?;
    let config = Config::load_from_file(config_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load config file: {}", e))?;

    let crawler =
        LeetCodeCrawler::new(config.rate_limit, &config.web_driver, &config.cookie).await?;
    let slug = extract_slug_from_url(&cli_args.url)?;

    info!("Crawling problem with slug: {}", slug);
    let crawl_result = crawler.crawl_problem(slug).await;
    crawler.close().await?;
    let problem = crawl_result?;

    let markdown = problem.to_markdown()?;
    let md_filename = cli_args.output_dir.join(format!("{}.md", problem.name));

    tokio::fs::create_dir_all(&cli_args.output_dir).await?;
    save_markdown_to_file(md_filename, &markdown).await?;
    info!("Problem saved to markdown successfully.");

    info!("Creating Anki deck for problem: {}", problem.name);
    // Load syntax highlighting theme
    let syntect_adapter = set_up_comrak_syntect_adapter()?;
    let mut deck = AnkiDeckManager::new(&config.anki, &syntect_adapter)?;
    deck.add_problem(&problem)?;

    let deck_filename = cli_args.output_dir.join(format!("{}.apkg", problem.name));
    deck.write_to_file(deck_filename)?;

    Ok(())
}

fn init_tracing_subscriber() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
