use std::collections::HashSet;

/// A simple id generator that wraps around when it reaches the maximum value.
#[derive(Debug, Clone, Default)]
pub struct IdGenerator {
    current: u32,
    already_used: HashSet<u32>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the next id, wrapping around if necessary.
    pub fn next_id(&mut self) -> u32 {
        loop {
            self.current = self.current.wrapping_add(1);
            if !self.already_used.contains(&self.current) {
                self.already_used.insert(self.current);
                return self.current;
            }
        }
    }

    pub fn sync_used_ids(&mut self, used_ids: &[u32]) {
        self.already_used.clear();
        for id in used_ids {
            self.already_used.insert(*id);
        }
    }

    /// Release an id, making it available for reuse. Do not change the current
    /// id to avoid useless checks and exploit wrapping increment.
    pub fn release_id(&mut self, id: u32) {
        self.already_used.remove(&id);
    }
}
