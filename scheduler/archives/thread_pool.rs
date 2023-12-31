// use std::sync::mpsc;
// use std::sync::Arc;
// use std::sync::Mutex;
use std::thread;

use crossbeam_channel::{bounded, Receiver, Sender};

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
    workers: Vec<Worker>,
    // sender: mpsc::Sender<Message>,
    sender: Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // let (sender, receiver) = mpsc::channel();
        let (sender, receiver) = bounded(size + 1);

        // let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Sending terminate to all workers
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        // Shutting down all workers
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Receiver<Message>) -> Worker {
        // fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.recv().unwrap();
                // let message = { receiver.lock().unwrap().recv().unwrap() };

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

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
