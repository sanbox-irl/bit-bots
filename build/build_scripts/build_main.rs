use failure::Error;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
};
use std::fs;

mod imgui_initialization;
mod serialization_util;
mod shader_builder;
mod sprite_packer;

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-env-changed=BUILD_ENABLED");

    initiate_logging()?;
    imgui_initialization::initialize_imgui()?;
    shader_builder::build()?;
    sprite_packer::pack_sprites()?;

    Ok(())
}

fn initiate_logging() -> Result<(), Error> {
    const LOG_LOCATION: &str = "assets/gen/build.log";
    let logfile = FileAppender::builder().build(LOG_LOCATION)?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    // Clear the log
    fs::write(LOG_LOCATION, "")?;

    log4rs::init_config(config)?;

    Ok(())
}
