use super::{UiHandler, Vec2};
use arrayvec::ArrayVec;
use imgui::*;
use std::time::Instant;

pub struct TimeKeeper {
    pub time: Instant,
    pub delta_time: f32,
    pub tick_count: u64,
    pub accumulator: f32,
    pub fps_tracker: FPSTracker,
    pub last_deltas: ArrayVec<[f32; 20]>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            time: Instant::now(),
            accumulator: 0.0,
            delta_time: Self::SIXTIETH,
            tick_count: 0,
            fps_tracker: FPSTracker::default(),
            last_deltas: ArrayVec::new(),
        }
    }

    pub const SIXTIETH: f32 = 1.0 / 60.0;

    pub fn start_frame(&mut self) {
        let new_time = Instant::now();
        let difference = new_time.duration_since(self.time);

        self.delta_time = difference.as_secs() as f32 + (f64::from(difference.subsec_nanos()) / 1.0e9) as f32;
        self.accumulator += self.delta_time;
        self.time = new_time;
        self.tick_count += 1;

        // STUFF TO NOT DO AT RELEASE
        // if self.last_deltas.is_full() {
        //     self.last_deltas.pop();
        // };
        // self.last_deltas.push(self.delta_time);

        // let average: f32 = self.last_deltas.iter().sum::<f32>() / self.last_deltas.len() as f32;
        // if self.delta_time > average * 1.5 {
        //     info!(
        //         "Delta time low! Average is {} -- this Delta is {}",
        //         average, self.delta_time
        //     );
        // } else if self.delta_time < average / 1.5 {
        //     info!(
        //         "Delta time high! Average is {} -- this Delta is {}",
        //         average, self.delta_time
        //     );
        // }

        self.fps_tracker.timer += self.delta_time;
        if self.fps_tracker.timer > 1.0 {
            self.fps_tracker.timer -= 1.0;
            self.fps_tracker.fps = self.tick_count - self.fps_tracker.last_tick;
            self.fps_tracker.last_tick = self.tick_count;
        }
    }

    pub fn create_imgui_window(&self, ui_handler: &mut UiHandler<'_>) -> bool {
        let mut is_opened = true;

        let ui = &mut ui_handler.ui;
        let time_keeper_window = Window::new(im_str!("TimeKeeper"))
            .size(Vec2::new(200.0, 100.0).into(), imgui::Condition::FirstUseEver)
            .opened(&mut is_opened);

        if let Some(window) = time_keeper_window.begin(ui) {
            ui.label_text(im_str!("FPS"), &im_str!("{}", self.fps_tracker.fps));
            ui.label_text(im_str!("Delta Time"), &im_str!("{}", self.delta_time));
            ui.label_text(im_str!("Tick Count"), &im_str!("{}", self.tick_count));
            window.end(ui);
        }

        is_opened
    }
}

#[derive(Debug, Default)]
pub struct FPSTracker {
    pub fps: u64,
    pub timer: f32,
    pub last_tick: u64,
}
