use serde::Deserialize;
use std::fmt;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    /// The TOML source could not be parsed at all.
    ParseError(String),
    /// The `token` key is absent from the file.
    MissingToken,
    /// The `allowed_users` key is absent from the file.
    MissingWhitelist,
    /// The `allowed_users` array is present but contains no entries.
    EmptyWhitelist,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError(msg) => write!(f, "TOML parse error: {msg}"),
            ConfigError::MissingToken => {
                write!(f, "config error: `token` key is missing")
            }
            ConfigError::MissingWhitelist => {
                write!(f, "config error: `allowed_users` key is missing")
            }
            ConfigError::EmptyWhitelist => {
                write!(f, "config error: `allowed_users` must not be empty")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// ── Internal raw deserialization target ───────────────────────────────────────

/// Intermediate struct that accepts optional fields so we can produce distinct
/// errors for missing vs empty values.
#[derive(Deserialize)]
struct RawConfig {
    token: Option<String>,
    allowed_users: Option<Vec<i64>>,
}

// ── Public config ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Telegram bot token obtained from @BotFather.
    pub token: String,
    /// Non-empty list of Telegram user IDs allowed to operate the bot.
    pub allowed_users: Vec<i64>,
}

impl Config {
    /// Parse and validate configuration from a TOML string.
    ///
    /// Returns `Err` for:
    /// - malformed TOML  → `ConfigError::ParseError`
    /// - missing key     → `ConfigError::MissingWhitelist`
    /// - empty array     → `ConfigError::EmptyWhitelist`
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let token = raw.token.ok_or(ConfigError::MissingToken)?;

        let users = match raw.allowed_users {
            None => return Err(ConfigError::MissingWhitelist),
            Some(u) if u.is_empty() => return Err(ConfigError::EmptyWhitelist),
            Some(u) => u,
        };

        Ok(Config { token, allowed_users: users })
    }

    /// Returns `true` if the given Telegram user ID is in the whitelist.
    #[inline]
    pub fn is_allowed(&self, user_id: i64) -> bool {
        self.allowed_users.contains(&user_id)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TOK: &str = r#"token = "test-token""#;

    fn with_token(rest: &str) -> String {
        format!("{TOK}\n{rest}")
    }

    // ── Parsing: valid inputs ──────────────────────────────────────────────────

    #[test]
    fn single_user_is_accepted() {
        let toml = with_token("allowed_users = [111111111]");
        let cfg = Config::from_toml(&toml).expect("should parse");
        assert_eq!(cfg.allowed_users, vec![111111111_i64]);
    }

    #[test]
    fn multiple_users_are_accepted() {
        let toml = with_token("allowed_users = [111111111, 222222222, 333333333]");
        let cfg = Config::from_toml(&toml).expect("should parse");
        assert_eq!(
            cfg.allowed_users,
            vec![111111111_i64, 222222222, 333333333]
        );
    }

    #[test]
    fn extra_keys_in_config_are_ignored() {
        let toml = r#"
            token = "test-token"
            allowed_users = [42]
            bot_name = "my-bot"
            log_level = "info"
        "#;
        let cfg = Config::from_toml(toml).expect("should parse");
        assert_eq!(cfg.allowed_users, vec![42_i64]);
    }

    #[test]
    fn large_user_ids_are_accepted() {
        let toml = with_token("allowed_users = [9999999999]");
        let cfg = Config::from_toml(&toml).expect("should parse");
        assert_eq!(cfg.allowed_users, vec![9_999_999_999_i64]);
    }

    #[test]
    fn negative_user_ids_are_accepted() {
        let toml = with_token("allowed_users = [-100123456789]");
        let cfg = Config::from_toml(&toml).expect("should parse");
        assert_eq!(cfg.allowed_users, vec![-100_123_456_789_i64]);
    }

    #[test]
    fn token_is_stored_in_config() {
        let toml = with_token("allowed_users = [1]");
        let cfg = Config::from_toml(&toml).expect("should parse");
        assert_eq!(cfg.token, "test-token");
    }

    // ── Parsing: invalid TOML syntax ──────────────────────────────────────────

    #[test]
    fn malformed_toml_returns_parse_error() {
        let toml = r#"allowed_users = [this is not valid toml"#;
        let err = Config::from_toml(toml).expect_err("should fail");
        assert!(
            matches!(err, ConfigError::ParseError(_)),
            "expected ParseError, got {err:?}"
        );
    }

    #[test]
    fn wrong_value_type_returns_parse_error() {
        let toml = with_token(r#"allowed_users = ["alice", "bob"]"#);
        let err = Config::from_toml(&toml).expect_err("should fail");
        assert!(
            matches!(err, ConfigError::ParseError(_)),
            "expected ParseError, got {err:?}"
        );
    }

    #[test]
    fn completely_empty_toml_returns_missing_token() {
        let err = Config::from_toml("").expect_err("should fail");
        assert_eq!(err, ConfigError::MissingToken);
    }

    // ── Parsing: missing `token` key ──────────────────────────────────────────

    #[test]
    fn missing_token_key_returns_missing_token_error() {
        let toml = "allowed_users = [1]";
        let err = Config::from_toml(toml).expect_err("should fail");
        assert_eq!(err, ConfigError::MissingToken);
    }

    #[test]
    fn missing_token_error_message_is_descriptive() {
        let err = Config::from_toml("allowed_users = [1]").expect_err("should fail");
        let msg = err.to_string();
        assert!(msg.contains("token"), "error message should mention `token`: {msg}");
    }

    // ── Parsing: missing `allowed_users` key ──────────────────────────────────

    #[test]
    fn missing_allowed_users_key_returns_missing_whitelist() {
        let toml = r#"token = "t""#;
        let err = Config::from_toml(toml).expect_err("should fail");
        assert_eq!(err, ConfigError::MissingWhitelist);
    }

    #[test]
    fn missing_allowed_users_error_message_is_descriptive() {
        let toml = r#"token = "t""#;
        let err = Config::from_toml(toml).expect_err("should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("allowed_users"),
            "error message should mention the missing key: {msg}"
        );
    }

    // ── Parsing: empty `allowed_users` array ─────────────────────────────────

    #[test]
    fn empty_allowed_users_array_returns_empty_whitelist_error() {
        let toml = with_token("allowed_users = []");
        let err = Config::from_toml(&toml).expect_err("should fail");
        assert_eq!(err, ConfigError::EmptyWhitelist);
    }

    #[test]
    fn empty_whitelist_error_message_is_descriptive() {
        let toml = with_token("allowed_users = []");
        let err = Config::from_toml(&toml).expect_err("should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("allowed_users") && msg.contains("empty"),
            "error message should describe the empty-whitelist condition: {msg}"
        );
    }

    // ── is_allowed() helper ───────────────────────────────────────────────────

    #[test]
    fn known_user_is_allowed() {
        let cfg = Config {
            token: "t".to_string(),
            allowed_users: vec![111111111, 222222222],
        };
        assert!(cfg.is_allowed(111111111));
        assert!(cfg.is_allowed(222222222));
    }

    #[test]
    fn unknown_user_is_not_allowed() {
        let cfg = Config {
            token: "t".to_string(),
            allowed_users: vec![111111111],
        };
        assert!(!cfg.is_allowed(999999999));
    }

    #[test]
    fn round_trip_parse_then_check_authorization() {
        let toml = with_token("allowed_users = [111111111]");
        let cfg = Config::from_toml(&toml).expect("should parse");

        // BDD: authorized user passes
        assert!(
            cfg.is_allowed(111111111),
            "authorized user should be allowed"
        );
        // BDD: unauthorized user is denied
        assert!(
            !cfg.is_allowed(999999999),
            "unauthorized user must be silently denied"
        );
    }

    // ── Error trait impl ──────────────────────────────────────────────────────

    #[test]
    fn config_error_implements_std_error() {
        fn accepts_error<E: std::error::Error>(_: E) {}
        let err = Config::from_toml("").expect_err("should fail");
        accepts_error(err);
    }

    #[test]
    fn parse_error_display_contains_toml_context() {
        let err = Config::from_toml("= broken").expect_err("should fail");
        let msg = err.to_string();
        assert!(
            msg.to_lowercase().contains("toml") || msg.contains("parse"),
            "display should hint at TOML: {msg}"
        );
    }
}
