use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(super) struct ReceptionQueue {
    queue: VecDeque<Instant>,
    threshold: Duration,
}

impl ReceptionQueue {
    pub(super) fn new(threshold: Duration) -> Self {
        Self {
            queue: VecDeque::new(),
            threshold,
        }
    }

    pub(super) fn push(&mut self, instant: Instant) {
        self.queue.push_front(instant);
        // clear the queue of all elements older than the threshold
        while let Some(front) = self.queue.back() {
            if instant.duration_since(*front) > self.threshold {
                self.queue.pop_back();
            } else {
                break;
            }
        }
    }

    pub(super) fn frequency(&self) -> f64 {
        let till = Instant::now();
        let since = till - self.threshold;
        self.queue.iter().take_while(|t| **t > since).count() as f64 / self.threshold.as_secs_f64()
    }

    pub(super) fn time_since_last_reception(&self) -> Option<Duration> {
        self.queue.front().map(|t| t.elapsed())
    }
}
