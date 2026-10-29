//! Kumkaj business core: money, fiscal packs, the local store, and LAN sync.
//!
//! The Makepad shell lives in the binary. This library is what tests and the
//! shell both call, so the public API is the product — not a pile of helpers
//! that only exist to be compiled.

#[cfg(target_os = "android")]
pub use makepad_widgets;

pub mod barcode_scan;
pub mod booking;
pub mod cod;
pub mod consignments;
pub mod courier_ops_view;
pub mod domain;
pub mod entity_template;
pub mod fiscal;
pub mod labels;
pub mod locale;
pub mod local_servers;
pub mod manifest;
pub mod media;
pub mod menu_images;
pub mod net;
pub mod notify;
pub mod ops_local;
pub mod phone;
pub mod precise;
pub mod print_share;
pub mod receipt_raster;
pub mod rates;
pub mod rider_mode;
pub mod startup;
pub mod store;
pub mod thumbs;
pub mod track;
pub mod uninstall_wipe;

// Android packaging compiles this library as the cdylib. The desktop
// binary's `app_main!` is not in that library, so Activity.onCreate
// finds no native method and the process dies at launch.
#[cfg(target_os = "android")]
#[path = "main.rs"]
mod shell;
