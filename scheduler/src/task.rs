use std::cmp::Ordering;

#[derive(Eq, PartialEq, Clone)]
pub struct Task {
    pub id: u16,
    pub total_operations: u64,
    pub operations_remaining: u64,
    pub result: u64
}

impl Task {
    pub fn new(id: u16, operations: u64) -> Self {
        Task {
            id,
            total_operations: operations,
            operations_remaining: operations,
            result: 0,
        }
    }

    pub fn step(&mut self) {
        self.result += self.operations_remaining;
        self.operations_remaining -= 1;
    }

    pub fn is_completed(&self) -> bool {
        self.operations_remaining == 0
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.operations_remaining.partial_cmp(&other.operations_remaining)
            .map(|v| v.reverse())
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.operations_remaining.cmp(&other.operations_remaining).reverse()
    }
}