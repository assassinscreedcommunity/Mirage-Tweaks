use crate::game::Game;
use crate::logger::set_logger;
use crate::menu::{Menu, SliderControl, State, Status};
use crate::tweaks::eject_height::EjectHeightTweak;
use crate::tweaks::sprint_speed::SprintSpeedTweak;
use log::error;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use windows::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

mod config;
mod game;
mod logger;
mod menu;
mod process;
mod tweaks;

fn main() -> ExitCode {
    let _guard = set_logger();

    let game = match Game::attach() {
        Ok(game) => game,
        Err(error) => {
            error!("Failed to attach to game: {error}");
            show_error(error);
            return ExitCode::FAILURE;
        }
    };

    let state = Arc::new(Mutex::new(State {
        controls: Vec::new(),
        status: Status::Loading,
    }));

    start_loading_tweaks(game, &state);

    match Menu::show(state) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            error!("Failed to show menu: {error}");
            show_error(error);
            ExitCode::FAILURE
        }
    }
}

fn start_loading_tweaks(game: Game, state: &Arc<Mutex<State>>) {
    let state = state.clone();
    std::thread::spawn(move || {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let mut eject_height_tweak = EjectHeightTweak::new(&game);
                match &mut eject_height_tweak {
                    Ok(tweak) => tweak.load_config(),
                    Err(error) => error!("Failed to create Eject Height tweak: {error}"),
                }
                state
                    .lock()
                    .unwrap()
                    .controls
                    .push(Box::new(SliderControl::new(eject_height_tweak)));
            });
            scope.spawn(|| {
                let mut sprint_speed_tweak = SprintSpeedTweak::new(&game);
                match &mut sprint_speed_tweak {
                    Ok(tweak) => tweak.load_config(),
                    Err(error) => error!("Failed to create Sprint Speed tweak: {error}"),
                }
                state
                    .lock()
                    .unwrap()
                    .controls
                    .push(Box::new(SliderControl::new(sprint_speed_tweak)));
            });
        });
        hide_console();
        state.lock().unwrap().status = Status::Done;
    });
}

fn hide_console() {
    unsafe {
        let mut processes = [0; 2];
        if GetConsoleProcessList(&mut processes) == 1 {
            let _ = FreeConsole();
        }
    }
}

fn show_error(error: anyhow::Error) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Mirage Tweaks")
        .set_description(error.to_string())
        .show();
}
