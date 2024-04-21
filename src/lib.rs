use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{JoinHandle, spawn};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    handle: Option<JoinHandle<()>>,
    id: usize,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let handle = spawn(move || loop {
            let job = receiver
                .lock()
                .unwrap()
                .recv();

            match job {
                Ok(job) => {
                    println!("Worker {id} received a job!");
                    job();
                },
                Err(_) => {
                    println!("Worker {id} shutting down!");
                    break
                }
            }
        });

        let handle = Some(handle);
        Worker { handle, id }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let sender = Some(sender);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool{workers, sender}
    }

    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}