mod build;
mod config;
mod content;
mod markdown;
mod serve;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

/// The press: a small static-site generator for formaliz.ing and its subdomains.
#[derive(Parser)]
#[command(name = "press")]
struct Cli {
    /// Repository root (holds `sites/` and `theme/`).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build sites into `<out>/<site>/` (all sites if none are named).
    Build {
        #[arg(long = "site")]
        sites: Vec<String>,
        /// Include entries marked `draft = true`.
        #[arg(long)]
        drafts: bool,
        #[arg(long, default_value = "public")]
        out: PathBuf,
    },
    /// Serve one site locally, rebuilding and reloading on change.
    Serve {
        #[arg(long, default_value = "main")]
        site: String,
        #[arg(long, default_value_t = 8000)]
        port: u16,
        /// Hide drafts (they are shown by default while serving).
        #[arg(long)]
        no_drafts: bool,
    },
    /// Start a new draft entry: `press new main notes "On Univalence"`.
    New {
        site: String,
        section: String,
        #[arg(required = true, num_args = 1..)]
        title: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build { sites, drafts, out } => {
            let sites = if sites.is_empty() {
                all_sites(&cli.root)?
            } else {
                sites
            };
            let opts = build::BuildOptions {
                drafts,
                base_url: None,
                live_reload: false,
            };
            for site in sites {
                let dest = cli.root.join(&out).join(&site);
                let n = build::build(&cli.root, &site, &dest, &opts)
                    .with_context(|| format!("building site {site:?}"))?;
                println!("built {site}: {n} entries → {}", dest.display());
            }
        }
        Command::Serve {
            site,
            port,
            no_drafts,
        } => serve::serve(&cli.root, &site, port, !no_drafts)?,
        Command::New {
            site,
            section,
            title,
        } => new_entry(&cli.root, &site, &section, &title.join(" "))?,
    }
    Ok(())
}

fn all_sites(root: &Path) -> Result<Vec<String>> {
    let mut sites = Vec::new();
    for entry in fs::read_dir(root.join("sites")).context("reading sites/")? {
        let path = entry?.path();
        if path.join("site.toml").exists() {
            sites.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    sites.sort();
    Ok(sites)
}

fn new_entry(root: &Path, site: &str, section: &str, title: &str) -> Result<()> {
    let dir = root.join("sites").join(site).join("content").join(section);
    if !dir.join("_index.md").exists() {
        bail!("{} is not a section (no _index.md)", dir.display());
    }
    let path = dir.join(format!("{}.md", markdown::slugify(title)));
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    let today = chrono::Local::now().date_naive();
    let title = toml::Value::String(title.to_string());
    fs::write(
        &path,
        format!("+++\ntitle = {title}\ndate = {today}\ndescription = \"\"\ntags = []\ndraft = true\n+++\n\n"),
    )?;
    println!("{}", path.display());
    Ok(())
}
