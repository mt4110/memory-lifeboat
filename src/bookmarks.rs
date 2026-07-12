use anyhow::{Result, anyhow};
use scraper::{Html, Selector};
use url::Url;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

pub fn parse_bookmarks(html: &str) -> Result<Vec<Bookmark>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a")
        .map_err(|error| anyhow!("failed to build bookmark selector: {error:?}"))?;
    let mut bookmarks = Vec::new();

    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let Ok(url) = Url::parse(href) else {
            continue;
        };
        let title = element.text().collect::<Vec<_>>().join(" ");
        bookmarks.push(Bookmark {
            title: title.split_whitespace().collect::<Vec<_>>().join(" "),
            url: url.into(),
        });
    }

    bookmarks.sort_by(|a, b| a.url.cmp(&b.url).then(a.title.cmp(&b.title)));
    bookmarks.dedup_by(|a, b| a.url == b.url && a.title == b.title);
    Ok(bookmarks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_netscape_bookmarks() {
        let html = r#"<DL><p><DT><A HREF="https://example.com">Example</A></DL>"#;
        let parsed = parse_bookmarks(html).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].title, "Example");
        assert_eq!(parsed[0].url, "https://example.com/");
    }
}
