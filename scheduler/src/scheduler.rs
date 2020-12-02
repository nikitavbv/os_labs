use std::collections::{BinaryHeap, HashMap};

use log::info;

use crate::task::Task;

pub struct Scheduler {
    pub current_task: Option<Task>,
    pub queue: BinaryHeap<Task>,

    pub tick: u64,
    pub schedule_time: HashMap<u16, u64>,

    pub total_wait_time: u64,
    pub total_tasks_completed: u64,
    pub wait_time_by_ops: HashMap<u64, u64>,
    pub total_tasks_by_ops: HashMap<u64, u64>,

    pub total_idle_time: u64,
}

impl Scheduler {

    pub fn new() -> Self {
        Scheduler {
            current_task: None,
            queue: BinaryHeap::new(),
            tick: 0,
            schedule_time: HashMap::new(),
            total_wait_time: 0,
            total_tasks_completed: 0,
            wait_time_by_ops: HashMap::new(),
            total_tasks_by_ops: HashMap::new(),
            total_idle_time: 0,
        }
    }

    pub fn schedule(&mut self, task: Task) {
        info!("[schedule]: scheduled task {} with {} ops", task.id, task.operations_remaining);
        self.schedule_time.insert(task.id, self.tick);
        self.total_tasks_by_ops.insert(
            task.total_operations,
            self.total_tasks_by_ops.get(&task.total_operations).unwrap_or(&0) + 1
        );
        self.queue.push(task);
    }

    pub fn tick(&mut self) {
        self.tick += 1;

        if let Some(task) = self.current_task.as_mut() {
            task.step();
            if task.is_completed() {
                info!("[tick]: finished running task {}", task.id);
                self.current_task = None;
                self.total_tasks_completed += 1;
            } else {
                info!("[tick]: continuing to run task {}, ops remaining: {}", task.id, task.operations_remaining);
            }
            return;
        }

        if let Some(mut task) = self.queue.pop() {
            let wait_time = self.tick - self.schedule_time.get(&task.id)
                .expect("schedule time should always be set when scheduling task");
            self.total_wait_time += wait_time;
            self.wait_time_by_ops.insert(
                task.total_operations,
                self.wait_time_by_ops.get(&task.total_operations).unwrap_or(&0) + wait_time
            );
            task.step();

            if !task.is_completed() {
                info!("[tick]: started task {}, ops remaining: {}", task.id, task.operations_remaining);
                self.current_task = Some(task);
            } else {
                info!("[tick]: started and completed task {} (1 ops)", task.id);
            }
        } else {
            info!("[tick]: no tasks to run");
            self.total_idle_time += 1;
        }
    }

    pub fn has_tasks(&self) -> bool {
        !self.queue.is_empty() || self.current_task.is_some()
    }

    pub fn average_wait_time(&self) -> f64 {
        self.total_wait_time as f64 / self.total_tasks_completed as f64
    }

    pub fn idle_time_percentage(&self) -> f64 {
        self.total_idle_time as f64 / self.tick as f64
    }
}