use std::ops::Drop;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job");
                    job();
                }

                Err(_) => {
                    println!("Worker {id} shutting down");
                    break;
                }
                
            }
        });
        Worker {
            id,
            handle: Some(thread),
        }
    }
}

// struct Job{

// }

pub struct ThreadPool {
    number_of_threads: usize,
    threads: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(number_of_threads: usize) -> ThreadPool {
        assert!(number_of_threads > 0);
        let mut threads = Vec::with_capacity(number_of_threads);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..number_of_threads {
            threads.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            number_of_threads,
            threads,
            sender: Some(sender),
        }
    }

    pub fn number_of_threads(&self) -> usize {
        self.number_of_threads
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.threads {
            drop(self.sender.take());
            println!("Shutting Down worker {}", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
