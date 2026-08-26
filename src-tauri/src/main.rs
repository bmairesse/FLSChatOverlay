// Sem console em release: o app vive na bandeja.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fls_chat_overlay_lib::run()
}
