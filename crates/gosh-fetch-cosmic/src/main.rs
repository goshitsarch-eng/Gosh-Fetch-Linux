//! Gosh-Fetch COSMIC - COSMIC desktop frontend for Gosh-Fetch download manager
//!
//! This is a skeleton implementation that will be fleshed out
//! to provide a native COSMIC desktop experience.

mod app;

fn main() -> cosmic::iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Gosh-Fetch COSMIC v2.0.0");

    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(800.0)
            .min_height(600.0),
    );

    cosmic::app::run::<app::App>(settings, ())
}
