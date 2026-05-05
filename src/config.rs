//! Application configuration constants

/// Server configuration
pub mod server {
    /// Default host to bind to (can be overridden via SNIP_HOST env var)
    pub const DEFAULT_HOST: &str = "0.0.0.0";
    /// Default port to listen on (can be overridden via SNIP_PORT env var)
    pub const DEFAULT_PORT: &str = "3000";
    /// Default database URL (can be overridden via DATABASE_URL env var)
    pub const DEFAULT_DATABASE_URL: &str = "sqlite:/data/snip.db";
    /// Default Redis URL (can be overridden via REDIS_URL env var)
    pub const DEFAULT_REDIS_URL: Option<&str> = None;
}

/// Snippet content limits
pub mod limits {
    /// Maximum snippet content length in characters
    pub const MAX_CONTENT_LENGTH: usize = 5000;
    /// Maximum snippet description length in characters
    pub const MAX_DESCRIPTION_LENGTH: usize = 255;
    /// Minimum username length
    pub const MIN_USERNAME_LENGTH: usize = 3;
    /// Maximum username length
    pub const MAX_USERNAME_LENGTH: usize = 32;
    /// Minimum password length
    pub const MIN_PASSWORD_LENGTH: usize = 6;
    /// Maximum comment length in characters
    pub const MAX_COMMENT_LENGTH: usize = 1000;
}

/// Rate limiting configuration
pub mod rate_limit {

    /// Default rate limit window for most endpoints
    pub const DEFAULT_WINDOW_SECS: u64 = 60;
    /// Extended window for registration (hourly)
    pub const REGISTRATION_WINDOW_SECS: u64 = 3600;

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

    /// Duration for view counter flush interval
    pub const VIEW_COUNTER_FLUSH_INTERVAL_SECS: u64 = 60;
}

/*
/// Pagination defaults
pub mod pagination {
    /// Default number of items per page
    pub const DEFAULT_PAGE_SIZE: i64 = 20;
    /// Maximum number of items per page
    pub const MAX_PAGE_SIZE: i64 = 100;
}
*/

/// Database configuration
pub mod database {
    /// Maximum database connections in the pool
    pub const MAX_CONNECTIONS: u32 = 5;
}

/// Bcrypt password hashing cost
/// Uses bcrypt's default cost (currently 12)
pub use bcrypt::DEFAULT_COST;

/// CLI configuration
pub mod cli {
    /// Default server URL for CLI (can be overridden via SNIP_URL_SERVER env var)
    pub const DEFAULT_SERVER_URL: &str = "http://localhost:3000";
    /*
    /// Config file directory name
    pub const CONFIG_DIR: &str = "snip";
    /// Config file name
    pub const CONFIG_FILE: &str = "config.json";
    */
}

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
