use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct BuildPaths {
    pub config_path: PathBuf,
    pub content_dir: PathBuf,
    pub static_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteConfig {
    pub base_url: String,
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub language_code: String,
    pub description: String,
    pub favicon: String,
    pub images: Vec<String>,
    pub enable_post_navigator: bool,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost/".to_string(),
            title: "Site".to_string(),
            author: String::new(),
            copyright: String::new(),
            language_code: "en-US".to_string(),
            description: String::new(),
            favicon: String::new(),
            images: Vec::new(),
            enable_post_navigator: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageKind {
    Section,
    Page,
    Post,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub body: String,
    pub date: Option<String>,
    pub draft: bool,
    pub menu: Option<String>,
    pub weight: Option<i32>,
    pub url_path: String,
    kind: PageKind,
}

#[derive(Debug, Clone)]
pub struct Site {
    pub config: SiteConfig,
    pub pages: Vec<Page>,
    pub blog: Page,
    pub posts: Vec<Page>,
}

pub fn generate_site(paths: &BuildPaths) -> Result<()> {
    let site = load_site(paths)?;
    write_site(&site, paths)
}

pub fn load_site(paths: &BuildPaths) -> Result<Site> {
    let config = parse_site_config(&fs::read_to_string(&paths.config_path)?)?;
    let blog = read_page(
        &paths.content_dir.join("blog").join("_index.md"),
        PageKind::Section,
        "/".to_string(),
    )?;

    let mut pages = Vec::new();
    for entry in fs::read_dir(&paths.content_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.file_name() == Some("_index.md".as_ref()) {
            continue;
        }
        if path.extension() == Some("md".as_ref()) {
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("invalid content filename: {}", path.display()))?;
            pages.push(read_page(&path, PageKind::Page, format!("/{stem}/"))?);
        }
    }
    pages.sort_by_key(|page| (page.weight.unwrap_or(i32::MAX), page.title.clone()));

    let mut posts = Vec::new();
    for entry in fs::read_dir(paths.content_dir.join("blog"))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.file_name() == Some("_index.md".as_ref()) {
            continue;
        }
        if path.extension() == Some("md".as_ref()) {
            let page = read_page(&path, PageKind::Post, String::new())?;
            if !page.draft {
                let mut page = page;
                page.url_path = format!("/{}/", slugify(&page.title));
                posts.push(page);
            }
        }
    }
    posts.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| left.title.cmp(&right.title))
    });

    Ok(Site {
        config,
        pages,
        blog,
        posts,
    })
}

pub fn parse_site_config(raw: &str) -> Result<SiteConfig> {
    let mut config = SiteConfig::default();
    let mut section = String::new();

    for raw_line in raw.lines() {
        let line = strip_inline_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_matches(&['[', ']'][..]).to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = parse_toml_value(value.trim());

        match (section.as_str(), key, value) {
            ("", "baseURL", TomlValue::String(value)) => config.base_url = value,
            ("", "title", TomlValue::String(value)) => config.title = value,
            ("", "author", TomlValue::String(value)) => config.author = value,
            ("", "copyright", TomlValue::String(value)) => config.copyright = value,
            ("", "languageCode", TomlValue::String(value)) => config.language_code = value,
            ("params", "description", TomlValue::String(value)) => config.description = value,
            ("params", "favicon", TomlValue::String(value)) => config.favicon = value,
            ("params", "images", TomlValue::Array(value)) => config.images = value,
            ("params", "title", TomlValue::String(value)) => config.title = value,
            ("params", "enablePostNavigator", TomlValue::Bool(value)) => {
                config.enable_post_navigator = value;
            }
            _ => {}
        }
    }

    Ok(config)
}

pub fn parse_front_matter(raw: &str) -> Result<(BTreeMap<String, TomlValue>, String)> {
    let Some(rest) = raw.strip_prefix("+++\n") else {
        return Err("content file is missing TOML front matter".into());
    };
    let Some(end) = rest.find("\n+++") else {
        return Err("content file has unterminated TOML front matter".into());
    };
    let front_matter = &rest[..end];
    let body = rest[end + "\n+++".len()..]
        .strip_prefix('\n')
        .unwrap_or(&rest[end + "\n+++".len()..])
        .to_string();

    let mut values = BTreeMap::new();
    for raw_line in front_matter.lines() {
        let line = strip_inline_comment(raw_line);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        values.insert(key.trim().to_string(), parse_toml_value(value.trim()));
    }

    Ok((values, body))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TomlValue {
    String(String),
    Bool(bool),
    Integer(i32),
    Array(Vec<String>),
}

fn read_page(path: &Path, kind: PageKind, url_path: String) -> Result<Page> {
    let raw = fs::read_to_string(path)?;
    let (front_matter, body) = parse_front_matter(&raw)?;
    let title = front_matter
        .get("title")
        .and_then(TomlValue::as_str)
        .ok_or_else(|| format!("{} is missing title", path.display()))?
        .to_string();

    Ok(Page {
        title,
        body,
        date: front_matter
            .get("date")
            .and_then(TomlValue::as_str)
            .map(str::to_string),
        draft: front_matter
            .get("draft")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false),
        menu: front_matter
            .get("menu")
            .and_then(TomlValue::as_str)
            .map(str::to_string),
        weight: front_matter.get("weight").and_then(TomlValue::as_int),
        url_path,
        kind,
    })
}

impl TomlValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i32> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }
}

fn parse_toml_value(value: &str) -> TomlValue {
    let value = value.trim();
    if let Some(items) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        return TomlValue::Array(
            items
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(unquote)
                .collect(),
        );
    }
    match value {
        "true" => TomlValue::Bool(true),
        "false" => TomlValue::Bool(false),
        _ => value
            .parse::<i32>()
            .map(TomlValue::Integer)
            .unwrap_or_else(|_| TomlValue::String(unquote(value))),
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn strip_inline_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => return line[..index].to_string(),
            _ => {}
        }
    }
    line.to_string()
}

fn write_site(site: &Site, paths: &BuildPaths) -> Result<()> {
    if paths.output_dir.exists() {
        fs::remove_dir_all(&paths.output_dir)?;
    }
    fs::create_dir_all(&paths.output_dir)?;

    copy_static(&paths.static_dir, &paths.output_dir)?;

    write_page_file(
        &paths.output_dir,
        &site.blog.url_path,
        &render_blog_index(site),
    )?;
    write_page_file(&paths.output_dir, "/blog/", &render_blog_index(site))?;
    for page in &site.pages {
        write_page_file(&paths.output_dir, &page.url_path, &render_page(site, page))?;
    }
    for post in &site.posts {
        write_page_file(&paths.output_dir, &post.url_path, &render_post(site, post))?;
    }

    fs::write(paths.output_dir.join("404.html"), render_404(site))?;
    fs::write(paths.output_dir.join("robots.txt"), render_robots(site))?;
    fs::write(paths.output_dir.join("sitemap.xml"), render_sitemap(site))?;
    fs::write(
        paths.output_dir.join("index.xml"),
        render_rss(site, FeedScope::Root),
    )?;
    fs::write(
        paths.output_dir.join("blog").join("index.xml"),
        render_rss(site, FeedScope::Archive),
    )?;

    Ok(())
}

fn copy_static(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&destination_path)?;
            copy_static(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, destination_path)?;
        }
    }
    Ok(())
}

fn write_page_file(output_dir: &Path, url_path: &str, html: &str) -> Result<()> {
    let relative = url_path.trim_matches('/');
    let file_path = if relative.is_empty() {
        output_dir.join("index.html")
    } else {
        output_dir.join(relative).join("index.html")
    };
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, html)?;
    Ok(())
}

fn render_page(site: &Site, page: &Page) -> String {
    let description = page_description(site, page);
    let content = render_markdown(&page.body);
    render_document(site, page, &description, &content)
}

fn render_blog_index(site: &Site) -> String {
    let mut content = String::from("<content>\n  <ul class=\"blog-posts\">\n");
    for post in &site.posts {
        let date = post.date.as_deref().unwrap_or_default();
        content.push_str("    <li>\n");
        content.push_str("      <span><i>");
        content.push_str(&format!(
            "<time datetime='{}'>{}</time>",
            html_escape(&date_only(date)),
            html_escape(&display_date(date))
        ));
        content.push_str("</i></span>\n");
        content.push_str(&format!(
            "      <a href=\"{}\">{}</a>\n",
            attr_escape(&page_url(&post.url_path)),
            html_escape(&post.title)
        ));
        content.push_str("    </li>\n");
    }
    content.push_str("  </ul>\n</content>\n");
    render_document(site, &site.blog, &site.config.description, &content)
}

fn render_post(site: &Site, post: &Page) -> String {
    let mut content = String::new();
    content.push_str(&format!("\n<h1>{}</h1>\n", html_escape(&post.title)));
    if let Some(date) = &post.date {
        content.push_str(&format!(
            "<p><i><time datetime='{}'>{}</time></i></p>\n",
            html_escape(&date_only(date)),
            html_escape(&display_date(date))
        ));
    }
    content.push_str("<content>\n");
    content.push_str(&render_markdown(&post.body));
    content.push_str("\n</content>\n");

    if site.config.enable_post_navigator {
        content.push_str(&render_post_navigator(site, post));
    }

    render_document(site, post, &page_description(site, post), &content)
}

fn render_post_navigator(site: &Site, post: &Page) -> String {
    let Some(index) = site
        .posts
        .iter()
        .position(|candidate| candidate.url_path == post.url_path)
    else {
        return String::new();
    };
    let previous = site.posts.get(index + 1);
    let next = index.checked_sub(1).and_then(|value| site.posts.get(value));

    let previous_html = previous
        .map(|page| {
            format!(
                "<a href=\"{}\">&lt;&lt; Previous Post</a>",
                attr_escape(&page_url(&page.url_path))
            )
        })
        .unwrap_or_else(|| "<strike>&lt;&lt; Previous Post</strike>".to_string());
    let next_html = next
        .map(|page| {
            format!(
                "<a href=\"{}\">Next Post &gt;&gt;</a>",
                attr_escape(&page_url(&page.url_path))
            )
        })
        .unwrap_or_else(|| "<strike>Next Post &gt;&gt;</strike>".to_string());

    format!(
        "\n<nav class=\"post-nav\" aria-label=\"Post navigation\">\n  <p>{previous_html}</p>\n  <p aria-hidden=\"true\">/</p>\n  <p>{next_html}</p>\n</nav>\n"
    )
}

fn render_404(site: &Site) -> String {
    let page = Page {
        title: "404".to_string(),
        body: String::new(),
        date: None,
        draft: false,
        menu: None,
        weight: None,
        url_path: "/404.html".to_string(),
        kind: PageKind::Page,
    };
    render_document(
        site,
        &page,
        "Page not found",
        "<h1>404</h1>\n<p>Page not found.</p>\n",
    )
}

fn render_document(site: &Site, page: &Page, description: &str, main_content: &str) -> String {
    let mut document = String::new();
    document.push_str("<!DOCTYPE html>\n<html lang=\"");
    document.push_str(&attr_escape(&site.config.language_code));
    document.push_str("\">\n\n<head>\n");
    document.push_str("  <meta charset=\"utf-8\">\n");
    document.push_str(
        "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n",
    );
    if !site.config.favicon.is_empty() {
        document.push_str(&format!(
            "<link rel=\"shortcut icon\" href=\"{}\" />\n",
            attr_escape(&asset_url(&site.config.favicon))
        ));
    }
    document.push_str(&format!(
        "<title>{} | {}</title>\n",
        html_escape(&page.title),
        html_escape(&site.config.title)
    ));
    document.push_str(&format!(
        "<meta name=\"title\" content=\"{}\" />\n",
        attr_escape(&page.title)
    ));
    document.push_str(&format!(
        "<meta name=\"description\" content=\"{}\" />\n",
        attr_escape(description)
    ));
    document.push_str("<meta name=\"keywords\" content=\"\" />\n\n");
    document.push_str(&render_open_graph(site, page, description));
    document.push_str("<meta name=\"referrer\" content=\"no-referrer-when-downgrade\" />\n");
    if page.kind == PageKind::Section && page.url_path == "/" {
        document.push_str(&format!(
            "  <link rel=\"alternate\" type=\"application/rss+xml\" href=\"{}\" title=\"{}\" />\n",
            attr_escape(&page_url("/index.xml")),
            attr_escape(&site.config.title)
        ));
    } else if page.kind == PageKind::Section {
        document.push_str(&format!(
            "  <link rel=\"alternate\" type=\"application/rss+xml\" href=\"{}\" title=\"{}\" />\n",
            attr_escape(&page_url("/blog/index.xml")),
            attr_escape(&site.config.title)
        ));
    }
    document.push_str("  <script>");
    document.push_str(THEME_BOOTSTRAP);
    document.push_str("</script>\n");
    document.push_str("  <style>\n");
    document.push_str(STYLE);
    document.push_str("\n</style>\n\n</head>\n\n<body>\n");
    document.push_str(&render_header(site));
    document.push_str("  <main>\n");
    document.push_str(main_content);
    document.push_str("\n  </main>\n  <footer>\n</footer>\n<script>");
    document.push_str(THEME_SCRIPT);
    document.push_str("</script>\n</body>\n\n</html>\n");
    document
}

fn render_open_graph(site: &Site, page: &Page, description: &str) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "\n<meta property=\"og:url\" content=\"{}\">\n",
        attr_escape(&absolute_url(site, &page.url_path))
    ));
    html.push_str(&format!(
        "  <meta property=\"og:site_name\" content=\"{}\">\n",
        attr_escape(&site.config.title)
    ));
    html.push_str(&format!(
        "  <meta property=\"og:title\" content=\"{}\">\n",
        attr_escape(&page.title)
    ));
    html.push_str(&format!(
        "  <meta property=\"og:description\" content=\"{}\">\n",
        attr_escape(description)
    ));
    html.push_str(&format!(
        "  <meta property=\"og:locale\" content=\"{}\">\n",
        attr_escape(&site.config.language_code.replace('-', "_"))
    ));
    html.push_str(&format!(
        "  <meta property=\"og:type\" content=\"{}\">\n",
        if page.kind == PageKind::Post {
            "article"
        } else {
            "website"
        }
    ));
    if page.kind == PageKind::Post
        && let Some(date) = &page.date
    {
        html.push_str("  <meta property=\"article:section\" content=\"blog\">\n");
        html.push_str(&format!(
            "  <meta property=\"article:published_time\" content=\"{}\">\n",
            attr_escape(date)
        ));
        html.push_str(&format!(
            "  <meta property=\"article:modified_time\" content=\"{}\">\n",
            attr_escape(date)
        ));
    }
    if let Some(image) = site.config.images.first() {
        html.push_str(&format!(
            "  <meta property=\"og:image\" content=\"{}\">\n",
            attr_escape(&absolute_asset_url(site, image))
        ));
        html.push_str(&format!(
            "  <meta name=\"twitter:card\" content=\"summary_large_image\">\n  <meta name=\"twitter:image\" content=\"{}\">\n",
            attr_escape(&absolute_asset_url(site, image))
        ));
    }
    html.push_str(&format!(
        "  <meta name=\"twitter:title\" content=\"{}\">\n",
        attr_escape(&page.title)
    ));
    html.push_str(&format!(
        "  <meta name=\"twitter:description\" content=\"{}\">\n",
        attr_escape(description)
    ));
    html.push_str(&format!(
        "  <meta itemprop=\"name\" content=\"{}\">\n",
        attr_escape(&page.title)
    ));
    html.push_str(&format!(
        "  <meta itemprop=\"description\" content=\"{}\">\n",
        attr_escape(description)
    ));
    if let Some(date) = page.date.as_deref().or_else(|| newest_post_date(site)) {
        html.push_str(&format!(
            "  <meta itemprop=\"datePublished\" content=\"{}\">\n",
            attr_escape(date)
        ));
        html.push_str(&format!(
            "  <meta itemprop=\"dateModified\" content=\"{}\">\n",
            attr_escape(date)
        ));
    }
    html
}

fn render_header(site: &Site) -> String {
    let mut pages = Vec::new();
    pages.push(&site.blog);
    pages.extend(
        site.pages
            .iter()
            .filter(|page| page.menu.as_deref() == Some("main")),
    );
    pages.sort_by_key(|page| (page.weight.unwrap_or(i32::MAX), page.title.clone()));

    let mut header = String::new();
    header.push_str("  <header><a href=\"/\" class=\"title\">\n");
    header.push_str(&format!("  <h2>{}</h2>\n", html_escape(&site.config.title)));
    header.push_str("</a>\n<nav>\n");
    for page in pages {
        header.push_str(&format!(
            "<a href=\"{}\">{}</a>\n\n",
            attr_escape(&page.url_path),
            html_escape(&page.title)
        ));
    }
    header.push_str(
        "<button class=\"theme-toggle\" type=\"button\" aria-label=\"Toggle color theme\" data-theme-toggle>\n  <span class=\"theme-toggle__track\" aria-hidden=\"true\"><span class=\"theme-toggle__knob\"></span></span>\n</button>\n",
    );
    header.push_str("</nav>\n</header>\n");
    header
}

pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(markdown, options);
    let mut events = parser.collect::<Vec<_>>();
    add_heading_ids(&mut events);

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());
    output
}

fn add_heading_ids(events: &mut [Event<'_>]) {
    let mut index = 0;
    while index < events.len() {
        if matches!(&events[index], Event::Start(Tag::Heading { id: None, .. })) {
            let mut heading_text = String::new();
            let mut scan = index + 1;
            while scan < events.len() {
                match &events[scan] {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(value) | Event::Code(value) => heading_text.push_str(value),
                    Event::SoftBreak | Event::HardBreak => heading_text.push(' '),
                    _ => {}
                }
                scan += 1;
            }

            let id_value = heading_id(&heading_text);
            if !id_value.is_empty()
                && let Event::Start(Tag::Heading { id, .. }) = &mut events[index]
            {
                *id = Some(CowStr::from(id_value));
            }
        }

        index += 1;
    }
}

fn page_description(site: &Site, page: &Page) -> String {
    let rendered = render_markdown(&page.body);
    let text = strip_tags(&rendered);
    if text.trim().is_empty() {
        return site.config.description.clone();
    }
    truncate_words(text.trim(), 55)
}

fn strip_tags(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_words(text: &str, max_words: usize) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        return text.to_string();
    }
    let mut truncated = words[..max_words].join(" ");
    truncated.push('…');
    truncated
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_' || ch == '/') && !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn heading_id(input: &str) -> String {
    let mut id = String::new();
    let mut previous_dash = false;

    for ch in input.chars() {
        if ch.is_alphanumeric() {
            for lowercase in ch.to_lowercase() {
                id.push(lowercase);
            }
            previous_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_' || ch == '/') && !previous_dash {
            id.push('-');
            previous_dash = true;
        }
    }

    id.trim_matches('-').to_string()
}

pub fn date_only(date: &str) -> String {
    date.chars().take(10).collect()
}

pub fn display_date(date: &str) -> String {
    let date = date_only(date);
    let mut parts = date.split('-');
    let year = parts.next().unwrap_or_default();
    let month = parts.next().unwrap_or_default();
    let day = parts.next().unwrap_or_default();
    let month = match month {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => "",
    };
    format!("{day} {month}, {year}")
}

fn rss_date(date: &str) -> String {
    let date_part = date_only(date);
    let mut date_parts = date_part.split('-');
    let year = date_parts
        .next()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1970);
    let month = date_parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);
    let day = date_parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);
    let time = date.get(11..19).unwrap_or("00:00:00");
    let offset = date
        .get(19..25)
        .map(|value| value.replace(':', ""))
        .unwrap_or_else(|| "+0000".to_string());
    format!(
        "{}, {day:02} {} {} {} {}",
        weekday_name(year, month, day),
        month_name(month),
        year,
        time,
        offset
    )
}

fn weekday_name(year: i32, month: u32, day: u32) -> &'static str {
    let month_offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut year = year;
    if month < 3 {
        year -= 1;
    }
    let index = (year + year / 4 - year / 100
        + year / 400
        + month_offsets[(month - 1) as usize]
        + day as i32)
        % 7;
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][index as usize]
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Jan",
    }
}

fn render_robots(site: &Site) -> String {
    format!(
        "User-Agent: *\nSitemap: {}\n",
        absolute_url(site, "/sitemap.xml")
    )
}

fn render_sitemap(site: &Site) -> String {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"\n  xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    push_sitemap_url(
        &mut sitemap,
        site,
        &site.blog.url_path,
        newest_post_date(site),
    );
    for page in &site.pages {
        push_sitemap_url(&mut sitemap, site, &page.url_path, page.date.as_deref());
    }
    push_sitemap_url(&mut sitemap, site, "/blog/", newest_post_date(site));
    for post in &site.posts {
        push_sitemap_url(&mut sitemap, site, &post.url_path, post.date.as_deref());
    }
    sitemap.push_str("</urlset>\n");
    sitemap
}

fn push_sitemap_url(sitemap: &mut String, site: &Site, path: &str, lastmod: Option<&str>) {
    sitemap.push_str("  <url>\n");
    sitemap.push_str(&format!(
        "    <loc>{}</loc>\n",
        xml_escape(&absolute_url(site, path))
    ));
    if let Some(lastmod) = lastmod {
        sitemap.push_str(&format!("    <lastmod>{}</lastmod>\n", xml_escape(lastmod)));
    }
    sitemap.push_str("  </url>\n");
}

#[derive(Clone, Copy)]
enum FeedScope {
    Root,
    Archive,
}

fn render_rss(site: &Site, scope: FeedScope) -> String {
    let (title, link, description, feed_path) = match scope {
        FeedScope::Root => (
            format!("Blog on {}", site.config.title),
            absolute_url(site, "/"),
            format!("Recent content in Blog on {}", site.config.title),
            "/index.xml",
        ),
        FeedScope::Archive => (
            format!("Blog on {}", site.config.title),
            absolute_url(site, "/blog/"),
            format!("Recent content in Blog on {}", site.config.title),
            "/blog/index.xml",
        ),
    };
    let last_build = newest_post_date(site)
        .map(rss_date)
        .unwrap_or_else(|| "Thu, 01 Jan 1970 00:00:00 +0000".to_string());

    let mut rss = String::new();
    rss.push_str("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n");
    rss.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n  <channel>\n");
    rss.push_str(&format!("    <title>{}</title>\n", xml_escape(&title)));
    rss.push_str(&format!("    <link>{}</link>\n", xml_escape(&link)));
    rss.push_str(&format!(
        "    <description>{}</description>\n",
        xml_escape(&description)
    ));
    rss.push_str("    <generator>aacnsilva-blog</generator>\n");
    rss.push_str(&format!(
        "    <language>{}</language>\n",
        xml_escape(&site.config.language_code)
    ));
    if !site.config.copyright.is_empty() {
        rss.push_str(&format!(
            "    <copyright>{}</copyright>\n",
            xml_escape(&site.config.copyright)
        ));
    }
    rss.push_str(&format!(
        "    <lastBuildDate>{last_build}</lastBuildDate>\n"
    ));
    rss.push_str(&format!(
        "    <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\" />\n",
        attr_escape(&absolute_url(site, feed_path))
    ));

    for post in &site.posts {
        rss.push_str("    <item>\n");
        rss.push_str(&format!(
            "      <title>{}</title>\n",
            xml_escape(&post.title)
        ));
        rss.push_str(&format!(
            "      <link>{}</link>\n",
            xml_escape(&absolute_url(site, &post.url_path))
        ));
        if let Some(date) = &post.date {
            rss.push_str(&format!("      <pubDate>{}</pubDate>\n", rss_date(date)));
        }
        rss.push_str(&format!(
            "      <guid>{}</guid>\n",
            xml_escape(&absolute_url(site, &post.url_path))
        ));
        rss.push_str(&format!(
            "      <description>{}</description>\n",
            xml_escape(&render_markdown(&post.body))
        ));
        rss.push_str("    </item>\n");
    }

    rss.push_str("  </channel>\n</rss>\n");
    rss
}

fn newest_post_date(site: &Site) -> Option<&str> {
    site.posts.first().and_then(|post| post.date.as_deref())
}

fn absolute_url(site: &Site, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let base = site.config.base_url.trim_end_matches('/');
    if path == "/" {
        return format!("{base}/");
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

fn page_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("mailto:") {
        return path.to_string();
    }
    if path == "/" {
        return "/".to_string();
    }
    format!("/{}", path.trim_start_matches('/'))
}

fn asset_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    page_url(path.strip_prefix("static/").unwrap_or(path))
}

fn absolute_asset_url(site: &Site, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    absolute_url(site, path.strip_prefix("static/").unwrap_or(path))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&#39;")
}

fn attr_escape(input: &str) -> String {
    html_escape(input).replace('"', "&quot;")
}

fn xml_escape(input: &str) -> String {
    attr_escape(input)
}

const THEME_BOOTSTRAP: &str = r#"(function(){try{var theme=localStorage.getItem("theme");if(theme==="light"||theme==="dark"){document.documentElement.dataset.theme=theme;}}catch(_){}})();"#;

const THEME_SCRIPT: &str = r#"(function(){var root=document.documentElement;var toggle=document.querySelector("[data-theme-toggle]");if(!toggle){return;}var media=window.matchMedia("(prefers-color-scheme: dark)");function current(){return root.dataset.theme||(media.matches?"dark":"light");}function sync(){toggle.setAttribute("aria-pressed",current()==="dark"?"true":"false");}toggle.addEventListener("click",function(){var next=current()==="dark"?"light":"dark";root.dataset.theme=next;try{localStorage.setItem("theme",next);}catch(_){}sync();});if(media.addEventListener){media.addEventListener("change",sync);}sync();})();"#;

const STYLE: &str = r#"  :root {
    color-scheme: light;
    --width: 760px;
    --font-main: Georgia, "Times New Roman", serif;
    --font-secondary: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --font-mono: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
    --background-color: #f7f3ed;
    --surface-color: #fffaf3;
    --surface-muted: #eee8de;
    --heading-color: #25231f;
    --text-color: #38342f;
    --muted-color: #6d675f;
    --border-color: #d9d0c3;
    --link-color: #315f6d;
    --visited-color: #735b84;
    --accent-color: #5c7568;
    --accent-contrast: #f8fbf7;
    --code-background: #ebe5da;
    --code-color: #27231f;
    --blockquote-background: #f0e8dc;
    --shadow-color: rgba(68, 55, 40, 0.08);
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
      color-scheme: dark;
      --background-color: #171a1d;
      --surface-color: #202428;
      --surface-muted: #2a2e32;
      --heading-color: #f3eee6;
      --text-color: #e7dfd3;
      --muted-color: #b9afa3;
      --border-color: #3c4144;
      --link-color: #9bc7d2;
      --visited-color: #d2adc6;
      --accent-color: #abc5b5;
      --accent-contrast: #162019;
      --code-background: #121518;
      --code-color: #f3eee6;
      --blockquote-background: #242b2b;
      --shadow-color: rgba(0, 0, 0, 0.24);
    }
  }

  :root[data-theme="dark"] {
    color-scheme: dark;
    --background-color: #171a1d;
    --surface-color: #202428;
    --surface-muted: #2a2e32;
    --heading-color: #f3eee6;
    --text-color: #e7dfd3;
    --muted-color: #b9afa3;
    --border-color: #3c4144;
    --link-color: #9bc7d2;
    --visited-color: #d2adc6;
    --accent-color: #abc5b5;
    --accent-contrast: #162019;
    --code-background: #121518;
    --code-color: #f3eee6;
    --blockquote-background: #242b2b;
    --shadow-color: rgba(0, 0, 0, 0.24);
  }

  :root[data-theme="light"] {
    color-scheme: light;
  }

  * {
    box-sizing: border-box;
  }

  html {
    background: var(--background-color);
  }

  body {
    min-height: 100vh;
    margin: 0;
    padding: 0 22px;
    text-align: left;
    background: var(--background-color);
    color: var(--text-color);
    font-family: var(--font-secondary);
    font-size: 17px;
    line-height: 1.72;
    word-wrap: break-word;
    overflow-wrap: break-word;
    transition: background-color 180ms ease, color 180ms ease;
  }

  header,
  main,
  footer {
    max-width: var(--width);
    margin-inline: auto;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 34px 0 22px;
    margin-bottom: 30px;
    border-bottom: 1px solid var(--border-color);
  }

  .title {
    color: var(--heading-color);
  }

  .title:hover {
    text-decoration: none;
  }

  .title h2 {
    margin: 0;
    font-family: var(--font-main);
    font-size: 1.42rem;
    font-weight: 650;
    line-height: 1.1;
  }

  nav {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 8px;
  }

  nav a {
    display: inline-flex;
    align-items: center;
    min-height: 36px;
    padding: 0 10px;
    border-radius: 999px;
    color: var(--muted-color);
    font-size: 0.9rem;
    font-weight: 650;
  }

  nav a:hover {
    background: var(--surface-muted);
    color: var(--heading-color);
    text-decoration: none;
  }

  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 36px;
    padding: 0;
    border: 1px solid var(--border-color);
    border-radius: 999px;
    background: var(--surface-color);
    color: var(--heading-color);
    box-shadow: 0 8px 24px var(--shadow-color);
    cursor: pointer;
  }

  .theme-toggle:focus-visible,
  a:focus-visible {
    outline: 3px solid var(--accent-color);
    outline-offset: 3px;
  }

  .theme-toggle__track {
    position: relative;
    width: 26px;
    height: 14px;
    border-radius: 999px;
    background: var(--surface-muted);
  }

  .theme-toggle__knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--accent-color);
    transition: transform 180ms ease;
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .theme-toggle__knob {
      transform: translateX(12px);
    }
  }

  :root[data-theme="dark"] .theme-toggle__knob {
    transform: translateX(12px);
  }

  :root[data-theme="light"] .theme-toggle__knob {
    transform: translateX(0);
  }

  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    color: var(--heading-color);
    font-family: var(--font-main);
    line-height: 1.18;
  }

  h1 {
    margin: 0 0 22px;
    font-size: 2.35rem;
    font-weight: 650;
  }

  h2 {
    margin-top: 2.4em;
    margin-bottom: 0.65em;
    font-size: 1.5rem;
  }

  h3 {
    margin-top: 2em;
    margin-bottom: 0.55em;
    font-size: 1.15rem;
  }

  h4,
  h5,
  h6 {
    margin-top: 1.7em;
    margin-bottom: 0.45em;
    font-size: 1rem;
  }

  p,
  ul,
  ol,
  blockquote,
  table,
  pre {
    margin-top: 0;
    margin-bottom: 1.18rem;
  }

  a {
    color: var(--link-color);
    cursor: pointer;
    text-decoration-thickness: 0.08em;
    text-underline-offset: 0.18em;
  }

  a:hover {
    color: var(--accent-color);
  }

  main a:visited {
    color: var(--visited-color);
  }

  strong,
  b {
    color: var(--heading-color);
  }

  time {
    color: var(--muted-color);
    font-family: var(--font-mono);
    font-size: 0.88rem;
    font-style: normal;
  }

  content {
    display: block;
  }

  hr {
    height: 1px;
    margin: 2rem 0;
    border: 0;
    background: var(--border-color);
  }

  img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
  }

  code {
    padding: 0.12em 0.34em;
    border-radius: 5px;
    background: var(--code-background);
    color: var(--code-color);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }

  pre {
    overflow-x: auto;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--code-background);
  }

  pre code {
    display: block;
    padding: 16px;
    background: transparent;
  }

  blockquote {
    padding: 18px 20px;
    border-left: 4px solid var(--accent-color);
    border-radius: 0 8px 8px 0;
    background: var(--blockquote-background);
    color: var(--text-color);
    font-style: italic;
  }

  table {
    display: block;
    width: 100%;
    border-collapse: collapse;
    overflow-x: auto;
    font-size: 0.95rem;
  }

  th,
  td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color);
    vertical-align: top;
  }

  th {
    color: var(--heading-color);
    text-align: left;
    background: var(--surface-muted);
  }

  footer {
    margin-top: 48px;
    min-height: 36px;
  }

  ul.blog-posts {
    margin: 0;
    padding: 0;
    list-style-type: none;
  }

  ul.blog-posts li {
    display: grid;
    grid-template-columns: 132px 1fr;
    gap: 18px;
    align-items: baseline;
    padding: 16px 0;
    border-bottom: 1px solid var(--border-color);
  }

  ul.blog-posts li a {
    color: var(--heading-color);
    font-family: var(--font-main);
    font-size: 1.08rem;
    line-height: 1.35;
    text-decoration: none;
  }

  ul.blog-posts li a:hover {
    color: var(--link-color);
    text-decoration: underline;
  }

  ul.blog-posts li a:visited {
    color: var(--visited-color);
  }

  .post-nav {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin-top: 42px;
    padding-top: 20px;
    border-top: 1px solid var(--border-color);
    color: var(--muted-color);
    font-size: 0.9rem;
  }

  .post-nav p {
    margin: 0;
  }

  .post-nav strike {
    color: var(--muted-color);
  }

  @media (max-width: 640px) {
    body {
      padding-inline: 18px;
      font-size: 16px;
    }

    header {
      align-items: flex-start;
      flex-direction: column;
      gap: 14px;
      margin-bottom: 34px;
      padding-top: 28px;
    }

    nav {
      justify-content: flex-start;
    }

    h1 {
      font-size: 2rem;
    }

    ul.blog-posts li {
      grid-template-columns: 1fr;
      gap: 4px;
    }

    .post-nav {
      flex-wrap: wrap;
    }
  }"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_site_config_values_used_by_the_generator() {
        let config = parse_site_config(
            r#"
baseURL = 'https://example.com/'
title = "Top title"
author = "Author"
copyright = "Copyright"
languageCode = "en-US"

[params]
description = "A blog"
favicon = "static/images/favicon.png"
images = ["static/images/share.png"]
title = "Param title"
enablePostNavigator = true
"#,
        )
        .unwrap();

        assert_eq!(config.base_url, "https://example.com/");
        assert_eq!(config.title, "Param title");
        assert_eq!(config.author, "Author");
        assert_eq!(config.description, "A blog");
        assert_eq!(config.images, vec!["static/images/share.png"]);
        assert!(config.enable_post_navigator);
    }

    #[test]
    fn parses_toml_front_matter_without_external_toml_dependency() {
        let (front_matter, body) = parse_front_matter(
            "+++\ndate = '2026-05-20T10:30:00+01:00'\ndraft = false\ntitle = 'Hello'\nweight = 10\n+++\n# Body\n",
        )
        .unwrap();

        assert_eq!(
            front_matter.get("title").and_then(TomlValue::as_str),
            Some("Hello")
        );
        assert_eq!(
            front_matter.get("date").and_then(TomlValue::as_str),
            Some("2026-05-20T10:30:00+01:00")
        );
        assert_eq!(
            front_matter.get("draft").and_then(TomlValue::as_bool),
            Some(false)
        );
        assert_eq!(
            front_matter.get("weight").and_then(TomlValue::as_int),
            Some(10)
        );
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn slugifies_post_titles_into_url_paths() {
        assert_eq!(
            slugify("Agentic programming for Business Central with AL, VS Code, and Copilot"),
            "agentic-programming-for-business-central-with-al-vs-code-and-copilot"
        );
        assert_eq!(
            slugify("Full-text search in Business Central"),
            "full-text-search-in-business-central"
        );
    }

    #[test]
    fn formats_dates_for_lists_and_feeds() {
        assert_eq!(date_only("2026-03-09T10:30:00+00:00"), "2026-03-09");
        assert_eq!(display_date("2026-03-09T10:30:00+00:00"), "09 Mar, 2026");
        assert_eq!(
            rss_date("2026-05-20T10:30:00+01:00"),
            "Wed, 20 May 2026 10:30:00 +0100"
        );
    }

    #[test]
    fn renders_markdown_features_used_by_the_content_directory() {
        let html = render_markdown(
            "## About {#about}\n\n## What is Business Central?\n\n| A | B |\n| - | - |\n| `x` | **y** |\n\n```al\ncodeunit 1 Foo {}\n```\n",
        );

        assert!(html.contains("<h2 id=\"about\">About</h2>"));
        assert!(
            html.contains("<h2 id=\"what-is-business-central\">What is Business Central?</h2>")
        );
        assert!(html.contains("<table>"));
        assert!(html.contains("<code>x</code>"));
        assert!(html.contains("<strong>y</strong>"));
        assert!(html.contains("class=\"language-al\""));
    }

    #[test]
    fn escapes_titles_and_attributes() {
        assert_eq!(
            html_escape("António's <DevLog>"),
            "António&#39;s &lt;DevLog&gt;"
        );
        assert_eq!(
            attr_escape("\"quoted\" & more"),
            "&quot;quoted&quot; &amp; more"
        );
    }

    #[test]
    fn builds_root_relative_browser_urls() {
        assert_eq!(page_url("/blog/"), "/blog/");
        assert_eq!(page_url("resume/"), "/resume/");
        assert_eq!(
            asset_url("static/images/favicon-32x32.png"),
            "/images/favicon-32x32.png"
        );
        assert_eq!(
            page_url("https://example.com/external"),
            "https://example.com/external"
        );
    }
}
