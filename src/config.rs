use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavLink {
    pub title: String,
    pub url: String,
}

/// Per-site settings, read from `sites/<name>/site.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteConfig {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
    pub base_url: String,
    #[serde(default)]
    pub author: String,
    /// Year the site began; the masthead's volume number counts from it.
    #[serde(default)]
    pub established: Option<i32>,
    #[serde(default)]
    pub nav: Vec<NavLink>,
    /// Extra class on `<body>`, for per-site theme variations.
    #[serde(default)]
    pub body_class: String,
    #[serde(default = "default_home_recent")]
    pub home_recent: usize,
    /// Passed straight through to MathJax's `tex.macros`.
    #[serde(default)]
    pub mathjax_macros: toml::Table,
}

fn default_home_recent() -> usize {
    12
}

impl SiteConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let src =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let mut config: SiteConfig =
            toml::from_str(&src).with_context(|| format!("parsing {}", path.display()))?;
        config.base_url = config.base_url.trim_end_matches('/').to_string();
        Ok(config)
    }
}
