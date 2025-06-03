use crate::config::AnkiConfig;
use crate::crawler::leetcode::LeetCodeProblem;
use crate::error::AnkiError;
use crate::markdown::ToMarkdown;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins, Options, Plugins};
use genanki_rs::{Deck, Field, Model, Note, Template};
use std::io::Cursor;
use std::path::Path;
use syntect::highlighting::ThemeSet;

pub struct AnkiDeckManager<'c, 'p> {
    pub deck: Deck,
    pub model: Model,
    pub comrak_options: Options<'c>,
    pub comrak_plugins: Plugins<'p>,
    pub syntect_adapter: &'p SyntectAdapter,
}

impl<'c, 'p> AnkiDeckManager<'c, 'p> {
    pub fn new(
        anki_config: &AnkiConfig,
        syntect_adapter: &'p SyntectAdapter,
    ) -> Result<Self, AnkiError> {
        let front_template = include_str!("../assets/templates/front.html");
        let back_template = include_str!("../assets/templates/back.html");
        let css = include_str!("../assets/templates/style.css");

        let model = new_model(
            anki_config.model.id,
            &anki_config.model.name,
            "LeetCode",
            front_template,
            back_template,
            css,
        );

        let comrak_plugins = set_up_comrak_plugins(syntect_adapter);
        let comrak_options = Options::default();

        let deck = Deck::new(
            anki_config.deck.id,
            &anki_config.deck.name,
            "LeetCode problems deck",
        );

        Ok(AnkiDeckManager {
            deck,
            model,
            comrak_options,
            comrak_plugins,
            syntect_adapter,
        })
    }

    pub fn add_problem(&mut self, problem: &LeetCodeProblem) -> Result<(), AnkiError> {
        let note = new_note(
            self.model.clone(),
            problem,
            &self.comrak_options,
            &self.comrak_plugins,
        )
        .map_err(AnkiError::CreateNoteError)?;

        self.deck.add_note(note);

        Ok(())
    }

    pub fn write_to_file(&self, filename: impl AsRef<Path>) -> Result<(), AnkiError> {
        let filename = filename
            .as_ref()
            .to_str()
            .ok_or_else(|| AnkiError::InvalidDeckFilename)?;

        self.deck
            .write_to_file(filename)
            .map_err(|e| AnkiError::WriteDeckError(Box::new(e)))?;

        Ok(())
    }
}

fn new_model(
    model_id: i64,
    model_name: &str,
    template_name: &str,
    front_template: &str,
    back_template: &str,
    css: &str,
) -> Model {
    Model::new(
        model_id,
        model_name,
        vec![
            Field::new("ID"),
            Field::new("Title"),
            Field::new("TitleSlug"),
            Field::new("Difficulty"),
            Field::new("Description"),
            Field::new("Tags"),
            Field::new("TagSlugs"),
            Field::new("Submission"),
            Field::new("Note"),
        ],
        vec![
            Template::new(template_name)
                .qfmt(front_template)
                .afmt(back_template),
        ],
    )
    .css(css)
}

fn new_note(
    model: Model,
    problem: &LeetCodeProblem,
    comrak_options: &ComrakOptions,
    comrak_plugins: &ComrakPlugins,
) -> Result<Note, Box<genanki_rs::Error>> {
    let tags_str = problem
        .description
        .topic_tags
        .iter()
        .map(|tag| tag.name.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let tag_slugs = problem
        .description
        .topic_tags
        .iter()
        .map(|tag| tag.slug.as_str())
        .collect::<Vec<_>>();
    let tag_slugs_str = tag_slugs.join(";");

    let code_htmls = problem
        .submissions
        .iter()
        .map(|code| {
            markdown_to_html_with_plugins(
                &code.to_markdown().unwrap(),
                comrak_options,
                comrak_plugins,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Note::new_with_options(
        model,
        vec![
            &problem.description.question_frontend_id,
            &problem.description.question_title,
            &problem.description.question_title_slug,
            &problem.description.difficulty.to_string(),
            &problem.description.content,
            &tags_str,
            &tag_slugs_str,
            &code_htmls,
            "",
        ],
        None,
        Some(tag_slugs),
        Some(&problem.description.question_frontend_id),
    )
    .map_err(Box::new)
}

pub fn set_up_comrak_syntect_adapter() -> Result<SyntectAdapter, AnkiError> {
    let theme_file = include_str!("../assets/themes/Dracula.tmTheme");
    let mut theme_reader = Cursor::new(theme_file);
    let mut theme_set = ThemeSet::new();
    let theme = ThemeSet::load_from_reader(&mut theme_reader)?;
    theme_set.themes.insert("Dracula".to_string(), theme);

    let syntect_adapter = SyntectAdapterBuilder::new()
        .theme("Dracula")
        .theme_set(theme_set)
        .build();

    Ok(syntect_adapter)
}

fn set_up_comrak_plugins(syntect_adapter: &SyntectAdapter) -> Plugins {
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(syntect_adapter);

    plugins
}
