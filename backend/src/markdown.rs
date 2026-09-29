//! Rendu Markdown → HTML assaini, avec ancres sur les titres et temps de lecture.

use std::sync::LazyLock;

use ammonia::Builder;
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};

const WORDS_PER_MINUTE: usize = 200;

static SANITIZER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let mut builder = Builder::default();
    builder
        .add_tag_attributes("code", &["class"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .add_tag_attributes("img", &["loading"])
        .id_prefix(None);
    builder
});

pub fn render(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut events: Vec<Event> = Parser::new_ext(markdown, options).collect();

    // Ajoute un identifiant (slug du texte) aux titres qui n'en ont pas.
    for i in 0..events.len() {
        if let Event::Start(Tag::Heading { id: None, .. }) = &events[i] {
            let mut text = String::new();
            for ev in &events[i + 1..] {
                match ev {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(t) | Event::Code(t) => text.push_str(t),
                    _ => {}
                }
            }
            if let Event::Start(Tag::Heading { id, .. }) = &mut events[i] {
                *id = Some(CowStr::from(slug::slugify(&text)));
            }
        }
    }

    let mut out = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(&mut out, events.into_iter());
    SANITIZER.clean(&out).to_string()
}

pub fn reading_minutes(markdown: &str) -> i32 {
    let words = markdown.split_whitespace().count();
    words.div_ceil(WORDS_PER_MINUTE).max(1) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_get_ids_and_scripts_are_removed() {
        let html = render("## Signals et Zoneless\n\n<script>alert(1)</script>\n\n```ts\nconst a = 1;\n```");
        assert!(html.contains(r#"<h2 id="signals-et-zoneless">"#));
        assert!(!html.contains("<script"));
        assert!(html.contains(r#"<code class="language-ts">"#));
    }

    #[test]
    fn reading_time_is_at_least_one_minute() {
        assert_eq!(reading_minutes("court"), 1);
        assert_eq!(reading_minutes(&"mot ".repeat(401)), 3);
    }
}
