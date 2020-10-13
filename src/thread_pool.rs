//! An implementation of M:N job scheduling model.
//!
//! ### Examples
//! ```rust
//! // Define a struct to hold all the needed information for you job.
//! struct MyJob(i32);
//!
//! // Implement Job trait for the struct so that thread pool knowns how to run your job.
//! impl Job for MyJob {
//!     fn run(&self) {
//!         // Say hello.
//!         println!("Hello from job {}", self.0);
//!     }
//! }
//!
//! fn main() {
//!     // Create a thread pool and start you jobs.
//!     let pool = ThreadPool::new(4);
//!     for i in 0..8 {
//!         pool.enqueue(Box::new(MyJob(i)));
//!     }
//! }
//! ```

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait Job: Send {
    fn run(&self);
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(nworkers: usize) -> ThreadPool {
        assert!(nworkers > 0);

        let (send, recv) = mpsc::channel();
        let mut workers = Vec::with_capacity(nworkers);

        // Wrap up recv so that it can be assigned to multi workers.
        let recv = Arc::new(Mutex::new(recv));

        // Create workers.
        for i in 0..nworkers {
            workers.push(Worker::new(i, Arc::clone(&recv)));
        }

        ThreadPool {
            workers: workers,
            sender: send,
        }
    }

    pub fn enqueue(&self, job: Box<dyn Job>) -> Result<(), String> {
        match self.sender.send(Message::Run(job)) {
            Err(_) => Err(String::from("Error enqueuing job!")),
            Ok(_) => Ok(()),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send stop message to workers.
        for _ in self.workers.iter() {
            self.sender.send(Message::Stop).unwrap();
        }

        // Wait worker threads to stop.
        for worker in &mut self.workers {
            println!("Stopping worker {}...", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }

            println!("Stopped worker {}!", worker.id);
        }
    }
}

enum Message {
    Run(Box<dyn Job>),
    Stop,
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, recv: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            // Acquire the lock and take message, then drop the lock.
            let message = recv.lock().unwrap().recv().unwrap();

            match message {
                Message::Run(job) => {
                    println!("Worker {} is working...", id);

                    // thread::sleep(std::time::Duration::from_millis(5000));
                    job.run();

                    println!("Worker {} finished work!", id);
                }
                Message::Stop => {
                    println!("Worker {} received stop message!", id);
                    return;
                }
            }
        });

        Worker {
            id,
            handle: Some(handle),
        }
    }
}
