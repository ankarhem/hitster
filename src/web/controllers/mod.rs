//! Web controllers for handling HTTP requests

pub mod view_controller;
pub mod playlist_controller;
pub mod jobs_controller;

pub use view_controller::ViewController;
pub use playlist_controller::PlaylistController;
pub use jobs_controller::JobsController;