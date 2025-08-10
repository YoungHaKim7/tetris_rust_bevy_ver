use bevy::prelude::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum GameColor {
    #[default] Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Orange,
    Purple,
    Gray,
    Pink,
}

impl From<GameColor> for Color {
    fn from(game_color: GameColor) -> Self {
        match game_color {
            GameColor::Red => Color::rgb_u8(255, 0, 0),
            GameColor::Green => Color::rgb_u8(0, 255, 0),
            GameColor::Blue => Color::rgb_u8(0, 0, 255),
            GameColor::Yellow => Color::rgb_u8(255, 255, 0),
            GameColor::Cyan => Color::rgb_u8(0, 255, 255),
            GameColor::Orange => Color::rgb_u8(255, 165, 0),
            GameColor::Purple => Color::rgb_u8(128, 0, 128),
            GameColor::Gray => Color::rgb_u8(128, 128, 128),
            GameColor::Pink => Color::rgb_u8(255, 192, 203),
        }
    }
}
