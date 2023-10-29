use anyhow::Result;

use crate::game::Game;

pub mod eject_height;
pub mod sprint_speed;

pub trait Tweak<T> {
    fn setup(game: &Game) -> Result<Self>
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn default_value(&self) -> T;
    fn set_value(&self, value: &T) -> Result<()>;
}

pub trait NumericTweak<T>: Tweak<T> {
    fn min_value(&self) -> T;
    fn max_value(&self) -> T;
    fn clamp(&self) -> Clamp {
        Clamp::None
    }
}

pub enum Clamp {
    None,
    Low,
    #[allow(dead_code)]
    High,
}
