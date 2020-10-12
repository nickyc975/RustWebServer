use std::sync::{mpsc, Arc, Mutex};
use std::thread;

enum Message {
    Job(Box<dyn Job>),
    Stop,
}

enum State {
    Running,
    Stopped,
}

pub trait Job: Send {
    fn execute(&mut self);
}

pub struct ThreadPool {
    state: State,
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(nworkers: usize) -> ThreadPool {
        let mut workers = Vec::new();
        let (send, recv) = mpsc::channel();

        // Wrap up recv so that it can be assigned to multi workers.
        let recv = Arc::new(Mutex::new(recv));

        for i in 0..nworkers {
            workers.push(Worker::new(i, Arc::clone(&recv)));
        }

        ThreadPool {
            state: State::Running,
            workers: workers,
            sender: send,
        }
    }

    pub fn add_job(&self, job: Box<dyn Job>) {
        self.sender.send(Message::Job(job)).unwrap();
    }

    pub fn close(&mut self) {
        match self.state {
            State::Running => {
                self.state = State::Stopped;

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
            _ => (),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.close();
    }
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
                Message::Job(mut job) => {
                    println!("Worker {} is working...", id);

                    // thread::sleep(std::time::Duration::from_millis(5000));
                    job.execute();

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
