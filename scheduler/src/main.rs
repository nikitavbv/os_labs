use crate::task::Task;
use rand::Rng;
use crate::scheduler::Scheduler;
use std::collections::HashMap;

mod task;
mod scheduler;

fn main() {
    env_logger::init();

    println!("collecting wait time stats...");

    let stats = run_scheduler_demo(10, 20, 1, 200);
    let wait_time_by_ops_stats = stats.wait_time_by_ops;
    let mut keys: Vec<&u64> = wait_time_by_ops_stats.keys().collect();
    keys.sort();
    println!("wait_time_by_ops_stats");
    for key in keys {
        println!(
            "{}\t{}",
            key,
            *wait_time_by_ops_stats.get(&key).expect("value should be set for this key") as f64 /
                *stats.total_tasks_by_ops.get(&key).expect("value should be set for this key") as f64
        );
    }

    let mut stats_wait_time = Vec::new();
    let mut stats_idle_time = Vec::new();

    for n in 0..50 {
        let min_time_till_next_schedule = n;
        let max_time_till_next_schedule = min_time_till_next_schedule + 10;
        let avg_time_till_next_schedule = (min_time_till_next_schedule + max_time_till_next_schedule) / 2;

        let min_ops = 5;
        let max_ops = 20;

        let stats = run_scheduler_demo(min_time_till_next_schedule, max_time_till_next_schedule, min_ops, max_ops);
        stats_wait_time.push((avg_time_till_next_schedule, stats.average_wait_time));
        stats_idle_time.push((avg_time_till_next_schedule, stats.idle_time_percentage));
    }

    println!("stats_wait_time:");
    for record in stats_wait_time {
        println!("{}\t{}", record.0, record.1);
    }

    println!("\nstats_idle_time:");
    for record in stats_idle_time {
        println!("{}\t{}", record.0, record.1);
    }
}

fn run_scheduler_demo(min_time_till_next_schedule: u64, max_time_till_next_schedule: u64, min_ops: u64, max_ops: u64) -> SchedulerStats {
    let total_tasks = 20000;

    println!("Scheduler demo");

    let mut scheduler = Scheduler::new();
    let mut remaining_tasks = total_tasks;
    let mut time_till_next_schedule = 0;

    while remaining_tasks > 0 || scheduler.has_tasks() {
        if remaining_tasks > 0 {
            if time_till_next_schedule == 0 {
                scheduler.schedule(generate_task(min_ops, max_ops));
                time_till_next_schedule = time_for_next_schedule(
                    min_time_till_next_schedule,
                    max_time_till_next_schedule
                );
                remaining_tasks -= 1;
            } else {
                time_till_next_schedule -= 1;
            }
        }

        scheduler.tick();
    }

    println!("done.");
    println!("average wait time: {}", scheduler.average_wait_time());
    println!("idle time: {}%", scheduler.idle_time_percentage() * 100.0);

    SchedulerStats {
        average_wait_time: scheduler.average_wait_time(),
        idle_time_percentage: scheduler.idle_time_percentage(),
        wait_time_by_ops: scheduler.wait_time_by_ops,
        total_tasks_by_ops: scheduler.total_tasks_by_ops
    }
}

fn generate_task(min_ops: u64, max_ops: u64) -> Task {
    Task::new(
        rand::random(),
        rand::thread_rng().gen_range(min_ops, max_ops)
    )
}

fn time_for_next_schedule(min_time_till_next_schedule: u64, max_time_till_next_schedule: u64) -> u64 {
    rand::thread_rng().gen_range(min_time_till_next_schedule, max_time_till_next_schedule)
}

struct SchedulerStats {
    average_wait_time: f64,
    idle_time_percentage: f64,
    wait_time_by_ops: HashMap<u64, u64>,
    total_tasks_by_ops: HashMap<u64, u64>
}