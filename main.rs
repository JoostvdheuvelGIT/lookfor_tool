use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[command(
    name = "lookfor",
    version,
    about = "A small, fast Rust-powered alternative to `find`."
)]
struct Args {
    /// Root path to start searching from
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Match on file/directory name (substring or regex)
    #[arg(short, long)]
    name: Option<String>,

    /// Treat --name as a regular expression
    #[arg(long)]
    regex: bool,

    /// Match on file extension (e.g. 'rs', 'txt')
    #[arg(short, long)]
    ext: Option<String>,

    /// Maximum directory depth (1 = only the root directory)
    #[arg(long)]
    max_depth: Option<usize>,

    /// Include hidden files and directories
    #[arg(long)]
    hidden: bool,

    /// Filter on type: file, dir, or any
    #[arg(long, value_enum, default_value_t = FileTypeFilter::Any)]
    r#type: FileTypeFilter,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum FileTypeFilter {
    File,
    Dir,
    Any,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn main() {
    let args = Args::parse();

    // Pre-compile regex if requested
    let name_regex = if args.regex {
        args.name
            .as_ref()
            .map(|pattern| Regex::new(pattern).unwrap_or_else(|e| {
                eprintln!("Invalid regex '{}': {e}", pattern);
                std::process::exit(1);
            }))
    } else {
        None
    };

    let mut walker = WalkDir::new(&args.path).follow_links(false);

    if let Some(depth) = args.max_depth {
        walker = walker.max_depth(depth);
    }

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        // Skip hidden if not requested
        if !args.hidden && is_hidden(&entry) {
            continue;
        }

        let file_type = entry.file_type();

        // Type filter
        if !match args.r#type {
            FileTypeFilter::File => file_type.is_file(),
            FileTypeFilter::Dir => file_type.is_dir(),
            FileTypeFilter::Any => true,
        } {
            continue;
        }

        let name = entry.file_name().to_string_lossy();

        // Name / regex filter
        if let Some(pattern) = &args.name {
            let matched = if let Some(re) = &name_regex {
                re.is_match(&name)
            } else {
                name.contains(pattern)
            };

            if !matched {
                continue;
            }
        }

        // Extension filter
        if let Some(ext_filter) = &args.ext {
            let ext_matches = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(ext_filter))
                .unwrap_or(false);

            if !ext_matches {
                continue;
            }
        }

        println!("{}", entry.path().display());
    }
}
