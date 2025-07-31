use chrono::NaiveDate;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
struct Post {
    title: String,
    date: String,
}

fn main() -> std::io::Result<()> {
    let posts_dir = Path::new("posts");
    let public_dir = Path::new("public");
    let base_template_path = Path::new("templates/base.hbs");
    let header_template_path = Path::new("templates/partials/header.hbs");
    let post_template_path = Path::new("templates/partials/post.hbs");
    let home_template_path = Path::new("templates/partials/home.hbs");

    if !public_dir.exists() {
        fs::create_dir(public_dir)?;
    }

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("base", fs::read_to_string(base_template_path)?)
        .expect("Failed to register base template");
    handlebars
        .register_template_string("header", fs::read_to_string(header_template_path)?)
        .expect("Failed to register layout template");
    handlebars
        .register_template_string("post", fs::read_to_string(post_template_path)?)
        .expect("Failed to register layout template");
    handlebars
        .register_template_string("home", fs::read_to_string(home_template_path)?)
        .expect("Failed to register index template");

    let header_html = handlebars.render("header", &{}).unwrap();

    let mut posts = Vec::new();
    for entry in fs::read_dir(posts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "md" {
            let markdown = fs::read_to_string(&path)?;
            let (front_matter, content) = parse_post_data(&markdown);

            let post: Post = serde_yaml::from_str(&front_matter).unwrap();
            let mut options = comrak::ComrakOptions::default();
            options.extension.table = true;
            let content_html = comrak::markdown_to_html(content, &options);

            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();

            let mut data = std::collections::HashMap::new();
            data.insert("title", post.title.clone());
            data.insert("date", post.date.clone());
            data.insert("content", content_html);

            let post_html = handlebars.render("post", &data).unwrap();
            let mut base_data = std::collections::HashMap::new();
            base_data.insert("content", post_html);
            base_data.insert("header", header_html.clone());
            base_data.insert("post_title", post.title.clone());
            let rendered_html = handlebars.render("base", &base_data).unwrap();
            let mut output_file = File::create(public_dir.join(format!("{}.html", file_name)))?;
            output_file.write_all(rendered_html.as_bytes())?;

            posts.push((post, file_name));
        }
    }

    posts.sort_by(|a, b| {
        let date_a = parse_date(&a.0.date);
        let date_b = parse_date(&b.0.date);
        date_b.cmp(&date_a)
    });

    let mut posts_data = Vec::new();
    for (post, file_name) in posts {
        let mut data = std::collections::HashMap::new();
        data.insert("title".to_string(), post.title);
        data.insert("date".to_string(), post.date);
        data.insert("file_name".to_string(), file_name);
        posts_data.push(data);
    }

    let mut data = std::collections::HashMap::new();
    data.insert(
        "posts".to_string(),
        serde_json::to_value(posts_data).unwrap(),
    );

    let home_html = handlebars.render("home", &data).unwrap();
    let mut base_data = std::collections::HashMap::new();
    base_data.insert("content", &home_html);
    base_data.insert("header", &header_html);
    let index_html = handlebars.render("base", &base_data).unwrap();
    let mut index_file = File::create(public_dir.join("index.html"))?;
    index_file.write_all(index_html.as_bytes())?;

    Ok(())
}

fn parse_date(date_str: &str) -> NaiveDate {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return date;
    }
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
        return date;
    }
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        return date;
    }
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return date;
    }

    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
}

fn parse_post_data(markdown: &str) -> (String, &str) {
    if markdown.starts_with("---") {
        if let Some(end_pos) = markdown[3..].find("---") {
            let front_matter = &markdown[3..end_pos + 3];
            let content = &markdown[end_pos + 6..];
            (front_matter.to_string(), content)
        } else {
            ("".to_string(), markdown)
        }
    } else {
        ("".to_string(), markdown)
    }
}
