use solina::intent::Intent;

#[derive(Default)]
pub struct SolinaMempool {
    // TODO: other data structures might be more suitable for our purposes
    mempool_data: Vec<Intent>,
    mempool_capacity: usize,
}

impl SolinaMempool {
    pub fn new(mempool_capacity: usize) -> Self {
        Self {
            mempool_data: Vec::with_capacity(mempool_capacity),
            mempool_capacity,
        }
    }

    pub fn insert(&mut self, intent: Intent) -> Option<Vec<Intent>> {
        self.mempool_data.push(intent);
        if self.mempool_data.len() == self.mempool_capacity {
            let mempool_data = std::mem::take(&mut self.mempool_data);
            self.mempool_data.clear();
            return Some(mempool_data);
        }
        None
    }
}
