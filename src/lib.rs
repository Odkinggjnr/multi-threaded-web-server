use std::sync::mpsc;   // channel for sending jobs between threads
use std::sync::Arc;    // lets multiple threads share one thing
use std::sync::Mutex;  // a lock so only one thread touches the data at a time
use std::thread;

// a Job is a function we want a worker to run
type Job = Box<dyn FnOnce() + Send + 'static>;

// what we send through the channel
enum Message {
    NewJob(Job),
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // create a channel — one end sends jobs, the other receives them
        let (sender, receiver) = mpsc::channel();

        // wrap receiver so all workers can share it safely
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // sends a job to the pool for a free worker to pick up
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(Message::NewJob(job)).unwrap();
    }
}

// runs automatically when the ThreadPool is cleaned up — graceful shutdown
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // close the channel so workers know no more jobs are coming
        drop(self.sender.take());

        // wait for each worker to finish
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // wait for a job from the channel
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(Message::NewJob(job)) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    // channel closed, time to stop
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}
