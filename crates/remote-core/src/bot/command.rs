#[derive(Debug, Clone, PartialEq)]
pub enum BotCommand {
    Play,
    Next,
    Previous,
    Player,
}

impl BotCommand {
    /// Parse the raw command token (text after the leading `/`).
    /// Case-insensitive. Returns `None` for unrecognised tokens.
    pub fn parse(text: &str) -> Option<Self> {
        match text.to_lowercase().as_str() {
            "play"              => Some(Self::Play),
            "next"              => Some(Self::Next),
            "previous" | "prev" => Some(Self::Previous),
            "player"            => Some(Self::Player),
            _                   => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_play() {
        assert_eq!(BotCommand::parse("play"), Some(BotCommand::Play));
    }

    #[test]
    fn parse_next() {
        assert_eq!(BotCommand::parse("next"), Some(BotCommand::Next));
    }

    #[test]
    fn parse_previous_long_form() {
        assert_eq!(BotCommand::parse("previous"), Some(BotCommand::Previous));
    }

    #[test]
    fn parse_previous_short_form() {
        assert_eq!(BotCommand::parse("prev"), Some(BotCommand::Previous));
    }

    #[test]
    fn parse_player() {
        assert_eq!(BotCommand::parse("player"), Some(BotCommand::Player));
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(BotCommand::parse("PLAY"), Some(BotCommand::Play));
        assert_eq!(BotCommand::parse("Next"), Some(BotCommand::Next));
        assert_eq!(BotCommand::parse("PLAYER"), Some(BotCommand::Player));
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(BotCommand::parse("start"), None);
        assert_eq!(BotCommand::parse("help"), None);
        assert_eq!(BotCommand::parse(""), None);
    }
}
