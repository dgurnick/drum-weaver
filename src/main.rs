use app::App;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::LevelFilter;
mod app;
use app::player::{Player, PlayerCommand, PlayerEvent};

use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::{trigger::size::SizeTrigger, CompoundPolicy};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    init_logging();

    let (player_command_sender, player_command_receiver): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = unbounded();
    let (player_event_sender, player_event_receiver): (Sender<PlayerEvent>, Receiver<PlayerEvent>) = unbounded();
    let mut player = Player::new(player_command_receiver, player_event_sender);
    player.run();

    let mut app = App::new(player_command_sender, player_event_receiver);
    app.run();
}

fn init_logging() {
    // DELETE EXISTING LOGS
    std::fs::remove_file("log/my.log").unwrap_or_default();
    if let Err(_err) = log4rs::init_file("logging_config.yml", Default::default()) {
        // Create the file appender
        let file_appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}")))
            .build(
                "log/my.log",
                Box::new(CompoundPolicy::new(
                    Box::new(SizeTrigger::new(50 * 1024)), // 50 KB
                    Box::new(DeleteRoller::new()),
                )),
            )
            .unwrap();

        // Create the root logger
        let root = Root::builder().appender("file").build(LevelFilter::Trace);

        // Create the configuration with the file appender and root logger
        let config = Config::builder().appender(Appender::builder().build("file", Box::new(file_appender))).build(root).unwrap();

        // Initialize the logger with the configuration
        log4rs::init_config(config).unwrap();
    };
}
