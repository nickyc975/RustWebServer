use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait Executable {
    fn execute(&mut self);
}

pub struct ThreadPool {
    #[used]
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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

    pub fn execute(&self, job: Job)
    {
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
    fn new(id: usize, recv: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            handle: std::thread::spawn(move || loop {
                let mut job = recv.lock().unwrap().recv().unwrap();
                println!("Worker {} is working...", id);
                job.execute();
                println!("Worker {} finished work!", id);
            }),
        }
    }
}

type Job = Box<dyn Executable + Send>;
