use chrono::Local;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log::LevelFilter;
use std::{env, path::PathBuf};
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = format!("{}/{}.log",
    PathBuf::from(env::var("LOCALAPPDATA").unwrap()).join("com.btjawa.genshinlauncher").join("Logs").to_string_lossy(),
    Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l} {d(%Y-%m-%d %H:%M:%S)}] {m}{n}")))
        .build(logfile)?;
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l} {d(%Y-%m-%d %H:%M:%S)}] {m}{n}")))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .appender(Appender::builder().build("console", Box::new(console_appender)))
        .build(Root::builder().appender("file").appender("console").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;
    Ok(())
}