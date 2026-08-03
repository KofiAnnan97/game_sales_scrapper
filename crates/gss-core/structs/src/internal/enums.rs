use std::{fmt::Display, write};

// Store IDs
const STEAM_STORE_ID : &str = "steam";
const GOG_STORE_ID : &str = "gog";
const MICROSOFT_STORE_ID : &str = "microsoft_store";

// Store Names (Plain text)
const STEAM_STORE_NAME : &str = "Steam";
const GOG_STORE_NAME : &str = "Good Old Games (GOG)";
const MICROSOFT_STORE_NAME : &str = "Microsoft Store (PC)";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameStore {
    STEAM,
    #[warn(nonstandard_style)]
    GOOD_OLD_GAMES,
    #[warn(nonstandard_style)]
    MICROSOFT_STORE_PC
}

impl GameStore {
    pub fn get_id(self) -> &'static str {
        match self {
            GameStore::STEAM => STEAM_STORE_ID,
            GameStore::GOOD_OLD_GAMES => GOG_STORE_ID,
            GameStore::MICROSOFT_STORE_PC => MICROSOFT_STORE_ID,
        }
    }

    pub fn get_name(self) -> &'static str {
        match self {
            GameStore::STEAM => STEAM_STORE_NAME,
            GameStore::GOOD_OLD_GAMES => GOG_STORE_NAME,
            GameStore::MICROSOFT_STORE_PC => MICROSOFT_STORE_NAME,
        }
    }
}

impl Display for GameStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GamesStore {{ id: {}, name: {} }}", self.get_id(), self.get_name())
    }
}