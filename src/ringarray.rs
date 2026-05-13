/// Struct representing a ring buffer of arbitrary objects.
/// The ring buffer can hold at most `capacity` elements.
/// When the size of the ring buffer exceeds `capacity` elements,
/// adding elements will cause the oldest element in the buffer
/// to be dropped.
pub struct RingArray<T> {
    data: Vec<Option<T>>,
    head: usize,
    capacity: usize,
    count: usize,
}

impl<T: Clone> RingArray<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![None; capacity],
            head: 0,
            capacity,
            count: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.data[self.head] = Some(item);
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start_index = if self.count < self.capacity {
            0
        } else {
            self.head
        };

        (0..self.count).map(move |i| {
            let idx = (start_index + i) % self.capacity;
            self.data[idx].as_ref().unwrap()
        })
    }

    pub fn len(&self) -> usize {
        self.count
    }
}