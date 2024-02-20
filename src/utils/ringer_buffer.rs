use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Deserialize, Serialize)]
pub struct RingBuffer<T>(VecDeque<T>);

impl<T> RingBuffer<T> {
    pub fn new() -> Self {
        Self(VecDeque::with_capacity(10))
    }

    pub fn push(&mut self, item: T) {
        if self.0.len() == self.0.capacity() {
            self.0.pop_back();
        }
        self.0.push_front(item);
    }
}
