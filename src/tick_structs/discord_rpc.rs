use discord_game_sdk::{Activity, Discord};
use std::ffi::CString;
use std::time::SystemTime;

pub struct DiscordSDK<'a> {
    discord: Discord<'a>,
}

impl<'a> DiscordSDK<'a> {
    const CLIENT_ID: i64 = 646882722068824074;
    const LARGE_KEY: &'static str = "clockwork_large";

    pub fn new() -> Option<DiscordSDK<'a>> {
        Self::i_new()
            .map_err(|e| error!("DiscordSDK Error on Startup: {}", e))
            .ok()
    }

    fn i_new() -> Result<DiscordSDK<'a>, failure::Error> {
        let mut discord = Discord::new(Self::CLIENT_ID)?;

        let mut activity = Activity::empty();
        activity
            .with_details(CString::new("The Clockwork yet turns...")?)
            .with_state(CString::new("Building the Clockwork")?)
            .with_large_image_key(CString::new(Self::LARGE_KEY)?)
            .with_large_image_tooltip(CString::new("Tick, Tick, Tick...")?)
            .with_start_time(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            );

        discord.update_activity(&activity, |_disc, res| {
            if let Err(e) = res {
                error!("DiscordSDK Error: {}", e);
            }
        });

        Ok(DiscordSDK { discord })
    }

    pub fn run(&mut self) {
        self.discord.empty_event_receivers();

        if let Err(e) = self.discord.run_callbacks() {
            log_once::error_once!("DiscordSDK Error on Callbacks: {}", e);
        }
    }
}
