use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool<T> where T: FnOnce() {
    workers: Vec<Worker<T>>,
    sender: mpsc::Sender<Job<T>>,
}

impl<T> ThreadPool<T> where T: FnOnce() {
    fn new(nworkers: usize) -> ThreadPool<T> {
        let mut workers = Vec::new();
        let (send, recv) = mpsc::channel();
        let recv = Arc::new(Mutex::new(recv));

        for i in 0..nworkers {
            workers.push(Worker::new(i, Arc::clone(&recv)));
        }

        ThreadPool {
            workers: workers,
            sender: send
        }
    }
}

struct Worker<T> {
    id: usize,
    handle: thread::JoinHandle<()>,
}

impl<T> Worker<T> where T: FnOnce() {
    fn new(id: usize, recv: Arc<Mutex<mpsc::Receiver<Job<T>>>>) -> Worker<T> {
        Worker {
            id,
            handle: std::thread::spawn(move || {
                loop {
                    let job = recv.lock().unwrap().recv().unwrap();
                    job.func();
                }
            }),
        }
    }
}

struct Job<T> where T: FnOnce() {
    func: T
}
