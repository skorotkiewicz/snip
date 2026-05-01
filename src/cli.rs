use clap::Parser;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "snip")]
#[command(about = "CLI tool for posting snippets to snipped server")]
struct Args {
    #[arg(short, long, help = "Description of the snippet")]
    desc: Option<String>,

    #[arg(
        short,
        long,
        help = "Language (plaintext,bash,c,cpp,csharp,css,go,html,java,javascript,json,kotlin,lua,markdown,php,python,ruby,rust,scala,shell,sql,swift,typescript,yaml,zig)"
    )]
    lang: Option<String>,

    #[arg(short, long, help = "API key (or set SNIP_API_KEY env var)")]
    api_key: Option<String>,

    #[arg(short, long, help = "Server URL (or set SNIP_URL_SERVER env var)")]
    server: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let api_key = args
        .api_key
        .or_else(|| std::env::var("SNIP_API_KEY").ok())
        .expect("API key required. Use --api-key or set SNIP_API_KEY env var");

    let server = args
        .server
        .or_else(|| std::env::var("SNIP_URL_SERVER").ok())
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    // Read from stdin
    let mut content = String::new();
    io::stdin().read_to_string(&mut content)?;

    if content.trim().is_empty() {
        eprintln!("Error: No content provided. Pipe content to snip, e.g.:");
        eprintln!("  cat file.txt | snip --desc 'my file' --lang rust");
        eprintln!("  echo 'hello' | snip --desc 'greeting'");
        std::process::exit(1);
    }

    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/api/snippets", server))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "content": content,
            "description": args.desc,
            "language": args.lang
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
