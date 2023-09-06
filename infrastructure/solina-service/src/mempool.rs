use solina::intent::Intent;

const MEMPOOL_CAPACITY: usize = 5;

pub struct SolinaMempool {
    // TODO: other data structures might be more suitable for our purposes
    mempool_data: Vec<Intent>,
}

impl SolinaMempool {
    pub fn new() -> Self {
        Self {
            mempool_data: Vec::with_capacity(MEMPOOL_CAPACITY),
        }
    }

    pub fn insert(&mut self, intent: Intent) -> Option<Vec<Intent>> {
        self.mempool_data.push(intent);
        if self.mempool_data.len() == MEMPOOL_CAPACITY {
            let mempool_data = std::mem::take(&mut self.mempool_data);
            self.mempool_data.clear();
            return Some(mempool_data);
        }
        None
    }
}
