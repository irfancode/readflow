use crate::Article;
use std::path::Path;
use tracing::info;

pub struct Exporter;

impl Exporter {
    pub fn export_markdown(article: &Article, path: &Path) -> Result<(), std::io::Error> {
        let mut content = String::new();

        content.push_str(&format!("# {}\n\n", article.title));

        if let Some(author) = &article.author {
            content.push_str(&format!("**Author:** {}\n\n", author));
        }

        if let Some(date) = &article.published_at {
            content.push_str(&format!("**Published:** {}\n\n", date.format("%Y-%m-%d")));
        }

        content.push_str(&format!(
            "**Reading time:** {} min\n\n",
            article.reading_time
        ));
        content.push_str(&format!("**Source:** {}\n\n", article.url));
        content.push_str("---\n\n");

        content.push_str(&article.content);

        std::fs::write(path, content)?;
        info!("Exported article to markdown: {:?}", path);

        Ok(())
    }

    pub fn export_html(article: &Article, path: &Path) -> Result<(), std::io::Error> {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            line-height: 1.6;
            color: #333;
        }}
        h1 {{ color: #111; }}
        .meta {{ color: #666; font-size: 0.9rem; }}
        .content {{ margin-top: 2rem; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="meta">
        {}
    </div>
    <div class="content">
        {}
    </div>
</body>
</html>"#,
            article.title,
            article.title,
            article.author.as_deref().unwrap_or("Unknown"),
            article
                .content
                .lines()
                .map(|l| format!("<p>{}</p>", l))
                .collect::<Vec<_>>()
                .join("\n")
        );

        std::fs::write(path, html)?;
        info!("Exported article to HTML: {:?}", path);

        Ok(())
    }

    pub fn export_text(article: &Article, path: &Path) -> Result<(), std::io::Error> {
        let mut content = String::new();

        content.push_str(&article.title);
        content.push_str("\n");
        content.push_str(&"=".repeat(article.title.len()));
        content.push_str("\n\n");

        if let Some(author) = &article.author {
            content.push_str(&format!("Author: {}\n", author));
        }

        if let Some(date) = &article.published_at {
            content.push_str(&format!("Published: {}\n", date.format("%Y-%m-%d")));
        }

        content.push_str(&format!("Reading time: {} min\n", article.reading_time));
        content.push_str(&format!("Source: {}\n", article.url));
        content.push_str("\n");
        content.push_str(&article.content);

        std::fs::write(path, content)?;
        info!("Exported article to text: {:?}", path);

        Ok(())
    }

    pub fn export_article(
        article: &Article,
        path: &Path,
        format: &str,
    ) -> Result<(), std::io::Error> {
        match format.to_lowercase().as_str() {
            "md" | "markdown" => Self::export_markdown(article, path),
            "html" => Self::export_html(article, path),
            "txt" | "text" => Self::export_text(article, path),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown format: {}", format),
            )),
        }
    }
}
