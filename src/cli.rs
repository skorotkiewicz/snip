use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "snip")]
#[command(about = "CLI tool for snipped snippet server", version)]
struct Args {
    #[arg(short, long, help = "API key (or set SNIP_API_KEY env var)")]
    api_key: Option<String>,

    #[arg(short, long, help = "Server URL (or set SNIP_URL_SERVER env var)")]
    server: Option<String>,

    #[arg(short, long, help = "Description of the snippet (for default post)")]
    desc: Option<String>,

    #[arg(
        short,
        long,
        help = "Language (for default post: plaintext,bash,c,cpp,csharp,css,go,html,java,javascript,json,kotlin,lua,markdown,php,python,ruby,rust,scala,shell,sql,swift,typescript,yaml,zig)"
    )]
    lang: Option<String>,

    #[arg(
        short,
        long,
        help = "Config file path (or set SNIP_CONFIG env var, default: ~/.config/snip/config.json)"
    )]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Post a snippet (default if stdin provided)
    Post {
        #[arg(short, long, help = "Description of the snippet")]
        desc: Option<String>,

        #[arg(
            short,
            long,
            help = "Language (plaintext,bash,c,cpp,csharp,css,go,html,java,javascript,json,kotlin,lua,markdown,php,python,ruby,rust,scala,shell,sql,swift,typescript,yaml,zig)"
        )]
        lang: Option<String>,
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
    fn default_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".config").join("snip").join("config.json")
    }

    fn load(path: &Path) -> Option<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
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
    fn new(args: &Args) -> Result<Self> {
        let config_path = args
            .config
            .clone()
            .or_else(|| std::env::var("SNIP_CONFIG").map(PathBuf::from).ok())
            .unwrap_or_else(Config::default_path);

        let config = Config::load(&config_path);

        let server = args
            .server
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                config
                    .as_ref()
                    .map(|c| c.server.clone())
                    .filter(|s| !s.is_empty())
            })
            .or_else(|| {
                std::env::var("SNIP_URL_SERVER")
                    .ok()
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let api_key = args
            .api_key
            .clone()
            .filter(|k| !k.is_empty())
            .or_else(|| {
                config
                    .as_ref()
                    .map(|c| c.api_key.clone())
                    .filter(|k| !k.is_empty())
            })
            .or_else(|| std::env::var("SNIP_API_KEY").ok().filter(|k| !k.is_empty()));

        Ok(Self {
            client: reqwest::Client::new(),
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
            println!("Server: {}", config.server);
            println!("Username: {}", config.username);
            println!(
                "API Key: {}...",
                &config.api_key[..config.api_key.len().min(8)]
            );
        } else {
            println!("Not logged in. Use 'snip login' or 'snip register'.");
        }
        Ok(())
    }

    fn logout(&self) -> Result<()> {
        Config::clear(&self.config_path)?;
        println!(
            "Logged out. Credentials cleared from {}.",
            self.config_path.display()
        );
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
            println!("Logged in as: {}", username);
            println!("Credentials saved to {}", self.config_path.display());
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
            println!("Registered and logged in as: {}", username);
            println!("Credentials saved to {}", self.config_path.display());
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
            println!("Snippet created! ID: {}", snippet.id);
            if let Some(desc) = &snippet.description
                && !desc.is_empty()
            {
                println!("Description: {}", desc);
            }
            println!("View at: {}/s/{}", self.server, snippet.id);
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
                println!("{}", text);
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
        let mut url = format!(
            "{}/api/search?q={}&limit={}",
            self.server,
            urlencoding::encode(&query),
            limit
        );
        if let Some(language) = lang {
            url.push_str(&format!("&lang={}", language));
        }

        let response = self.client.get(&url).send().await?;
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
            println!("Snippet {} deleted successfully", id);
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
                "Starred snippet {}. Total stars: {}",
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
            println!(
                "Unstarred snippet {}. Total stars: {}",
                id, result.total_stars
            );
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
                "Forked snippet {} as {}. Total forks: {}",
                id, result.forked_id, result.total_forks
            );
        } else {
            return Err(api_error(response).await);
        }
        Ok(())
    }
}

// ===== Helpers =====

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
    Ok(password.trim().to_string())
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
    } else if weeks < 4 {
        format!("{}w ago", weeks)
    } else if months < 12 {
        format!("{}mo ago", months)
    } else {
        format!("{}y ago", years)
    }
}

// ===== Main =====

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let ctx = Context::new(&args)?;

    let mut stdin_content: Option<String> = None;

    let command = match args.command {
        Some(cmd) => cmd,
        None => {
            if io::stdin().is_terminal() {
                let mut cmd = Args::command();
                cmd.print_help()?;
                println!();
                std::process::exit(1);
            }

            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;

            if buf.trim().is_empty() {
                bail!("No input provided. Pipe content to post a snippet, or use a subcommand.");
            }

            stdin_content = Some(buf);
            Command::Post {
                desc: args.desc,
                lang: args.lang,
            }
        }
    };

    match command {
        Command::Post { desc, lang } => {
            let content = match stdin_content {
                Some(c) => c,
                None => {
                    let mut c = String::new();
                    io::stdin().read_to_string(&mut c)?;
                    if c.trim().is_empty() {
                        bail!("No content provided. Pipe content to snip post.");
                    }
                    c
                }
            };
            ctx.post(desc, lang, content).await?;
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
        Command::Complete { shell } => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }

    Ok(())
}
