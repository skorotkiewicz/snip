use anyhow::{Context as _, Result, bail};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "snip")]
#[command(
    about = "A little home for your snippets",
    version,
    subcommand_precedence_over_arg = true
)]
struct Args {
    #[command(flatten)]
    connection: ConnectionOptions,

    #[command(flatten)]
    input: PostInput,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Args {
    fn validate_input(&self) -> Result<()> {
        if self.command.is_some()
            && (self.input.file.is_some() || self.input.desc.is_some() || self.input.lang.is_some())
        {
            bail!("Put the file and its options after post: snip post FILE -l markdown");
        }
        Ok(())
    }
}

#[derive(clap::Args, Default)]
struct ConnectionOptions {
    /// API key, or SNIP_API_KEY
    #[arg(short, long, global = true, hide_short_help = true)]
    api_key: Option<String>,

    /// Server URL, or SNIP_URL_SERVER
    #[arg(short, long, global = true, hide_short_help = true)]
    server: Option<String>,

    /// Credentials file, or SNIP_CONFIG (default: ~/.config/snip/config.json)
    #[arg(short, long, global = true, hide_short_help = true)]
    config: Option<PathBuf>,
}

#[derive(clap::Args, Default)]
struct PostInput {
    /// File to share; use - or pipe text to read stdin
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// A short description
    #[arg(short, long)]
    desc: Option<String>,

    /// Syntax language; guessed from the file extension when omitted
    #[arg(short, long)]
    lang: Option<String>,
}

impl PostInput {
    fn use_file_defaults(&mut self) {
        let Some(file) = self.file.as_deref().filter(|path| *path != Path::new("-")) else {
            return;
        };
        if self.desc.is_none() {
            self.desc = file
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
        }
        if self.lang.is_none() {
            self.lang = infer_language(file).map(str::to_owned);
        }
    }
}

fn infer_language(file: &Path) -> Option<&'static str> {
    let extension = file.extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "rs" => "rust",
        "py" | "pyw" => "python",
        "md" | "markdown" => "markdown",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" | "mts" | "cts" => "typescript",
        "sh" | "bash" => "bash",
        "zsh" => "shell",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "css" => "css",
        "go" => "go",
        "htm" | "html" => "html",
        "java" => "java",
        "json" => "json",
        "kt" | "kts" => "kotlin",
        "lua" => "lua",
        "php" => "php",
        "rb" => "ruby",
        "scala" | "sc" => "scala",
        "sql" => "sql",
        "swift" => "swift",
        "yml" | "yaml" => "yaml",
        "zig" => "zig",
        "txt" => "plaintext",
        _ => return None,
    })
}

#[derive(Subcommand)]
enum Command {
    /// Share a file or piped text
    Post {
        #[command(flatten)]
        input: PostInput,
    },

    /// Get a snippet by ID
    Get {
        /// Snippet ID
        id: i64,

        #[arg(short, long, help = "Show only raw content (no metadata)")]
        raw: bool,
    },

    /// Search snippets
    Search {
        /// Search query
        query: String,

        #[arg(short, long, help = "Filter by language")]
        lang: Option<String>,

        #[arg(short = 'n', long, default_value = "10", help = "Limit results")]
        limit: i64,
    },

    /// Login with username and password
    Login {
        /// Username
        username: String,
        /// Password (will prompt if not provided)
        password: Option<String>,
    },

    /// Register a new account
    Register {
        /// Username
        username: String,
        /// Password (will prompt if not provided)
        password: Option<String>,
    },

    /// Logout (clear saved credentials)
    Logout,

    /// Show current user info
    Whoami,

    /// Change password
    ChangePassword {
        /// Old password (will prompt if not provided)
        old_password: Option<String>,
        /// New password (will prompt if not provided)
        new_password: Option<String>,
    },

    /// Delete a snippet by ID
    Delete {
        /// Snippet ID
        id: i64,
    },

    /// Star a snippet by ID
    Star {
        /// Snippet ID
        id: i64,
    },

    /// Unstar a snippet by ID
    Unstar {
        /// Snippet ID
        id: i64,
    },

    /// Fork a snippet by ID (creates a copy in your account)
    Fork {
        /// Snippet ID
        id: i64,
    },

    /// Generate shell completion (add `eval "$(snip complete zsh)"` to .zshrc)
    Complete { shell: clap_complete::Shell },
}

// ===== API Response Types =====

#[derive(Deserialize, Debug)]
struct Snippet {
    id: i64,
    #[serde(default)]
    author: String,
    content: Option<String>,
    description: Option<String>,
    language: Option<String>,
    created_at: Option<String>,
    #[serde(default)]
    views: i64,
    #[serde(default)]
    stars: i64,
    #[serde(default)]
    forks: i64,
    forked_from: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    #[serde(default)]
    snippets: Vec<Snippet>,
    #[serde(default)]
    total: i64,
    #[serde(default)]
    page: i64,
    #[serde(default)]
    limit: i64,
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    api_key: String,
}

#[derive(Deserialize, Debug)]
struct StarResponse {
    #[serde(default)]
    total_stars: i64,
}

#[derive(Deserialize, Debug)]
struct ForkResponse {
    #[serde(default)]
    forked_id: i64,
    #[serde(default)]
    total_forks: i64,
}

// ===== Configuration =====

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Config {
    server: String,
    api_key: String,
    username: String,
}

impl Config {
    fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory; provide --config"))?;
        Ok(home.join(".config").join("snip").join("config.json"))
    }

    fn load(path: &Path) -> Result<Option<Self>> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(Some(serde_json::from_str(&content)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = (|| -> Result<()> {
            let mut file = options.open(&temporary)?;
            file.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&temporary, path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }

    fn clear(path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

// ===== Context =====

struct Context {
    client: reqwest::Client,
    config_path: PathBuf,
    config: Option<Config>,
    server: String,
    api_key: Option<String>,
}

impl Context {
    fn new(args: &ConnectionOptions) -> Result<Self> {
        let config_path = match args
            .config
            .clone()
            .or_else(|| std::env::var_os("SNIP_CONFIG").map(PathBuf::from))
        {
            Some(path) => path,
            None => Config::default_path()?,
        };

        let mut config = Config::load(&config_path)?;

        let server = args
            .server
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                std::env::var("SNIP_URL_SERVER")
                    .ok()
                    .filter(|value| !value.is_empty())
            })
            .or_else(|| config.as_ref().map(|saved| saved.server.clone()))
            .unwrap_or_else(|| snip::config::DEFAULT_SERVER_URL.to_owned());
        let server = normalize_server(&server)?;
        // Credentials are scoped to the saved server, including any base path.
        config = config
            .filter(|saved| normalize_server(&saved.server).is_ok_and(|saved| saved == server));
        let api_key = args
            .api_key
            .clone()
            .filter(|key| !key.is_empty())
            .or_else(|| {
                std::env::var("SNIP_API_KEY")
                    .ok()
                    .filter(|key| !key.is_empty())
            })
            .or_else(|| {
                config
                    .as_ref()
                    .map(|saved| saved.api_key.clone())
                    .filter(|key| !key.is_empty())
            });
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            config_path,
            config,
            server,
            api_key,
        })
    }

    fn require_api_key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or_else(|| {
            anyhow::anyhow!("API key required. Use 'snip login' or provide --api-key")
        })
    }

    fn whoami(&self) -> Result<()> {
        if let Some(config) = &self.config {
            println!("{} @ {}", config.username, self.server);
        } else if self.api_key.is_some() {
            println!("API key ready @ {}", self.server);
        } else {
            println!("Not signed in. Your first snip starts with: snip login USERNAME");
        }
        Ok(())
    }

    fn logout(&self) -> Result<()> {
        Config::clear(&self.config_path)?;
        println!("Saved login cleared. Catch you next snip.");
        Ok(())
    }

    async fn login(&self, username: String, password: String) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/api/login", self.server))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;

        if response.status().is_success() {
            let data: AuthResponse = response.json().await?;
            let config = Config {
                server: self.server.clone(),
                api_key: data.api_key,
                username: username.clone(),
            };
            config.save(&self.config_path)?;
            println!("You're in, {}. Ready, set, snip.", username);
            terminal_note("Try: snip FILE -l markdown");
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn register(&self, username: String, password: String) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/api/register", self.server))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;

        if response.status().is_success() {
            let data: AuthResponse = response.json().await?;
            let config = Config {
                server: self.server.clone(),
                api_key: data.api_key,
                username: username.clone(),
            };
            config.save(&self.config_path)?;
            println!("Welcome aboard, {}. Your first snip awaits.", username);
            terminal_note("Try: snip FILE -l markdown");
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn post(
        &self,
        desc: Option<String>,
        lang: Option<String>,
        content: String,
    ) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .post(format!("{}/api/snippets", self.server))
            .header("X-API-Key", api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "content": content,
                "description": desc,
                "language": lang
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let snippet: Snippet = response.json().await?;
            terminal_note(&format!("✂ snip! #{} is out in the world.", snippet.id));
            println!("{}/s/{}", self.server, snippet.id);
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn get(&self, id: i64, raw: bool) -> Result<()> {
        if raw {
            let response = self
                .client
                .get(format!("{}/raw/{}", self.server, id))
                .send()
                .await?;
            if response.status().is_success() {
                let text = response.text().await?;
                print!("{}", text);
            } else {
                return Err(api_error(response).await);
            }
        } else {
            let response = self
                .client
                .get(format!("{}/api/snippets/{}", self.server, id))
                .send()
                .await?;
            if response.status().is_success() {
                let s: Snippet = response.json().await?;

                let lang_part = match s.language.as_deref() {
                    Some(l) if l != "plaintext" && !l.is_empty() => format!(" ({})", l),
                    _ => String::new(),
                };
                let desc_part = match s.description.as_deref() {
                    Some(d) if !d.is_empty() => d,
                    _ => "untitled",
                };
                let fork_part = if let Some(orig_id) = s.forked_from {
                    format!(" | forked from /s/{}", orig_id)
                } else if s.forks > 0 {
                    format!(" | {} fork{}", s.forks, if s.forks == 1 { "" } else { "s" })
                } else {
                    String::new()
                };

                println!(
                    "#{} | {}{} | {} | {} views | {} stars{}",
                    s.id, desc_part, lang_part, s.author, s.views, s.stars, fork_part
                );
                println!();
                if let Some(content) = &s.content {
                    println!("{}", content);
                }
            } else {
                return Err(api_error(response).await);
            }
        }
        Ok(())
    }

    async fn search(&self, query: String, lang: Option<String>, limit: i64) -> Result<()> {
        let mut params = vec![("q", query), ("limit", limit.to_string())];
        if let Some(language) = lang {
            params.push(("lang", language));
        }
        let response = self
            .client
            .get(format!("{}/api/search", self.server))
            .query(&params)
            .send()
            .await?;
        if response.status().is_success() {
            let data: SearchResult = response.json().await?;
            if data.snippets.is_empty() {
                println!("No snippets found");
            } else {
                for s in &data.snippets {
                    let desc_fmt = match s.description.as_deref() {
                        Some(d) if !d.is_empty() => {
                            if d.chars().count() > 30 {
                                let truncated: String = d.chars().take(27).collect();
                                format!("{}...", truncated)
                            } else {
                                d.to_string()
                            }
                        }
                        _ => "(no description)".to_string(),
                    };

                    let time_ago = format_time_ago(s.created_at.as_deref().unwrap_or(""));

                    let fork_info = if let Some(orig_id) = s.forked_from {
                        format!("forked from #{}", orig_id)
                    } else if s.forks > 0 {
                        format!("{} forks · {}", s.forks, time_ago)
                    } else {
                        time_ago
                    };

                    println!(
                        "#{:<5} {:<32} {:<15} {:<10} {}",
                        s.id,
                        desc_fmt,
                        s.author,
                        format!("{}v/{}s", s.views, s.stars),
                        fork_info
                    );
                }
                println!();
                let total_pages = (data.total as f64 / data.limit as f64).ceil() as i64;
                println!(
                    "Total: {} | Page: {}/{}",
                    data.total, data.page, total_pages
                );
            }
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn change_password(&self, old_password: String, new_password: String) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .post(format!("{}/api/change-password", self.server))
            .header("X-API-Key", api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "old_password": old_password,
                "new_password": new_password
            }))
            .send()
            .await?;

        if response.status().is_success() {
            println!("Password changed successfully");
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .delete(format!("{}/api/snippets/{}", self.server, id))
            .header("X-API-Key", api_key)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Snip #{} is gone. Room for the next idea.", id);
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn star(&self, id: i64) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .post(format!("{}/api/snippets/{}/star", self.server, id))
            .header("X-API-Key", api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let result: StarResponse = response.json().await?;
            println!(
                "A little appreciation for #{}. Stars: {}",
                id, result.total_stars
            );
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn unstar(&self, id: i64) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .delete(format!("{}/api/snippets/{}/star", self.server, id))
            .header("X-API-Key", api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let result: StarResponse = response.json().await?;
            println!("Unstarred #{}. Stars: {}", id, result.total_stars);
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }

    async fn fork(&self, id: i64) -> Result<()> {
        let api_key = self.require_api_key()?;
        let response = self
            .client
            .post(format!("{}/api/snippets/{}/fork", self.server, id))
            .header("X-API-Key", api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let result: ForkResponse = response.json().await?;
            println!(
                "A new branch for #{}: #{}. Forks: {}",
                id, result.forked_id, result.total_forks
            );
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }
}

// ===== Helpers =====

fn read_snippet(file: Option<&Path>, stdin: impl Read) -> Result<String> {
    // Every Unicode scalar value takes at most four UTF-8 bytes.
    let max_bytes = snip::config::MAX_CONTENT_LENGTH * 4;
    let mut bytes = Vec::new();
    match file.filter(|path| *path != Path::new("-")) {
        Some(path) => {
            std::fs::File::open(path)
                .with_context(|| format!("Cannot open {}", path.display()))?
                .take(max_bytes as u64 + 1)
                .read_to_end(&mut bytes)?;
        }
        None => {
            stdin.take(max_bytes as u64 + 1).read_to_end(&mut bytes)?;
        }
    }
    if bytes.len() > max_bytes {
        bail!(
            "That snip is too big. The limit is {} characters.",
            snip::config::MAX_CONTENT_LENGTH
        );
    }
    let content = String::from_utf8(bytes).context("Snippets need UTF-8 text")?;
    if content.chars().count() > snip::config::MAX_CONTENT_LENGTH {
        bail!(
            "That snip is too big. The limit is {} characters.",
            snip::config::MAX_CONTENT_LENGTH
        );
    }
    if content.trim().is_empty() {
        bail!("Nothing to snip yet. Try: snip LICENSE -l markdown");
    }
    Ok(content)
}

fn normalize_server(server: &str) -> Result<String> {
    let url = reqwest::Url::parse(server)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        bail!("Server must be an HTTP(S) URL without credentials, a query, or a fragment");
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

async fn api_error(response: reqwest::Response) -> anyhow::Error {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let msg = if text.is_empty() {
        match status {
            StatusCode::UNAUTHORIZED => "Unauthorized. Please login first.".to_string(),
            StatusCode::FORBIDDEN => "Forbidden. You don't have permission.".to_string(),
            StatusCode::NOT_FOUND => "Not found.".to_string(),
            StatusCode::TOO_MANY_REQUESTS => "Rate limit exceeded.".to_string(),
            _ => status
                .canonical_reason()
                .unwrap_or("Unknown error")
                .to_string(),
        }
    } else {
        text
    };
    anyhow::anyhow!("{}: {}", status, msg)
}

fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    Ok(password.trim_end_matches(['\r', '\n']).to_owned())
}

fn format_time_ago(date_str: &str) -> String {
    if date_str.is_empty() {
        return "unknown".to_string();
    }

    let Ok(created) = chrono::DateTime::parse_from_rfc3339(date_str) else {
        return "unknown".to_string();
    };

    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(created.with_timezone(&chrono::Utc));

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    if seconds < 60 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{}m ago", minutes)
    } else if hours < 24 {
        format!("{}h ago", hours)
    } else if days < 7 {
        format!("{}d ago", days)
    } else if days < 30 {
        format!("{}w ago", weeks)
    } else if months < 12 {
        format!("{}mo ago", months)
    } else {
        format!("{}y ago", years)
    }
}

#[derive(Clone, Copy)]
enum HelpStyle {
    Brief,
    Detailed,
}

fn help_command(context: Option<&Context>, style: HelpStyle) -> clap::Command {
    let signed_in = context.is_some_and(|context| context.api_key.is_some());
    let status = match context {
        Some(context) if signed_in => match &context.config {
            Some(config) => format!("{} @ {} • ready to snip", config.username, context.server),
            None => format!("API key ready @ {}", context.server),
        },
        _ => "Your next little idea belongs here. Sign in to share it.".to_owned(),
    };
    Args::command()
        .disable_help_subcommand(true)
        .before_help(format!("✂ snip\n{status}"))
        .after_help(
            "Try a snip:
  snip main.rs
  snip LICENSE -l markdown
  cat main.rs | snip -l rust

More tools: snip --help",
        )
        .mut_subcommands(|command| {
            let hidden = match command.get_name() {
                "login" | "register" => signed_in,
                "get" | "search" => false,
                "post" | "delete" | "logout" | "whoami" => !signed_in,
                _ => !signed_in || matches!(style, HelpStyle::Brief),
            };
            command.hide(hidden)
        })
}

// Read connection flags even when Clap will exit for help or a missing command argument.
// The real parser below still validates every argument and renders all errors.
fn argument_preview(arguments: &[std::ffi::OsString]) -> Result<clap::ArgMatches, clap::Error> {
    fn without_help(command: clap::Command) -> clap::Command {
        command
            .disable_help_flag(true)
            .disable_version_flag(true)
            .disable_help_subcommand(true)
            .mut_subcommands(without_help)
    }
    without_help(Args::command())
        .arg(
            clap::Arg::new("short_help")
                .short('h')
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("long_help")
                .long("help")
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("show_version")
                .short('V')
                .long("version")
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
        .ignore_errors(true)
        .try_get_matches_from(arguments)
}

fn terminal_note(message: &str) {
    if io::stderr().is_terminal() {
        eprintln!("{message}");
    }
}

// ===== Main =====

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("snip: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args_os().collect();
    let preview = argument_preview(&arguments)?;
    let connection = ConnectionOptions::from_arg_matches(&preview)?;
    let context = Context::new(&connection);
    let style = if preview.get_flag("long_help") {
        HelpStyle::Detailed
    } else {
        HelpStyle::Brief
    };
    let mut help = help_command(context.as_ref().ok(), style);
    let matches = help.clone().get_matches_from(arguments);
    let args = Args::from_arg_matches(&matches)?;
    args.validate_input()?;
    if let Some(Command::Complete { shell }) = &args.command {
        let mut command = Args::command();
        let name = command.get_name().to_owned();
        clap_complete::generate(*shell, &mut command, name, &mut io::stdout());
        return Ok(());
    }
    let ctx = context?;

    let command = match args.command {
        Some(command) => command,
        None => {
            if args.input.file.is_none()
                && args.input.desc.is_none()
                && args.input.lang.is_none()
                && io::stdin().is_terminal()
            {
                help.print_help()?;
                println!();
                return Ok(());
            }
            Command::Post { input: args.input }
        }
    };

    match command {
        Command::Post { mut input } => {
            ctx.require_api_key()?;
            input.use_file_defaults();
            if input.file.is_none() && io::stdin().is_terminal() {
                bail!("Give me a file or pipe some text: snip LICENSE -l markdown");
            }
            let content = read_snippet(input.file.as_deref(), io::stdin().lock())?;
            ctx.post(input.desc, input.lang, content).await?;
        }
        Command::Get { id, raw } => ctx.get(id, raw).await?,
        Command::Search { query, lang, limit } => ctx.search(query, lang, limit).await?,
        Command::Login { username, password } => {
            let password = match password {
                Some(p) => p,
                None => prompt_password("Password: ")?,
            };
            ctx.login(username, password).await?;
        }
        Command::Register { username, password } => {
            let password = match password {
                Some(p) => p,
                None => prompt_password("Password: ")?,
            };
            ctx.register(username, password).await?;
        }
        Command::Logout => ctx.logout()?,
        Command::Whoami => ctx.whoami()?,
        Command::ChangePassword {
            old_password,
            new_password,
        } => {
            let old_password = match old_password {
                Some(p) => p,
                None => prompt_password("Old password: ")?,
            };
            let new_password = match new_password {
                Some(p) => p,
                None => prompt_password("New password: ")?,
            };
            ctx.change_password(old_password, new_password).await?;
        }
        Command::Delete { id } => ctx.delete(id).await?,
        Command::Star { id } => ctx.star(id).await?,
        Command::Unstar { id } => ctx.unstar(id).await?,
        Command::Fork { id } => ctx.fork(id).await?,
        Command::Complete { .. } => unreachable!("completions return before loading configuration"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_path() -> PathBuf {
        std::env::temp_dir().join(format!("snip-cli-{}.json", uuid::Uuid::new_v4()))
    }

    #[test]
    fn explicit_post_accepts_a_file_after_options() {
        let parsed =
            Args::try_parse_from(["snip", "post", "-d", "", "-l", "markdown", "--", "LICENSE"]);
        assert!(
            parsed.is_ok(),
            "posting a file must parse: {}",
            parsed
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default()
        );
    }

    #[test]
    fn default_post_accepts_a_file() {
        assert!(
            Args::try_parse_from(["snip", "LICENSE", "-l", "markdown"]).is_ok(),
            "the default command must accept a file"
        );
    }

    #[test]
    fn file_input_and_stdin_share_validation() {
        let path = test_config_path();
        std::fs::write(&path, "  hello \n").expect("create input fixture");
        assert_eq!(
            read_snippet(Some(&path), &b"ignored"[..]).expect("read file"),
            "  hello \n"
        );
        assert_eq!(
            read_snippet(None, &b"piped text"[..]).expect("read pipe"),
            "piped text"
        );
        assert_eq!(
            read_snippet(Some(Path::new("-")), &b"stdin"[..]).expect("read explicit stdin"),
            "stdin"
        );
        assert!(
            read_snippet(None, &b"  \n"[..]).is_err(),
            "empty content must fail before an HTTP request"
        );
        assert!(
            read_snippet(None, &b"\xff"[..]).is_err(),
            "binary content must fail clearly"
        );
        let unicode = "界".repeat(snip::config::MAX_CONTENT_LENGTH);
        assert_eq!(
            read_snippet(None, unicode.as_bytes()).expect("accept Unicode limit"),
            unicode
        );
        assert!(
            read_snippet(
                None,
                "a".repeat(snip::config::MAX_CONTENT_LENGTH + 1).as_bytes()
            )
            .is_err(),
            "reject oversized text"
        );
        Config::clear(&path).expect("remove input fixture");
        assert!(
            read_snippet(Some(&path), &b""[..]).is_err(),
            "missing files must fail clearly"
        );
    }

    #[test]
    fn file_defaults_save_typing_and_preserve_overrides() {
        let mut input = PostInput {
            file: Some("src/main.rs".into()),
            ..PostInput::default()
        };
        input.use_file_defaults();
        assert_eq!(input.desc.as_deref(), Some("main.rs"));
        assert_eq!(input.lang.as_deref(), Some("rust"));
        let mut input = PostInput {
            file: Some("README.md".into()),
            desc: Some(String::new()),
            lang: Some("plaintext".into()),
        };
        input.use_file_defaults();
        assert_eq!(
            input.desc.as_deref(),
            Some(""),
            "an explicit empty description stays empty"
        );
        assert_eq!(input.lang.as_deref(), Some("plaintext"));
        let mut input = PostInput {
            file: Some("-".into()),
            ..PostInput::default()
        };
        input.use_file_defaults();
        assert!(
            input.desc.is_none() && input.lang.is_none(),
            "stdin must not inherit file metadata"
        );
        assert_eq!(infer_language(Path::new("README.MD")), Some("markdown"));
        assert_eq!(infer_language(Path::new("LICENSE")), None);
    }

    #[test]
    fn parsing_does_not_mix_file_input_with_other_commands() {
        let mixed =
            Args::try_parse_from(["snip", "LICENSE", "get", "1"]).expect("parse mixed input");
        assert!(
            mixed.validate_input().is_err(),
            "a file must never be silently ignored"
        );
        assert!(
            Args::try_parse_from(["snip", "get", "1", "--server", "https://example.com"]).is_ok(),
            "connection flags must work after commands"
        );
        let configured = Args::try_parse_from(["snip", "--config", "custom.json", "get", "1"])
            .expect("connection flags may precede commands");
        assert!(
            matches!(configured.command, Some(Command::Get { id: 1, .. })),
            "connection flags must not consume the command"
        );
        assert!(
            configured.validate_input().is_ok(),
            "connection flags are independent of snippet input"
        );
        let input = Args::try_parse_from(["snip", "--", "login"])
            .expect("escape a filename matching a command");
        assert_eq!(input.input.file.as_deref(), Some(Path::new("login")));
    }

    #[test]
    fn help_follows_the_login_state_and_keeps_account_switching_available() {
        let guest = help_command(None, HelpStyle::Brief)
            .render_help()
            .to_string();
        assert!(
            guest.contains("  login ") && guest.contains("  register "),
            "guests need sign-in commands"
        );
        assert!(
            !guest.contains("  logout ") && !guest.contains("  change-password "),
            "guests do not need account management"
        );
        let context = Context {
            client: reqwest::Client::new(),
            config_path: test_config_path(),
            config: Some(Config {
                username: "alice".into(),
                server: "https://example.com".into(),
                api_key: "key".into(),
            }),
            server: "https://example.com".into(),
            api_key: Some("key".into()),
        };
        let brief = help_command(Some(&context), HelpStyle::Brief)
            .render_help()
            .to_string();
        assert!(
            brief.contains("alice @ https://example.com"),
            "show the active account and server"
        );
        assert!(
            !brief.contains("  login ") && !brief.contains("  register "),
            "signed-in help should omit sign-in commands"
        );
        assert!(
            brief.contains("  post ") && brief.contains("  logout "),
            "show useful signed-in actions"
        );
        assert!(
            !brief.contains("  complete ") && !brief.contains("--api-key"),
            "short help should stay short"
        );
        let detailed = help_command(Some(&context), HelpStyle::Detailed)
            .render_long_help()
            .to_string();
        assert!(
            detailed.contains("  complete ") && detailed.contains("--api-key"),
            "full help exposes advanced tools"
        );
        assert!(
            help_command(Some(&context), HelpStyle::Brief)
                .try_get_matches_from(["snip", "login", "bob"])
                .is_ok(),
            "hidden login remains available to switch accounts"
        );
    }

    #[test]
    fn help_preview_reads_connection_flags_after_help_and_subcommands() {
        let arguments: Vec<_> = [
            "snip",
            "get",
            "--help",
            "--server",
            "https://example.com",
            "--config",
            "custom.json",
        ]
        .map(std::ffi::OsString::from)
        .into();
        let preview = argument_preview(&arguments).expect("preview incomplete command help");
        let connection =
            ConnectionOptions::from_arg_matches(&preview).expect("read connection flags");
        assert_eq!(connection.server.as_deref(), Some("https://example.com"));
        assert_eq!(connection.config.as_deref(), Some(Path::new("custom.json")));
        assert!(
            preview.get_flag("long_help"),
            "retain the request for full help"
        );
    }

    #[test]
    fn server_urls_are_normalized_and_validated() {
        assert_eq!(
            normalize_server("HTTP://EXAMPLE.COM:80/base/").expect("valid server"),
            "http://example.com/base"
        );
        for url in [
            "file:///tmp/test",
            "https://user:password@example.com",
            "https://example.com?key=secret",
            "https://example.com#fragment",
        ] {
            assert!(
                normalize_server(url).is_err(),
                "reject invalid server URL: {url}"
            );
        }
    }

    #[test]
    fn saved_credentials_are_scoped_to_their_server() {
        let path = test_config_path();
        Config {
            server: "http://localhost:3000/".into(),
            api_key: "saved-secret".into(),
            username: "alice".into(),
        }
        .save(&path)
        .expect("save test config");
        let args = Args::try_parse_from([
            "snip",
            "--config",
            path.to_str().expect("UTF-8 test path"),
            "--server",
            "http://localhost:3001",
        ])
        .expect("parse arguments");
        let context = Context::new(&args.connection).expect("create context");
        assert!(
            context.config.is_none(),
            "another server must not inherit saved credentials"
        );
        assert_ne!(context.api_key.as_deref(), Some("saved-secret"));
        Config::clear(&path).expect("clean up test config");
    }

    #[test]
    fn saved_credentials_replace_existing_files_and_report_corruption() {
        let path = test_config_path();
        std::fs::write(&path, "invalid JSON").expect("write corrupt config");
        assert!(
            Config::load(&path).is_err(),
            "invalid configuration must be reported"
        );
        Config {
            server: "https://example.com".into(),
            api_key: "secret".into(),
            username: "alice".into(),
        }
        .save(&path)
        .expect("replace test config");
        assert_eq!(
            Config::load(&path)
                .expect("load saved config")
                .expect("config exists")
                .api_key,
            "secret"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("config metadata")
                .permissions()
                .mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "credentials must be readable only by their owner"
            );
        }
        Config::clear(&path).expect("clean up test config");
    }
}
