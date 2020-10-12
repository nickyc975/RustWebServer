use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait Job: Send {
    fn execute(&mut self);
}

pub struct ThreadPool {
    #[used]
    workers: Vec<Worker>,
    sender: mpsc::Sender<Box<dyn Job>>,
}

impl ThreadPool {
    pub fn new(nworkers: usize) -> ThreadPool {
        let mut workers = Vec::new();
        let (send, recv) = mpsc::channel();
        let recv = Arc::new(Mutex::new(recv));

        for i in 0..nworkers {
            workers.push(Worker::new(i, Arc::clone(&recv)));
        }

        ThreadPool {
            workers: workers,
            sender: send,
        }
    }

    pub fn add_job(&self, job: Box<dyn Job>) {
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    #[used]
    id: usize,
    #[used]
    handle: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, recv: Arc<Mutex<mpsc::Receiver<Box<dyn Job>>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            let mut job = recv.lock().unwrap().recv().unwrap();
            println!("Worker {} is working...", id);
            // thread::sleep(std::time::Duration::from_millis(5000));
            job.execute();
            println!("Worker {} finished work!", id);
        });

        Worker { id, handle }
    }
}
