#![windows_subsystem = "windows"]

use crate::control::SliderControl;
use crate::game::Game;
use crate::menu::Menu;
use crate::tweaks::eject_height::EjectHeight;
use crate::tweaks::sprint_speed::SprintSpeed;
use crate::tweaks::Tweak;

mod control;
mod game;
mod menu;
mod message;
mod process;
mod tweaks;

fn main() -> anyhow::Result<()> {
    let result = setup();
    if let Err(error) = &result {
        message::show(error.to_string());
    }
    result
}

fn setup() -> anyhow::Result<()> {
    let mut menu = Menu::new();
    let game = Game::attach()?;
    menu.add_control(SliderControl::new(EjectHeight::setup(&game)?));
    menu.add_control(SliderControl::new(SprintSpeed::setup(&game)?));
    menu.show()
}
