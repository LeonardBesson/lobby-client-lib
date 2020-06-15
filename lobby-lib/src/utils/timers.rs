use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq)]
enum TimerResult {
    /// Timer triggered and should continue the be scheduled
    Continue,
    /// Timer is not enabled currently but can be resumed later
    Pause,
    /// Timer has exhausted all its triggers and should be descheduled
    Exhausted,
}

pub struct Timer {
    handle: TimerHandle,
    enabled: bool,
    interval: Duration,
    next_trigger: Instant,
    repeat_count: isize,
    callback: Box<dyn FnMut() -> ()>,
}

impl Timer {
    fn trigger(&mut self) -> TimerResult {
        if !self.enabled {
            return TimerResult::Pause;
        }
        if self.repeat_count < 0 {
            return TimerResult::Exhausted;
        }

        self.repeat_count -= 1;
        self.next_trigger += self.interval;
        (self.callback)();

        if self.repeat_count < 0 {
            self.enabled = false;
            TimerResult::Exhausted
        } else {
            TimerResult::Continue
        }
    }

    pub fn should_trigger(&self) -> bool {
        Instant::now() >= self.next_trigger
    }
}

pub type TimerHandle = usize;

pub struct TimerManager {
    active_timers: HashMap<TimerHandle, Timer>,
    inactive_timers: HashMap<TimerHandle, Timer>,
    current_handle: TimerHandle,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            active_timers: HashMap::new(),
            inactive_timers: HashMap::new(),
            current_handle: 0,
        }
    }

    pub fn schedule_once(
        &mut self,
        at: Instant,
        callback: impl FnMut() -> () + 'static,
    ) -> TimerHandle {
        let handle = self.current_handle;
        let timer = Timer {
            handle,
            enabled: true,
            // Won't be used, doesn't matter
            interval: Duration::from_millis(0),
            next_trigger: at,
            repeat_count: 0,
            callback: Box::new(callback),
        };
        self.active_timers.insert(handle, timer);
        self.current_handle += 1;

        handle
    }

    pub fn schedule(
        &mut self,
        interval: Duration,
        repeat_count: isize,
        callback: impl FnMut() -> () + 'static,
    ) -> TimerHandle {
        let handle = self.current_handle;
        let timer = Timer {
            handle,
            enabled: true,
            interval,
            next_trigger: Instant::now() + interval,
            repeat_count,
            callback: Box::new(callback),
        };
        self.active_timers.insert(handle, timer);
        self.current_handle += 1;

        handle
    }

    pub fn pause(&mut self, timer_handle: TimerHandle) {
        self.active_timers.remove(&timer_handle).map(|mut timer| {
            timer.enabled = false;
            self.inactive_timers.insert(timer_handle, timer)
        });
    }

    pub fn resume(&mut self, timer_handle: TimerHandle) {
        self.inactive_timers.remove(&timer_handle).map(|mut timer| {
            timer.enabled = true;
            self.active_timers.insert(timer_handle, timer)
        });
    }

    pub fn remove(&mut self, timer_handle: TimerHandle) {
        self.active_timers.remove(&timer_handle);
        self.inactive_timers.remove(&timer_handle);
    }

    pub fn tick(&mut self) {
        let mut to_remove = Vec::new();
        let mut to_pause = Vec::new();
        for (&handle, timer) in self.active_timers.iter_mut() {
            if timer.should_trigger() {
                match timer.trigger() {
                    TimerResult::Continue => {}
                    TimerResult::Pause => {
                        println!("Timer {} paused.", handle);
                        to_pause.push(handle);
                    }
                    TimerResult::Exhausted => {
                        println!("Timer {} exhausted, removing.", handle);
                        to_remove.push(handle);
                    }
                }
            }
        }
        for handle in to_remove {
            self.remove(handle);
        }
        for handle in to_pause {
            self.pause(handle);
        }
    }
}
