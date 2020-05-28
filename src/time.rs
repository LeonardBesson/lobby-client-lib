use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

pub const ZERO: Duration = Duration::from_millis(0);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FrameLimitStrategy {
    /// Single yield
    Unlimited,
    /// Yield until next frame (busy)
    Yield,
    /// Sleep until next frame (non busy but no guarantees)
    Sleep,
    /// Sleep until Duration remaining and then yield
    SleepAndYield(Duration),
}

pub struct FrameLimit {
    pub strategy: FrameLimitStrategy,
    pub frame_duration: Duration,
    pub last_frame: Instant,
}

impl FrameLimit {
    pub fn new(strategy: FrameLimitStrategy, max_fps: u32) -> Self {
        let mut s = Self {
            strategy,
            frame_duration: ZERO,
            last_frame: Instant::now(),
        };
        if max_fps == 0 || strategy == FrameLimitStrategy::Unlimited {
            s.strategy = FrameLimitStrategy::Unlimited;
            s.frame_duration = ZERO;
        } else {
            s.frame_duration = Duration::from_secs(1) / max_fps;
        }
        s
    }

    fn do_yield(&self) {
        while self.last_frame + self.frame_duration > Instant::now() {
            yield_now();
        }
    }

    fn do_sleep(&self, stop_on_remaining: Duration) {
        let frame_duration = self.frame_duration - stop_on_remaining;
        loop {
            let elapsed = Instant::now() - self.last_frame;
            if elapsed >= frame_duration {
                break;
            } else {
                sleep(frame_duration - elapsed);
            }
        }
    }

    pub fn run(&mut self) {
        match self.strategy {
            FrameLimitStrategy::Unlimited => yield_now(),
            FrameLimitStrategy::Yield => {
                self.do_yield();
            }
            FrameLimitStrategy::Sleep => {
                self.do_sleep(ZERO);
            }
            FrameLimitStrategy::SleepAndYield(dur) => {
                self.do_sleep(dur);
                self.do_yield();
            }
        }

        self.last_frame = Instant::now();
    }
}

pub struct Time {
    pub started_at: Instant,
    pub current: Instant,
    pub next_wanted_tick: Instant,
    pub tick_count: u128,
    pub total_time: u128,
    pub prev_total_time: u128,
    pub delta: f32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            current: Instant::now(),
            next_wanted_tick: Instant::now(),
            tick_count: 0,
            total_time: 0,
            prev_total_time: 0,
            delta: 0.0,
        }
    }

    pub fn tick(&mut self, frame_limit: &FrameLimit) {
        self.current = Instant::now();
        self.next_wanted_tick += frame_limit.frame_duration;
        self.tick_count += 1;
        self.prev_total_time = self.total_time;
        self.total_time = (self.current - self.started_at).as_micros();
        self.delta = (self.total_time - self.prev_total_time) as f32 / 1_000_000.0
    }
}
