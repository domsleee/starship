use std::{collections::HashMap, time::Instant};

use systemstat::Duration;

#[derive(Default)]
pub struct Timings {
    started: HashMap<usize, Instant>,
    total: HashMap<usize, Duration>,
    count: HashMap<usize, usize>,
}

impl Timings {
    pub fn start(&mut self, id: usize) {
        self.started.insert(id, std::time::Instant::now());
    }

    pub fn stop(&mut self, id: usize) {
        match self.started.get(&id) {
            Some(start) => {
                let existing_total = self.total.get(&id).unwrap_or(&Duration::default()).clone();
                let existing_count = self.count.get(&id).unwrap_or(&0);
                self.total.insert(
                    id,
                    existing_total + std::time::Instant::now().duration_since(*start),
                );
                self.count.insert(id, existing_count + 1);
            }
            None => {
                eprintln!("what?");
            }
        }
    }

    pub fn print(&self) {
        println!("ID  COUNT   AVERAGE   TOTAL");
        for (id, total) in &self.total {
            let count = *self.count.get(id).unwrap_or(&0);
            let average_duration = total.checked_div(count as u32).unwrap_or_default();
            let total_duration = format!("{:?}", total);
            println!(
                "<{:3}:>  <{:3}:>  <{:8?}>  <{:8}>",
                id, count, average_duration, total_duration
            );
        }
    }
}
