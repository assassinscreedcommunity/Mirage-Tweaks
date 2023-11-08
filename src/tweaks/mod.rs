pub mod eject_height;
pub mod sprint_speed;

pub trait Tweak<N>: Send {
    const NAME: &'static str;
    const DEFAULT: N;
    const MIN: N;
    const MAX: N;
    const INTENT: TweakIntent;
    fn enabled(&self) -> bool;
    fn value(&self) -> N;
    fn enable(&mut self);
    fn disable(&mut self);
    fn set_value(&mut self, value: N);
    fn reset_value(&mut self);
}

pub enum TweakIntent {
    Increase,
}
