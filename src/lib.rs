// use std::sync::mpsc::Receiver;
// use std::{sync::mpsc, sync::Mutex, sync::Arc};
// use std::thread::{self};
// pub struct ThreadPool {
//     workers: Vec<Worker>,
//     sender: Option<mpsc::Sender<Job>>,
// }

// type Job = Box<dyn FnOnce() + Send + 'static>;

// pub struct Worker {
//     id: usize,
//     thread: Option<thread::JoinHandle<()>>,
// }

// impl Worker {
//     pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
//         let thread = std::thread::spawn(move || {
//             loop {
//                 let message = receiver.lock().unwrap().recv();
//                 match message {
//                     Ok(job) => {
//                         println!("Worker {id} got a job; executing.");
//                         job();
//                     }
//                     Err(_) => {
//                         println!("Worker {id} disconnected; shutting down.");
//                         break;
//                     }
//                 }
//             }
//         });
//         Worker { id, thread:Some(thread) }
//     }
// }

// impl ThreadPool {
//     pub fn new(size: usize) -> ThreadPool {
//         assert!(size > 0);


//         let (sender, receiver) = mpsc::channel();
//         let receiver = Arc::new(Mutex::new(receiver));
//         let mut workers = Vec::with_capacity(size);
//         for i in 0..size {
//             // threads.push(thread::JoinHandle<>);
//             workers.push(Worker::new(i, Arc::clone(&receiver)));
//         }
//         ThreadPool { workers: workers, sender: Some(sender) }
//     }

//     pub fn execute<F>(&self, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         let job = Box::new(f);
//         self.sender.as_ref().unwrap().send(job).unwrap();
//     }
// }

// impl Drop for ThreadPool {
//     fn drop(&mut self) {
//         drop(self.sender.take());

//         for worker in &mut self.workers {
//             println!("Shutting down worker {}", worker.id);
//             if let Some(thread) = worker.thread.take() {
//                 thread.join().unwrap();
//             }
//         }
//     }
// }
