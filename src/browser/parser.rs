use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

pub struct HtmlParser;

impl HtmlParser {
    pub fn parse(html: &str) -> ParsedDocument {
        let document = Html::parse_document(html);

        let title = Self::extract_title(&document);
        let meta = Self::extract_meta(&document);
        let links = Self::extract_links(&document);
        let images = Self::extract_images(&document);
        let scripts = Self::extract_scripts(&document);
        let styles = Self::extract_styles(&document);
        let main_content = Self::extract_main_content(&document);
        let forms = Self::extract_forms(&document);

        ParsedDocument {
            title,
            meta,
            links,
            images,
            scripts,
            styles,
            main_content,
            forms,
            raw_html: html.to_string(),
        }
    }

    fn extract_title(document: &Html) -> String {
        let selector = Selector::parse("title").ok();
        if let Some(sel) = selector {
            if let Some(element) = document.select(&sel).next() {
                return element.text().collect::<String>().trim().to_string();
            }
        }

        let selector = Selector::parse("meta[property='og:title']").ok();
        if let Some(sel) = selector {
            if let Some(element) = document.select(&sel).next() {
                if let Some(content) = element.value().attr("content") {
                    return content.to_string();
                }
            }
        }

        String::new()
    }

    fn extract_meta(document: &Html) -> HashMap<String, String> {
        let mut meta = HashMap::new();

        let selector = Selector::parse("meta").ok();
        if let Some(sel) = selector {
            for element in document.select(&sel) {
                let name = element
                    .value()
                    .attr("name")
                    .or_else(|| element.value().attr("property"))
                    .unwrap_or("");
                let content = element.value().attr("content").unwrap_or("");

                if !name.is_empty() && !content.is_empty() {
                    meta.insert(name.to_string(), content.to_string());
                }
            }
        }

        meta
    }

    fn extract_links(document: &Html) -> Vec<Link> {
        let mut links = Vec::new();

        let selector = Selector::parse("a").ok();
        if let Some(sel) = selector {
            for element in document.select(&sel) {
                let href = element.value().attr("href").unwrap_or("");
                let text = element.text().collect::<String>().trim().to_string();
                let title = element.value().attr("title").map(String::from);

                if !href.is_empty() {
                    links.push(Link {
                        href: href.to_string(),
                        text,
                        title,
                    });
                }
            }
        }

        links
    }

    fn extract_images(document: &Html) -> Vec<Image> {
        let mut images = Vec::new();

        let selector = Selector::parse("img").ok();
        if let Some(sel) = selector {
            for element in document.select(&sel) {
                let src = element.value().attr("src").unwrap_or("");
                let alt = element.value().attr("alt").unwrap_or("").to_string();
                let title = element.value().attr("title").map(String::from);

                if !src.is_empty() {
                    images.push(Image {
                        src: src.to_string(),
                        alt,
                        title,
                    });
                }
            }
        }

        images
    }

    fn extract_scripts(document: &Html) -> Vec<String> {
        let selector = Selector::parse("script").ok();
        if let Some(sel) = selector {
            document
                .select(&sel)
                .filter_map(|e| e.value().attr("src"))
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn extract_styles(document: &Html) -> Vec<String> {
        let selector = Selector::parse("style").ok();
        if let Some(sel) = selector {
            document
                .select(&sel)
                .flat_map(|e| e.text().collect::<Vec<_>>())
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn extract_main_content(document: &Html) -> String {
        let candidates = [
            "main",
            "article",
            "[role='main']",
            ".post-content",
            ".article-content",
            ".entry-content",
            ".content",
            "#content",
        ];

        for selector_str in candidates {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    return element.html();
                }
            }
        }

        let selector = Selector::parse("body").ok();
        if let Some(sel) = selector {
            if let Some(element) = document.select(&sel).next() {
                return element.html();
            }
        }

        String::new()
    }

    pub fn extract_text(html: &str) -> String {
        let document = Html::parse_fragment(html);
        let mut text = String::new();

        fn extract_from_element(element: &ElementRef, text: &mut String) {
            for child in element.children() {
                if let Some(el) = child.value().as_text() {
                    let content = el.trim();
                    if !content.is_empty() {
                        text.push_str(content);
                        text.push(' ');
                    }
                } else if child.value().is_element() {
                    let el = child.value().as_element().unwrap();
                    let tag_name = el.name();
                    match tag_name {
                        "script" | "style" | "noscript" => continue,
                        "p" | "div" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li"
                        | "blockquote" => {
                            text.push('\n');
                            extract_from_element(&scraper::ElementRef::wrap(child).unwrap(), text);
                            text.push('\n');
                        }
                        "hr" => {
                            text.push_str("\n---\n");
                        }
                        _ => {
                            extract_from_element(&scraper::ElementRef::wrap(child).unwrap(), text);
                        }
                    }
                }
            }
        }

        if let Some(root) = document.root_element().into() {
            extract_from_element(&root, &mut text);
        }

        text.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn extract_links_with_text(html: &str) -> Vec<(String, String)> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a").ok();

        if let Some(sel) = selector {
            document
                .select(&sel)
                .filter_map(|el| {
                    let href = el.value().attr("href")?;
                    if href.starts_with('#')
                        || href.starts_with("javascript:")
                        || href.starts_with("mailto:")
                    {
                        return None;
                    }
                    let text = el.text().collect::<String>().trim().to_string();
                    if text.is_empty() {
                        let title = el.value().attr("title").map(|s| s.trim().to_string());
                        if let Some(title) = title {
                            if !title.is_empty() {
                                return Some((href.to_string(), title));
                            }
                        }
                        None
                    } else {
                        Some((href.to_string(), text))
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn extract_all_links(html: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a").ok();

        if let Some(sel) = selector {
            document
                .select(&sel)
                .filter_map(|el| {
                    let href = el.value().attr("href")?;
                    if href.starts_with('#')
                        || href.starts_with("javascript:")
                        || href.starts_with("mailto:")
                    {
                        return None;
                    }
                    Some(href.to_string())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn resolve_url(base: &str, relative: &str) -> String {
        if relative.starts_with("http://") || relative.starts_with("https://") {
            return relative.to_string();
        }

        if relative.starts_with('/') {
            if let Ok(base_url) = url::Url::parse(base) {
                if let Some(host) = base_url.host_str() {
                    let scheme = base_url.scheme();
                    return format!("{}://{}{}", scheme, host, relative);
                }
            }
        }

        if let Ok(base_url) = url::Url::parse(base) {
            if let Ok(resolved) = base_url.join(relative) {
                return resolved.to_string();
            }
        }

        relative.to_string()
    }

    pub fn extract_forms(document: &Html) -> Vec<Form> {
        let selector = Selector::parse("form").ok();

        if let Some(sel) = selector {
            document
                .select(&sel)
                .map(|form_el| {
                    let action = form_el.value().attr("action").unwrap_or("").to_string();
                    let method = form_el
                        .value()
                        .attr("method")
                        .unwrap_or("get")
                        .to_lowercase();

                    let mut inputs = Vec::new();
                    let input_sel = Selector::parse("input").ok();
                    if let Some(is) = input_sel {
                        for input in form_el.select(&is) {
                            let name = input.value().attr("name").unwrap_or("").to_string();
                            let input_type =
                                input.value().attr("type").unwrap_or("text").to_lowercase();
                            let value = input.value().attr("value").unwrap_or("").to_string();
                            let placeholder =
                                input.value().attr("placeholder").unwrap_or("").to_string();

                            if !name.is_empty()
                                && input_type != "submit"
                                && input_type != "button"
                                && input_type != "hidden"
                            {
                                inputs.push(FormInput {
                                    name,
                                    input_type,
                                    value,
                                    placeholder,
                                });
                            }
                        }
                    }

                    Form {
                        action,
                        method,
                        inputs,
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub meta: HashMap<String, String>,
    pub links: Vec<Link>,
    pub images: Vec<Image>,
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
    pub main_content: String,
    pub forms: Vec<Form>,
    pub raw_html: String,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub src: String,
    pub alt: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub inputs: Vec<FormInput>,
}

#[derive(Debug, Clone)]
pub struct FormInput {
    pub name: String,
    pub input_type: String,
    pub value: String,
    pub placeholder: String,
}
