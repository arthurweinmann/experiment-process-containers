//! Work stealing thread pool
//!

use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use std::borrow::Borrow;
use std::iter;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

pub struct ThreadPool {
    ingester: Arc<Injector<Message>>,
    stealers_ref: Arc<Vec<Stealer<Message>>>,
    taskers: Vec<Tasker>,
}

impl ThreadPool {
    pub fn new(workers_count: usize) -> ThreadPool {
        assert!(workers_count > 0);

        let mut stealers = Vec::with_capacity(workers_count);
        let mut workers = Vec::with_capacity(workers_count);
        for _ in 0..workers_count {
            let w = Worker::new_fifo();
            stealers.push(w.stealer());
            workers.push(w);
        }

        let stealers_ref = Arc::new(stealers);
        let ingester = Arc::new(Injector::new());
        let mut taskers = Vec::with_capacity(workers_count);

        for id in 0..workers_count {
            taskers.push(Tasker::new(
                id,
                stealers_ref.clone(),
                workers.pop().unwrap(),
                ingester.clone(),
            ));
        }

        ThreadPool {
            ingester,
            stealers_ref,
            taskers,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.ingester.push(Message::NewJob(job));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Sending terminate to all workers
        for _ in &mut self.taskers {
            self.ingester.push(Message::Terminate);
        }

        // Shutting down all workers
        for ta in &mut self.taskers {
            if let Some(thread) = ta.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Tasker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Tasker {
    fn new(
        id: usize,
        stealers: Arc<Vec<Stealer<Message>>>,
        worker: Worker<Message>,
        ingester: Arc<Injector<Message>>,
    ) -> Tasker {
        let thread = thread::spawn(move || loop {
            if let Some(message) = Tasker::find_task(&worker, ingester.borrow(), stealers.clone()) {
                match message {
                    Message::NewJob(job) => {
                        job.call_box();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            }
        });

        Tasker {
            id,
            thread: Some(thread),
        }
    }

    fn find_task<T>(
        local: &Worker<T>,
        global: &Injector<T>,
        stealers: Arc<Vec<Stealer<T>>>,
    ) -> Option<T> {
        // Pop a task from the local queue, if not empty.
        local.pop().or_else(|| {
            // Otherwise, we need to look for a task elsewhere.
            iter::repeat_with(|| {
                // Try stealing a batch of tasks from the global queue.
                global
                    .steal_batch_and_pop(local)
                    // Or try stealing a task from one of the other threads.
                    .or_else(|| stealers.iter().map(|s| s.steal()).collect())
            })
            // Loop while no task was stolen and any steal operation needs to be retried.
            .find(|s| !s.is_retry())
            // Extract the stolen task, if there is one.
            .and_then(|s| s.success())
        })
    }
}
