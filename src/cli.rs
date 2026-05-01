use clap::{CommandFactory, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process;

fn exit_with_help() -> ! {
    eprintln!("Error: No input provided.");
    eprintln!("\nPipe content to post a snippet, or use a subcommand:");
    eprintln!("  echo 'code' | snip --desc 'note' --lang rust");
    eprintln!("  snip --help    Show all commands");
    eprintln!("  snip get <id>  Get a snippet");
    eprintln!("  snip search    Search snippets");
    process::exit(1);
}

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

    /// Delete a snippet by ID
    Delete {
        /// Snippet ID
        id: i64,
    },

    /// Generate shell completion (add `eval "$(snip complete zsh)"` to .zshrc)
    Complete { shell: clap_complete::Shell },
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    server: String,
    api_key: String,
    username: String,
}

impl Config {
    fn path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".config").join("snip").join("config.json")
    }

    fn load() -> Option<Self> {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn clear() -> anyhow::Result<()> {
        let path = Self::path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

fn prompt_password(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    password.trim().to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load config if exists
    let config = Config::load();

    // Determine server URL
    let server = args
        .server
        .or_else(|| config.as_ref().map(|c| c.server.clone()))
        .or_else(|| std::env::var("SNIP_URL_SERVER").ok())
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    let client = reqwest::Client::new();

    match args.command {
        Some(Command::Login { username, password }) => {
            let password = password.unwrap_or_else(|| prompt_password("Password: "));

            let response = client
                .post(format!("{}/api/login", server))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "username": username,
                    "password": password
                }))
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let api_key = data["api_key"].as_str().unwrap_or("").to_string();

                let config = Config {
                    server: server.clone(),
                    api_key,
                    username: username.clone(),
                };
                config.save()?;

                println!("Logged in as: {}", username);
                println!("Credentials saved to ~/.config/snip/config.json");
            } else {
                eprintln!(
                    "Login failed: {}",
                    response.text().await.unwrap_or_default()
                );
                std::process::exit(1);
            }
            return Ok(());
        }

        Some(Command::Register { username, password }) => {
            let password = password.unwrap_or_else(|| prompt_password("Password: "));

            let response = client
                .post(format!("{}/api/register", server))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "username": username,
                    "password": password
                }))
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let api_key = data["api_key"].as_str().unwrap_or("").to_string();

                let config = Config {
                    server: server.clone(),
                    api_key,
                    username: username.clone(),
                };
                config.save()?;

                println!("Registered and logged in as: {}", username);
                println!("Credentials saved to ~/.config/snip/config.json");
            } else if response.status() == 409 {
                eprintln!("Username already exists");
                std::process::exit(1);
            } else {
                eprintln!(
                    "Registration failed: {}",
                    response.text().await.unwrap_or_default()
                );
                std::process::exit(1);
            }
            return Ok(());
        }

        Some(Command::Logout) => {
            Config::clear()?;
            println!("Logged out. Credentials cleared.");
            return Ok(());
        }

        Some(Command::Whoami) => {
            if let Some(config) = config {
                println!("Server: {}", config.server);
                println!("Username: {}", config.username);
                println!(
                    "API Key: {}...",
                    &config.api_key[..config.api_key.len().min(8)]
                );
            } else {
                println!("Not logged in. Use 'snip login' or 'snip register'.");
            }
            return Ok(());
        }

        _ => {}
    }

    // Get API key from args, config, or env
    let api_key = args
        .api_key
        .or_else(|| config.as_ref().map(|c| c.api_key.clone()))
        .or_else(|| std::env::var("SNIP_API_KEY").ok())
        .expect("API key required. Use 'snip login' or provide --api-key");

    // Save top-level args for default post behavior
    let default_desc = args.desc;
    let default_lang = args.lang;

    // If no command and stdin has content, treat as post
    let command = match args.command {
        Some(cmd) => cmd,
        None => {
            // If stdin is a terminal (not piped), show help
            if io::stdin().is_terminal() {
                exit_with_help();
            }

            // Try to read piped content
            let mut stdin_content = String::new();
            if io::stdin().read_to_string(&mut stdin_content).is_ok()
                && !stdin_content.trim().is_empty()
            {
                // Post the piped content
                return post_snippet(
                    &client,
                    &server,
                    &api_key,
                    default_desc,
                    default_lang,
                    &stdin_content,
                )
                .await;
            }

            // Piped but empty
            exit_with_help();
        }
    };

    match command {
        Command::Post { desc, lang } => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;

            if content.trim().is_empty() {
                eprintln!("Error: No content provided. Pipe content to snip post, e.g.:");
                eprintln!("  cat file.txt | snip post --desc 'my file' --lang rust");
                std::process::exit(1);
            }

            post_snippet(&client, &server, &api_key, desc, lang, &content).await
        }

        Command::Get { id } => {
            let response = client
                .get(format!("{}/api/snippets/{}", server, id))
                .send()
                .await?;

            if response.status().is_success() {
                let snippet: serde_json::Value = response.json().await?;
                println!("ID: {}", snippet["id"].as_i64().unwrap_or(0));
                if let Some(desc) = snippet["description"].as_str() {
                    println!("Description: {}", desc);
                }
                if let Some(lang) = snippet["language"].as_str()
                    && lang != "plaintext"
                {
                    println!("Language: {}", lang);
                }
                println!(
                    "Author: {}",
                    snippet["author"].as_str().unwrap_or("unknown")
                );
                println!("View: {}/s/{}", server, id);
                println!("Raw: {}/raw/{}", server, id);
                println!("---");
                println!("{}", snippet["content"].as_str().unwrap_or(""));
            } else if response.status() == 404 {
                eprintln!("Snippet not found");
                std::process::exit(1);
            } else {
                eprintln!("Error: {}", response.status());
                std::process::exit(1);
            }
            Ok(())
        }

        Command::Search { query, lang, limit } => {
            let mut url = format!(
                "{}/api/search?q={}&limit={}",
                server,
                urlencoding::encode(&query),
                limit
            );
            if let Some(language) = lang {
                url.push_str(&format!("&lang={}", language));
            }

            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let empty_vec = vec![];
                let snippets = data["snippets"].as_array().unwrap_or(&empty_vec);

                if snippets.is_empty() {
                    println!("No snippets found");
                } else {
                    for (i, s) in snippets.iter().enumerate() {
                        if i > 0 {
                            println!("\n---\n");
                        }
                        let id = s["id"].as_i64().unwrap_or(0);
                        println!("ID: {} | {}/s/{}", id, server, id);
                        if let Some(desc) = s["description"].as_str() {
                            println!("Desc: {}", desc);
                        }
                        if let Some(lang) = s["language"].as_str()
                            && lang != "plaintext"
                        {
                            println!("Lang: {}", lang);
                        }
                        println!("Author: {}", s["author"].as_str().unwrap_or("unknown"));
                        let preview: String = s["content"]
                            .as_str()
                            .unwrap_or("")
                            .chars()
                            .take(100)
                            .collect();
                        println!("{}", preview);
                        if s["content"].as_str().unwrap_or("").len() > 100 {
                            println!("... ({} chars)", s["content"].as_str().unwrap_or("").len());
                        }
                    }
                    println!("\n---");
                    println!(
                        "Total: {} | Page: {}/{}",
                        data["total"].as_i64().unwrap_or(0),
                        data["page"].as_i64().unwrap_or(1),
                        (data["total"].as_i64().unwrap_or(0) as f64
                            / data["limit"].as_i64().unwrap_or(10) as f64)
                            .ceil() as i64
                    );
                }
            } else {
                eprintln!("Error: {}", response.status());
                std::process::exit(1);
            }
            Ok(())
        }

        Command::Delete { id } => {
            let response = client
                .delete(format!("{}/api/snippets/{}", server, id))
                .header("X-API-Key", api_key)
                .send()
                .await?;

            if response.status().is_success() {
                println!("Snippet {} deleted successfully", id);
            } else if response.status() == 404 {
                eprintln!("Snippet {} not found", id);
                std::process::exit(1);
            } else if response.status() == 403 {
                eprintln!("Not authorized to delete snippet {}", id);
                std::process::exit(1);
            } else {
                eprintln!("Error: {}", response.status());
                std::process::exit(1);
            }
            Ok(())
        }

        Command::Complete { shell } => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }

        _ => Ok(()), // Handled above
    }
}

async fn post_snippet(
    client: &reqwest::Client,
    server: &str,
    api_key: &str,
    desc: Option<String>,
    lang: Option<String>,
    content: &str,
) -> anyhow::Result<()> {
    let response = client
        .post(format!("{}/api/snippets", server))
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
        let snippet: serde_json::Value = response.json().await?;
        println!(
            "Snippet created! ID: {}",
            snippet["id"].as_i64().unwrap_or(0)
        );
        if let Some(desc) = snippet["description"].as_str() {
            println!("Description: {}", desc);
        }
        println!(
            "View at: {}/s/{}",
            server,
            snippet["id"].as_i64().unwrap_or(0)
        );
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("Error: {} - {}", status, error_text);
        std::process::exit(1);
    }

    Ok(())
}
