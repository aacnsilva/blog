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
    pub menu_links: Vec<MenuLink>,
    /// Home page headline and the line under it.
    pub headline: String,
    pub intro: String,
    /// The projects shown as panels on the home page and as a strip elsewhere.
    pub projects: Vec<Project>,
}

/// One project panel. The four colours are CSS colours for that panel only,
/// so each project keeps its own identity in both site themes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub url: String,
    pub label: String,
    pub description: String,
    pub background: String,
    pub text: String,
    pub muted: String,
    pub title: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenuLink {
    pub name: String,
    pub url: String,
    pub weight: Option<i32>,
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
            menu_links: Vec::new(),
            headline: String::new(),
            intro: String::new(),
            projects: Vec::new(),
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
            if line.starts_with("[[") && section == "menu.main" {
                config.menu_links.push(MenuLink::default());
            }
            if line.starts_with("[[") && section == "projects" {
                config.projects.push(Project::default());
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = parse_toml_value(value.trim());

        match (section.as_str(), key, value) {
            ("", "baseURL", TomlValue::String(value)) => config.base_url = value,
            ("" | "params", "title", TomlValue::String(value)) => config.title = value,
            ("", "author", TomlValue::String(value)) => config.author = value,
            ("", "copyright", TomlValue::String(value)) => config.copyright = value,
            ("", "languageCode", TomlValue::String(value)) => config.language_code = value,
            ("params", "description", TomlValue::String(value)) => config.description = value,
            ("params", "favicon", TomlValue::String(value)) => config.favicon = value,
            ("params", "images", TomlValue::Array(value)) => config.images = value,
            ("params", "enablePostNavigator", TomlValue::Bool(value)) => {
                config.enable_post_navigator = value;
            }
            ("params", "headline", TomlValue::String(value)) => config.headline = value,
            ("params", "intro", TomlValue::String(value)) => config.intro = value,
            ("projects", key, TomlValue::String(value)) => {
                if let Some(project) = config.projects.last_mut() {
                    match key {
                        "name" => project.name = value,
                        "url" => project.url = value,
                        "label" => project.label = value,
                        "description" => project.description = value,
                        "background" => project.background = value,
                        "text" => project.text = value,
                        "muted" => project.muted = value,
                        "title" => project.title = value,
                        _ => {}
                    }
                }
            }
            ("menu.main", "name", TomlValue::String(value)) => {
                if let Some(link) = config.menu_links.last_mut() {
                    link.name = value;
                }
            }
            ("menu.main", "url", TomlValue::String(value)) => {
                if let Some(link) = config.menu_links.last_mut() {
                    link.url = value;
                }
            }
            ("menu.main", "weight", TomlValue::Integer(value)) => {
                if let Some(link) = config.menu_links.last_mut() {
                    link.weight = Some(value);
                }
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
            .map_or_else(|_| TomlValue::String(unquote(value)), TomlValue::Integer),
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

    write_page_file(&paths.output_dir, &site.blog.url_path, &render_home(site))?;
    write_page_file(&paths.output_dir, ARCHIVE_PATH, &render_archive(site))?;
    for page in &site.pages {
        write_page_file(&paths.output_dir, &page.url_path, &render_page(site, page))?;
    }
    for post in &site.posts {
        write_page_file(&paths.output_dir, &post.url_path, &render_post(site, post))?;
    }

    fs::write(paths.output_dir.join("404.html"), render_404(site))?;
    fs::write(paths.output_dir.join("robots.txt"), render_robots(site))?;
    fs::write(paths.output_dir.join(".nojekyll"), [])?;
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

/// Where the full list of posts lives. The home page shows only the latest few.
const ARCHIVE_PATH: &str = "/blog/";
const LATEST_POSTS_ON_HOME: usize = 6;

fn render_home(site: &Site) -> String {
    let config = &site.config;
    let headline = if config.headline.is_empty() {
        &config.title
    } else {
        &config.headline
    };

    let mut content = String::from("<div class=\"home\">\n<section class=\"hero\">\n");
    content.push_str(&format!("  <h1>{}</h1>\n", html_escape(headline)));
    if !config.intro.is_empty() {
        content.push_str(&format!("  <p>{}</p>\n", html_escape(&config.intro)));
    }
    content.push_str("</section>\n");

    if !config.projects.is_empty() {
        content.push_str("<section class=\"doors\" aria-label=\"Projects\">\n");
        content.push_str(&render_projects(site));
        content.push_str("</section>\n");
    }

    content.push_str("<section aria-labelledby=\"latest-writing\">\n  <div class=\"section-head\">\n    <h2 id=\"latest-writing\">Latest writing</h2>\n");
    content.push_str(&format!(
        "    <a href=\"{}\">All {} posts →</a>\n  </div>\n  <ul class=\"latest-posts\">\n",
        attr_escape(&page_url(ARCHIVE_PATH)),
        site.posts.len()
    ));
    for post in site.posts.iter().take(LATEST_POSTS_ON_HOME) {
        content.push_str(&format!(
            "    <li>\n      {}\n      <a href=\"{}\">{}</a>\n    </li>\n",
            render_time(post.date.as_deref().unwrap_or_default()),
            attr_escape(&page_url(&post.url_path)),
            html_escape(&post.title)
        ));
    }
    content.push_str("  </ul>\n</section>\n</div>\n");

    let home = Page {
        title: config.title.clone(),
        ..site.blog.clone()
    };
    render_document(site, &home, &config.description, &content)
}

fn render_archive(site: &Site) -> String {
    let mut content = format!(
        "<h1>{}</h1>\n<div class=\"content\">\n  <ul class=\"blog-posts\">\n",
        html_escape(&site.blog.title)
    );
    for post in &site.posts {
        content.push_str(&format!(
            "    <li>\n      {}\n      <a href=\"{}\">{}</a>\n    </li>\n",
            render_time(post.date.as_deref().unwrap_or_default()),
            attr_escape(&page_url(&post.url_path)),
            html_escape(&post.title)
        ));
    }
    content.push_str("  </ul>\n</div>\n");

    let archive = Page {
        url_path: ARCHIVE_PATH.to_string(),
        ..site.blog.clone()
    };
    render_document(site, &archive, &site.config.description, &content)
}

fn render_time(date: &str) -> String {
    format!(
        "<time datetime=\"{}\">{}</time>",
        attr_escape(&date_only(date)),
        html_escape(&display_date(date))
    )
}

/// The project panels: full size on the home page, compact under every other page.
fn render_projects(site: &Site) -> String {
    const ARROW: &str = "<svg aria-hidden=\"true\" width=\"28\" height=\"28\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M7 17 17 7M8 7h9v9\"></path></svg>";

    let mut html = String::new();
    for project in &site.config.projects {
        let mut style = String::new();
        for (property, value) in [
            ("--door-bg", &project.background),
            ("--door-fg", &project.text),
            ("--door-muted", &project.muted),
            ("--door-title", &project.title),
        ] {
            if !value.is_empty() {
                style.push_str(&format!("{property}:{value};"));
            }
        }
        html.push_str(&format!(
            "  <a class=\"door\" href=\"{}\" style=\"{}\">\n    <span class=\"door__label\"><span>{}</span>{ARROW}</span>\n    <span class=\"door__body\">\n      <span class=\"door__name\">{}</span>\n      <span class=\"door__text\">{}</span>\n    </span>\n  </a>\n",
            attr_escape(&project.url),
            attr_escape(&style),
            html_escape(&project.label),
            html_escape(&project.name),
            html_escape(&project.description)
        ));
    }
    html
}

fn render_post(site: &Site, post: &Page) -> String {
    let mut content = String::new();
    content.push_str(&format!("\n<h1>{}</h1>\n", html_escape(&post.title)));
    if let Some(date) = &post.date {
        content.push_str(&format!(
            "<p class=\"post-meta\">{}</p>\n",
            render_time(date)
        ));
    }
    content.push_str("<div class=\"content\">\n");
    content.push_str(&render_markdown(&post.body));
    content.push_str("\n</div>\n");

    if site.config.enable_post_navigator {
        content.push_str(&render_post_navigator(site, post));
    }

    render_document(site, post, &page_description(site, post), &content)
}

fn render_post_navigator(site: &Site, post: &Page) -> String {
    let index = site
        .posts
        .iter()
        .position(|candidate| candidate.url_path == post.url_path)
        .expect("post should exist in site.posts");
    let previous = site.posts.get(index + 1);
    let next = index.checked_sub(1).and_then(|value| site.posts.get(value));

    let previous_html = previous.map_or_else(
        || "<span class=\"post-nav__disabled\">&lt;&lt; Previous Post</span>".to_string(),
        |page| {
            format!(
                "<a href=\"{}\">&lt;&lt; Previous Post</a>",
                attr_escape(&page_url(&page.url_path))
            )
        },
    );
    let next_html = next.map_or_else(
        || "<span class=\"post-nav__disabled\">Next Post &gt;&gt;</span>".to_string(),
        |page| {
            format!(
                "<a href=\"{}\">Next Post &gt;&gt;</a>",
                attr_escape(&page_url(&page.url_path))
            )
        },
    );

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
            "<link rel=\"icon\" href=\"{}\" />\n",
            attr_escape(&asset_url(&site.config.favicon))
        ));
    }
    let is_home = page.kind == PageKind::Section && page.url_path == "/";
    if is_home {
        document.push_str(&format!(
            "<title>{}</title>\n",
            html_escape(&site.config.title)
        ));
    } else {
        document.push_str(&format!(
            "<title>{} | {}</title>\n",
            html_escape(&page.title),
            html_escape(&site.config.title)
        ));
    }
    document.push_str(
        "<link rel=\"preload\" href=\"/fonts/bricolage-grotesque-latin-wght-normal.woff2\" as=\"font\" type=\"font/woff2\" crossorigin />\n",
    );
    document.push_str(&format!(
        "<meta name=\"title\" content=\"{}\" />\n",
        attr_escape(&page.title)
    ));
    document.push_str(&format!(
        "<meta name=\"description\" content=\"{}\" />\n",
        attr_escape(description)
    ));
    if page.url_path != "/404.html" {
        document.push_str(&format!(
            "<link rel=\"canonical\" href=\"{}\" />\n",
            attr_escape(&absolute_url(site, &page.url_path))
        ));
    }
    document.push_str(&render_open_graph(site, page, description));
    document.push_str("<meta name=\"referrer\" content=\"no-referrer-when-downgrade\" />\n");
    if is_home {
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
    document.push_str(if is_home {
        "  <main>\n"
    } else {
        "  <main class=\"prose\">\n"
    });
    document.push_str(main_content);
    document.push_str("\n  </main>\n");
    if !is_home && !site.config.projects.is_empty() {
        document.push_str("  <aside class=\"projects-strip\" aria-labelledby=\"projects-strip\">\n  <h2 id=\"projects-strip\">Things I've built</h2>\n  <div class=\"doors doors--compact\">\n");
        document.push_str(&render_projects(site));
        document.push_str("  </div>\n  </aside>\n");
    }
    document.push_str(&render_footer(site));
    document.push_str("<script>");
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
    // The section page is the home page; its menu entry points at the full archive.
    let mut links: Vec<(i32, &str, &str)> = std::iter::once((
        site.blog.weight.unwrap_or(i32::MAX),
        site.blog.title.as_str(),
        ARCHIVE_PATH,
    ))
    .chain(
        site.pages
            .iter()
            .filter(|page| page.menu.as_deref() == Some("main"))
            .map(|page| {
                (
                    page.weight.unwrap_or(i32::MAX),
                    page.title.as_str(),
                    page.url_path.as_str(),
                )
            }),
    )
    .chain(site.config.menu_links.iter().map(|link| {
        (
            link.weight.unwrap_or(i32::MAX),
            link.name.as_str(),
            link.url.as_str(),
        )
    }))
    .collect();
    links.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));

    let brand = if site.config.author.is_empty() {
        &site.config.title
    } else {
        &site.config.author
    };

    let mut header = String::new();
    header.push_str(&format!(
        "  <header class=\"site-header\">\n<a href=\"/\" class=\"brand\">{}</a>\n<nav class=\"site-nav\" aria-label=\"Main\">\n",
        html_escape(brand)
    ));
    for (_, title, url) in links {
        header.push_str(&format!(
            "<a href=\"{}\">{}</a>\n\n",
            attr_escape(url),
            html_escape(title)
        ));
    }
    header.push_str(
        "<button class=\"theme-toggle\" type=\"button\" aria-label=\"Toggle color theme\" data-theme-toggle>\n  <span class=\"theme-toggle__track\" aria-hidden=\"true\"><span class=\"theme-toggle__knob\"></span></span>\n</button>\n",
    );
    header.push_str("</nav>\n</header>\n");
    header
}

fn render_footer(site: &Site) -> String {
    let mut footer = String::from("  <footer class=\"site-footer\">\n");
    if !site.config.copyright.is_empty() {
        footer.push_str(&format!(
            "  <span>{}</span>\n",
            html_escape(&site.config.copyright)
        ));
    }
    footer.push_str("  <nav aria-label=\"Footer\">\n");
    for page in site
        .pages
        .iter()
        .filter(|page| page.menu.as_deref() == Some("main"))
    {
        footer.push_str(&format!(
            "    <a href=\"{}\">{}</a>\n",
            attr_escape(&page.url_path),
            html_escape(&page.title)
        ));
    }
    footer.push_str(&format!(
        "    <a href=\"{}\">RSS</a>\n  </nav>\n  </footer>\n",
        attr_escape(&page_url("/index.xml"))
    ));
    footer
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
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
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

            // Repeated headings get -1, -2, ... so every id stays unique on the page.
            let base = heading_id(&heading_text);
            let repeats = used.entry(base.clone()).or_insert(0);
            let id_value = if *repeats == 0 {
                base
            } else {
                format!("{base}-{repeats}")
            };
            *repeats += 1;
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
    let month = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let day = parts.next().unwrap_or_default();
    format!("{day} {}, {year}", month_name(month))
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
        .map_or_else(|| "+0000".to_string(), |value| value.replace(':', ""));
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
    let day = i32::try_from(day).unwrap_or(1);
    let index =
        (year + year / 4 - year / 100 + year / 400 + month_offsets[(month - 1) as usize] + day)
            .rem_euclid(7);
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][usize::try_from(index).unwrap_or(0)]
}

fn month_name(month: u32) -> &'static str {
    match month {
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
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
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
    // Hand-written landing page. /justa/ask/ and /justa/privacy/ stay unlisted.
    push_sitemap_url(&mut sitemap, site, "/justa/", None);
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
        .map_or_else(|| "Thu, 01 Jan 1970 00:00:00 +0000".to_string(), rss_date);

    let mut rss = String::new();
    rss.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
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
        "    <lastBuildDate>{}</lastBuildDate>\n",
        xml_escape(&last_build)
    ));
    rss.push_str(&format!(
        "    <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\" />\n",
        xml_escape(&absolute_url(site, feed_path))
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
            rss.push_str(&format!(
                "      <pubDate>{}</pubDate>\n",
                xml_escape(&rss_date(date))
            ));
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
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            ch if is_illegal_xml_char(ch) => {}
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn is_illegal_xml_char(ch: char) -> bool {
    matches!(ch as u32, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0xFFFE | 0xFFFF)
}

const THEME_BOOTSTRAP: &str = r#"(function(){try{var theme=localStorage.getItem("theme");if(theme==="light"||theme==="dark"){document.documentElement.dataset.theme=theme;}}catch(_){}})();"#;

const THEME_SCRIPT: &str = r#"(function(){var root=document.documentElement;var toggle=document.querySelector("[data-theme-toggle]");if(!toggle){return;}var media=window.matchMedia("(prefers-color-scheme: dark)");function current(){return root.dataset.theme||(media.matches?"dark":"light");}function sync(){toggle.setAttribute("aria-pressed",current()==="dark"?"true":"false");}toggle.addEventListener("click",function(){var next=current()==="dark"?"light":"dark";root.dataset.theme=next;try{localStorage.setItem("theme",next);}catch(_){}sync();});if(media.addEventListener){media.addEventListener("change",sync);}sync();})();"#;

const STYLE: &str = include_str!("style.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_site_config() {
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
        assert_eq!(config.menu_links, Vec::new());
    }

    #[test]
    fn parses_menu_links() {
        let config = parse_site_config(
            r#"
title = "Site"

[[menu.main]]
name = "Justa"
url = "/justa/"
weight = 30

[[menu.main]]
name = "CV"
url = "/cv/"
"#,
        )
        .unwrap();

        assert_eq!(
            config.menu_links,
            vec![
                MenuLink {
                    name: "Justa".to_string(),
                    url: "/justa/".to_string(),
                    weight: Some(30),
                },
                MenuLink {
                    name: "CV".to_string(),
                    url: "/cv/".to_string(),
                    weight: None,
                },
            ]
        );
    }

    #[test]
    fn parses_projects_and_home_copy() {
        let config = parse_site_config(
            r##"
[params]
headline = "Hello"
intro = "A line under it"

[[projects]]
name = "Justa"
url = "/justa/"
background = "#1A3A32" # a hash inside quotes is not a comment

[[projects]]
name = "Music"
url = "https://music.example.com/"
"##,
        )
        .unwrap();

        assert_eq!(config.headline, "Hello");
        assert_eq!(config.intro, "A line under it");
        assert_eq!(config.projects.len(), 2);
        assert_eq!(config.projects[0].name, "Justa");
        assert_eq!(config.projects[0].background, "#1A3A32");
        assert_eq!(config.projects[1].url, "https://music.example.com/");
    }

    #[test]
    fn parses_front_matter() {
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
    fn slugifies_titles() {
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
    fn formats_dates() {
        assert_eq!(date_only("2026-03-09T10:30:00+00:00"), "2026-03-09");
        assert_eq!(display_date("2026-03-09T10:30:00+00:00"), "09 Mar, 2026");
        assert_eq!(
            rss_date("2026-05-20T10:30:00+01:00"),
            "Wed, 20 May 2026 10:30:00 +0100"
        );
    }

    #[test]
    fn renders_markdown() {
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
        assert_eq!(
            xml_escape("António's <DevLog> & \"quotes\""),
            "António&apos;s &lt;DevLog&gt; &amp; &quot;quotes&quot;"
        );
        assert_eq!(xml_escape("keep\tnew\nlines\rok"), "keep\tnew\nlines\rok");
        assert_eq!(xml_escape("drop\u{0008}controls\u{000c}"), "dropcontrols");
    }

    #[test]
    fn rss_and_sitemap_escape_markup_in_titles() {
        let site = Site {
            config: SiteConfig {
                base_url: "https://example.com/".to_string(),
                title: "A & B's <blog>".to_string(),
                copyright: "© \"Jane\" & 'Doe'".to_string(),
                language_code: "en-US".to_string(),
                ..SiteConfig::default()
            },
            pages: Vec::new(),
            blog: Page {
                title: "Blog".to_string(),
                body: String::new(),
                date: None,
                draft: false,
                menu: None,
                weight: None,
                url_path: "/".to_string(),
                kind: PageKind::Section,
            },
            posts: vec![Page {
                title: "Foo & <bar>\u{0008}".to_string(),
                body: "Hello.".to_string(),
                date: Some("2026-01-02T03:04:05+00:00".to_string()),
                draft: false,
                menu: None,
                weight: None,
                url_path: "/foo-and-bar/".to_string(),
                kind: PageKind::Post,
            }],
        };

        let rss = render_rss(&site, FeedScope::Root);
        assert!(rss.contains("<title>Foo &amp; &lt;bar&gt;</title>"));
        assert!(!rss.contains("<bar>"));
        assert!(!rss.contains('\u{0008}'));
        assert!(rss.contains("<copyright>© &quot;Jane&quot; &amp; &apos;Doe&apos;</copyright>"));

        let sitemap = render_sitemap(&site);
        assert!(sitemap.contains("<loc>https://example.com/foo-and-bar/</loc>"));
        assert!(!sitemap.contains("xmlns:xhtml"));
    }

    #[test]
    fn theme_diagram_css_follows_site_toggle() {
        assert!(STYLE.contains(".theme-diagram img.theme-diagram__light"));
        assert!(STYLE.contains(".theme-diagram img.theme-diagram__dark"));
        assert!(
            STYLE.contains(
                ":root:not([data-theme=\"light\"]) .theme-diagram img.theme-diagram__dark"
            )
        );
        assert!(
            STYLE.contains(":root[data-theme=\"dark\"] .theme-diagram img.theme-diagram__dark")
        );
        assert!(
            STYLE.contains(":root[data-theme=\"light\"] .theme-diagram img.theme-diagram__light")
        );
        assert!(STYLE.contains("prefers-color-scheme: dark"));
        let img_rule = STYLE
            .split(".theme-diagram img {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("theme-diagram img rule");
        assert!(
            !img_rule.contains("display"),
            "shared img rule must not set display: {img_rule}"
        );
    }

    #[test]
    fn passes_through_theme_diagram_html() {
        let html = render_markdown(
            "<figure class=\"theme-diagram\">\n  <img class=\"theme-diagram__light\" src=\"/images/a.svg\" alt=\"x\">\n  <img class=\"theme-diagram__dark\" src=\"/images/b.svg\" alt=\"x\">\n</figure>\n",
        );

        assert!(html.contains("<figure class=\"theme-diagram\">"));
        assert!(html.contains("class=\"theme-diagram__light\""));
        assert!(html.contains("class=\"theme-diagram__dark\""));
        assert!(html.contains("src=\"/images/a.svg\""));
        assert!(html.contains("src=\"/images/b.svg\""));
    }

    #[test]
    fn root_relative_urls() {
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
