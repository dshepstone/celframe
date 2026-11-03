// Stops a console window opening behind the app in Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    celframe_desktop::run();
}
