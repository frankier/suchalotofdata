use std::cmp::Reverse;


pub trait MaxSizePriorityQ<T> where T: Ord {
    fn push(&mut self, item: T);

    fn pop(&mut self) -> Option<T>;
}

pub struct VecMaxSizePriorityQ<T> {
    max_size: u32,
    pub queue: Vec<T>,
}

impl<T> VecMaxSizePriorityQ<T> where T: Ord {
    pub fn new(max_size: u32) -> VecMaxSizePriorityQ<T> {
        VecMaxSizePriorityQ {
            max_size,
            queue: Vec::<T>::with_capacity(max_size as usize),
        }
    }
}

fn reverse<T>(x: &T) -> Reverse<&T> {
    Reverse(x)
}

impl<T> MaxSizePriorityQ<T> for VecMaxSizePriorityQ<T> where T: Ord {
    fn push(&mut self, item: T) {
        let position = match self.queue.binary_search(&item) {
            Ok(position) => position,
            Err(position) => position
        };
        if position == self.max_size as usize {
            return;
        }
        if self.queue.len() >= self.max_size as usize {
            self.queue.pop();
        }
        self.queue.insert(position, item);
    }

    fn pop(&mut self) -> Option<T> {
        self.queue.pop()
    }
}
