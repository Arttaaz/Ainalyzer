use libgtp::prelude::*;
use libgtp::model::*;

lazy_static! {
    pub static ref COMMAND_ANALYZE: Command = Command::new(CommandName::KataAnalyze, Some(Args::string("interval 50 ownership true maxmoves 30".to_owned())));
    pub static ref COMMAND_RULES_JAPANESE: Command = Command::new(CommandName::KataSetRules, Some(Args::string("japanese".to_owned())));
}

pub const COMMAND_STOP: Command = Command::new(CommandName::Stop, None);
pub const COMMAND_KOMI: Command = Command::new(CommandName::Komi, Some(Args::float(6.5)));
pub const COMMAND_CLEARBOARD: Command = Command::new(CommandName::ClearBoard, None);
pub const COMMAND_UNDO: Command = Command::new(CommandName::Undo, None);
