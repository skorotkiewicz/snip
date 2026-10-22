//! Application configuration constants

/// Default host to bind to (can be overridden via SNIP_HOST env var)
pub const DEFAULT_HOST: &str = "0.0.0.0";
/// Default port to listen on (can be overridden via SNIP_PORT env var)
pub const DEFAULT_PORT: &str = "3000";
/// Default database URL (can be overridden via DATABASE_URL env var)
pub const DEFAULT_DATABASE_URL: &str = "sqlite:snip.db";

/// Maximum snippet content length in characters
pub const MAX_CONTENT_LENGTH: usize = 5000;
/// Maximum snippet description length in characters
pub const MAX_DESCRIPTION_LENGTH: usize = 255;
/// Minimum username length
pub const MIN_USERNAME_LENGTH: usize = 3;
/// Maximum username length
pub const MAX_USERNAME_LENGTH: usize = 32;
/// Minimum password length in Unicode scalar values.
pub const MIN_PASSWORD_LENGTH: usize = 6;
/// Maximum comment length in characters
pub const MAX_COMMENT_LENGTH: usize = 1000;

/// Default rate limit window for most endpoints
pub const DEFAULT_WINDOW_SECS: u32 = 60;
/// Extended window for registration (hourly)
pub const REGISTRATION_WINDOW_SECS: u32 = 3600;

/// Number of requests allowed per window for snippet creation
pub const SNIPPET_CREATE_MAX_REQUESTS: u32 = 10;
/// Number of requests allowed per window for forking
pub const FORK_MAX_REQUESTS: u32 = 10;
/// Number of requests allowed per window for login
pub const LOGIN_MAX_REQUESTS: u32 = 10;
/// Number of requests allowed per window for registration
pub const REGISTER_MAX_REQUESTS: u32 = 5;
/// Number of requests allowed per window for comment creation
pub const COMMENT_CREATE_MAX_REQUESTS: u32 = 30;

/// Maximum database connections in the pool
pub const MAX_CONNECTIONS: u32 = 5;

/// Bcrypt password hashing cost
/// Uses bcrypt's default cost (currently 12)
pub use bcrypt::DEFAULT_COST;

/// Default server URL for CLI (can be overridden via SNIP_URL_SERVER env var)
pub const DEFAULT_SERVER_URL: &str = "http://localhost:3000";

/// Supported programming languages for syntax highlighting
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "plaintext",
    "bash",
    "c",
    "cpp",
    "csharp",
    "css",
    "go",
    "html",
    "java",
    "javascript",
    "json",
    "kotlin",
    "lua",
    "markdown",
    "php",
    "python",
    "ruby",
    "rust",
    "scala",
    "shell",
    "sql",
    "swift",
    "typescript",
    "yaml",
    "zig",
];

/// Maximum password length in UTF-8 bytes, as supported by bcrypt.
pub const MAX_PASSWORD_BYTES: usize = 72;

/// Maximum number of comment levels, including the root comment.
pub const MAX_COMMENT_DEPTH: i64 = 8;
