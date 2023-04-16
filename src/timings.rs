use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
    time::Instant,
};
use systemstat::Duration;

const IS_ENABLED: bool = true;

#[derive(Default)]
pub struct Timings {
    started: Arc<Mutex<HashMap<ThreadId, HashMap<usize, Instant>>>>,
    total: Arc<Mutex<HashMap<ThreadId, HashMap<usize, Duration>>>>,
    count: Arc<Mutex<HashMap<ThreadId, HashMap<usize, usize>>>>,
}

impl Timings {
    pub fn start(&self, id: usize) {
        if !self.is_enabled(id) {
            return;
        }
        let mut started_lock = self.started.lock().unwrap();
        let started = started_lock
            .entry(thread::current().id())
            .or_insert_with(HashMap::new);
        started.insert(id, std::time::Instant::now());
    }

    fn is_enabled(&self, id: usize) -> bool {
        return IS_ENABLED && id >= 15;
    }

    pub fn stop(&self, id: usize) {
        if !self.is_enabled(id) {
            return;
        }

        let stopped_instant = std::time::Instant::now();
        let mut started_lock = self.started.lock().unwrap();
        let started = started_lock
            .entry(thread::current().id())
            .or_insert_with(HashMap::new);
        let mut total_lock = self.total.lock().unwrap();
        let total = total_lock
            .entry(thread::current().id())
            .or_insert_with(HashMap::new);
        let mut count_lock = self.count.lock().unwrap();
        let count = count_lock
            .entry(thread::current().id())
            .or_insert_with(HashMap::new);
        match started.get(&id) {
            Some(start) => {
                let existing_total = total.get(&id).unwrap_or(&Duration::default()).clone();
                let existing_count = count.get(&id).unwrap_or(&0).clone();
                total.insert(id, existing_total + stopped_instant.duration_since(*start));
                count.insert(id, existing_count + 1);
            }
            None => {
                eprintln!("what?");
            }
        }
    }

    pub fn print(&self) {
        // println!("AVG FROM {} threads", fla.len());
        println!("{:3} {:5} {:8} {:8}", "id", "count", "average", "total");
        let count = self.sum_all_threads(&self.count);
        let total = self.sum_all_threads(&self.total);
        for (id, id_count) in total.iter() {
            let count_duration = *count.get(id).unwrap_or(&0);
            let average_duration = id_count
                .checked_div(count_duration as u32)
                .unwrap_or_default();
            let total_duration = format!("{:?}", id_count);
            println!(
                "{:3} {:5} {:8?} {:8}",
                id, count_duration, average_duration, total_duration
            );
        }
    }

    fn sum_all_threads<
        T: PartialEq + Eq + std::hash::Hash + Copy + Default,
        U: std::ops::AddAssign + Copy + Default,
    >(
        &self,
        hashmap: &Arc<Mutex<HashMap<ThreadId, HashMap<T, U>>>>,
    ) -> HashMap<T, U> {
        let mut res = HashMap::new();
        let map = hashmap.lock().unwrap();
        for (_, inner_map) in map.iter() {
            for (k, v) in inner_map.iter() {
                *res.entry(*k).or_insert_with(Default::default) += *v;
            }
        }
        res
    }
}
