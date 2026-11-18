//! Android frontend for the desktop hardware model and C recompiler.
mod compiler;
mod engine;
mod jni;
mod save_backups;
// Presentation-only color models mirrored from the desktop screen crate.
mod color;
mod profile;
mod lut;

mod audio;
mod normal_audio;
mod ui_layout;
mod touch;
mod scheduling;
