use crossbeam_channel::{Sender, bounded};
use std::thread;

struct WorkPool<T: Send + 'static> {
    tx: Sender<T>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl<T: Send + 'static> WorkPool<T> {
    pub fn new<F>(thread_count: usize, chan_size: usize, work_fn: F) -> Self
    where
        F: Fn(T) + Send + Clone + 'static,
    {
        let (tx, rx) = bounded(chan_size);

        let mut threads = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let thread_rx = rx.clone();
            let thread_work = work_fn.clone();
            threads.push(thread::spawn(move || {
                while let Ok(val) = thread_rx.recv() {
                    thread_work(val);
                }
            }));
        }

        Self { tx, threads }
    }

    pub fn send(&self, value: T) {
        self.tx.send(value).unwrap();
    }

    pub fn join(self) {
        drop(self.tx);
        for t in self.threads {
            t.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn is_prime(i: u64) -> bool {
        for j in 2..i {
            if i % j == 0 {
                return false;
            }
        }
        true
    }

    #[test]
    fn simple() {
        let results = HashMap::<u64, bool>::new();
        let res_lock = Arc::new(Mutex::new(results));
        let res_lock2 = res_lock.clone();

        let pool = WorkPool::new(8, 32, move |i| {
            let p = is_prime(i);
            let mut r = res_lock.lock().unwrap();
            r.insert(i, p);
        });

        pool.send(5);
        pool.send(10);
        pool.send(15);
        pool.send(19);
        pool.join();

        let results = res_lock2.lock().unwrap();
        let mut res_pairs: Vec<_> = results.iter().collect();
        res_pairs.sort();
        assert_eq!(
            res_pairs,
            &[(&5, &true), (&10, &false), (&15, &false), (&19, &true)]
        );
    }
}
