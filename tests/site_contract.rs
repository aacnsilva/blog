use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const POSTS: &[(&str, &str, &str)] = &[
    (
        "Agentic programming for Business Central with AL, VS Code, and Copilot",
        "/agentic-programming-for-business-central-with-al-vs-code-and-copilot/",
        "20 May, 2026",
    ),
    (
        "My Business Central dev workflow with Glaze WM and AI agents",
        "/my-business-central-dev-workflow-with-glaze-wm-and-ai-agents/",
        "14 Mar, 2026",
    ),
    (
        "Business Central AL tools with Claude Code and Codex",
        "/business-central-al-tools-with-claude-code-and-codex/",
        "09 Mar, 2026",
    ),
    (
        "Full-text search in Business Central",
        "/full-text-search-in-business-central/",
        "19 Jul, 2025",
    ),
    (
        "Variant Type Inference in Business Central",
        "/variant-type-inference-in-business-central/",
        "27 Jun, 2025",
    ),
    (
        "Keyboard Shortcuts in Business Central",
        "/keyboard-shortcuts-in-business-central/",
        "27 Jun, 2025",
    ),
    (
        "What are events in Business Central",
        "/what-are-events-in-business-central/",
        "29 Mar, 2020",
    ),
];

#[test]
fn rust_generator_satisfies_the_blog_contract() {
    let output = target_dir("contract-rust");
    run(
        env!("CARGO_BIN_EXE_aacnsilva-blog"),
        &["--output", output.to_str().unwrap()],
    );

    assert_site_contract(&output);
    assert_rust_design_contract(&output);
}

#[test]
fn current_hugo_site_satisfies_the_same_contract_when_requested() {
    if std::env::var("VERIFY_HUGO").ok().as_deref() != Some("1") {
        eprintln!("set VERIFY_HUGO=1 to run the parity contract against Hugo");
        return;
    }

    let output = target_dir("contract-hugo");
    run(
        "hugo",
        &[
            "--destination",
            output.to_str().unwrap(),
            "--cleanDestinationDir",
            "--minify=false",
        ],
    );

    assert_site_contract(&output);
}

fn assert_site_contract(root: &Path) {
    assert_required_files_exist(root);
    assert_navigation_and_core_pages(root);
    assert_blog_index(root);
    assert_post_pages(root);
    assert_feeds_and_discovery_files(root);
}

fn assert_required_files_exist(root: &Path) {
    let required = [
        "index.html",
        "blog/index.html",
        "resume/index.html",
        "404.html",
        "robots.txt",
        "sitemap.xml",
        "index.xml",
        "blog/index.xml",
        "favicon.ico",
        "images/favicon-16x16.png",
        "images/favicon-32x32.png",
        "images/share.png",
        "cv/index.html",
        "cv/styles.css",
        "cv/script.js",
        "cv/myPhoto.jpeg",
    ];

    for file in required {
        assert!(
            root.join(file).exists(),
            "{} should exist",
            root.join(file).display()
        );
    }

    for (_, path, _) in POSTS {
        assert!(
            root.join(path.trim_matches('/'))
                .join("index.html")
                .exists()
        );
    }
}

fn assert_navigation_and_core_pages(root: &Path) {
    let home = read(root, "index.html");
    assert!(home.contains("<a href=\"/\">Home</a>"));
    assert!(home.contains("<a href=\"/resume/\">Resume</a>"));
    assert!(home.contains("<a href=\"/blog/\">Blog</a>"));
    assert!(home.contains("Senior Software Engineer"));
    assert!(home.contains("mailto:antoniosilva1017@gmail.com"));

    let resume = read(root, "resume/index.html");
    assert!(resume.contains("Senior Software Engineer"));
    assert!(resume.contains("id=\"about\""));
    assert!(resume.contains("id=\"skills\""));
    assert!(resume.contains("LS Retail"));
}

fn assert_blog_index(root: &Path) {
    let blog = read(root, "blog/index.html");
    assert!(blog.contains("<ul class=\"blog-posts\">"));

    let mut last_position = 0;
    for (title, path, date) in POSTS {
        let position = blog
            .find(title)
            .unwrap_or_else(|| panic!("blog index should contain {title}"));
        assert!(
            position >= last_position,
            "{title} should be in descending date order"
        );
        last_position = position;
        assert!(blog.contains(path), "blog index should link to {path}");
        assert!(blog.contains(date), "blog index should display {date}");
    }
}

fn assert_post_pages(root: &Path) {
    for (title, path, date) in POSTS {
        let html = read(root, &format!("{}index.html", path.trim_start_matches('/')));
        assert!(html.contains(&format!("<h1>{title}</h1>")));
        assert!(html.contains(date));
        assert!(html.contains("<content>"));
        assert!(html.contains("article:published_time"));
    }

    let events = read(root, "what-are-events-in-business-central/index.html");
    assert!(events.contains("<blockquote>"));
    assert!(events.contains("<table>"));
    assert!(events.contains("id=\"explanation\""));
    assert!(events.contains("class=\"language-al\""));
    assert!(
        events.contains("https://aacnsilva.wordpress.com/wp-content/uploads/2020/03/image-4.png")
    );
    assert!(events.contains("Next Post"));

    let search = read(root, "full-text-search-in-business-central/index.html");
    assert!(search.contains("Full-text search input"));
    assert!(search.contains("<table>"));

    let newest = read(
        root,
        "agentic-programming-for-business-central-with-al-vs-code-and-copilot/index.html",
    );
    assert!(newest.contains("Previous Post"));
    assert!(newest.contains("<strike>Next Post"));
}

fn assert_feeds_and_discovery_files(root: &Path) {
    let robots = read(root, "robots.txt");
    assert!(robots.contains("User-Agent: *"));
    assert!(robots.contains("Sitemap: https://aacnsilva.com/sitemap.xml"));

    let sitemap = read(root, "sitemap.xml");
    assert!(sitemap.contains("<urlset"));
    for (_, path, _) in POSTS {
        assert!(sitemap.contains(&format!("https://aacnsilva.com{path}")));
    }

    let rss = read(root, "blog/index.xml");
    assert!(rss.contains("<rss"));
    for (title, path, _) in POSTS {
        assert!(rss.contains(title));
        assert!(rss.contains(&format!("https://aacnsilva.com{path}")));
    }
}

fn assert_rust_design_contract(root: &Path) {
    let home = read(root, "index.html");
    assert!(home.contains("data-theme-toggle"));
    assert!(home.contains("localStorage.getItem(\"theme\")"));
    assert!(home.contains("color-scheme: light"));
    assert!(home.contains("prefers-color-scheme: dark"));
    assert!(home.contains("--surface-color"));
    assert!(home.contains("--accent-color"));
    assert!(home.contains("href=\"/images/favicon-32x32.png\""));
    assert!(!home.contains("href=\"https://aacnsilva.com/"));

    let post = read(
        root,
        "agentic-programming-for-business-central-with-al-vs-code-and-copilot/index.html",
    );
    assert!(post.contains("class=\"post-nav\""));
    assert!(!post.contains("style=\"font-size:0.8em"));
    assert!(
        post.contains("href=\"/my-business-central-dev-workflow-with-glaze-wm-and-ai-agents/\"")
    );
    assert!(!post.contains("<a href=\"https://aacnsilva.com/"));

    let blog = read(root, "blog/index.html");
    assert!(blog.contains(
        "href=\"/agentic-programming-for-business-central-with-al-vs-code-and-copilot/\""
    ));
    assert!(!blog.contains("<a href=\"https://aacnsilva.com/"));
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.join(relative).display()))
}

fn run(program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .status()
        .unwrap_or_else(|error| panic!("failed to run {program}: {error}"));
    assert!(status.success(), "{program} should exit successfully");
}

fn target_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    dir
}
