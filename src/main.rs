// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_family = "wasm"))]
    {
        snake_game::start_app()
    }
    #[cfg(target_family = "wasm")]
    {
        Ok(())
    }
}
