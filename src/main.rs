use std::env;
use std::path::PathBuf;

use aacnsilva_blog::{BuildPaths, generate_site};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> aacnsilva_blog::Result<()> {
    let root = env::current_dir()?;
    let mut paths = BuildPaths {
        config_path: root.join("site.toml"),
        content_dir: root.join("content"),
        static_dir: root.join("static"),
        output_dir: root.join("public"),
    };

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "build" => {}
            "--config" => paths.config_path = next_path(&mut args, "--config")?,
            "--content" => paths.content_dir = next_path(&mut args, "--content")?,
            "--static" => paths.static_dir = next_path(&mut args, "--static")?,
            "--output" | "-o" => paths.output_dir = next_path(&mut args, "--output")?,
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            unknown => return Err(format!("unknown argument: {unknown}").into()),
        }
    }

    generate_site(&paths)
}

fn next_path(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> aacnsilva_blog::Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} requires a path").into())
}

fn print_help() {
    println!(
        "Usage: aacnsilva-blog [build] [--config site.toml] [--content content] [--static static] [--output public]"
    );
}
