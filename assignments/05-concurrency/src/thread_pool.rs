use crate::error::ThreadPoolError;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
pub struct ThreadPool<T: Send + 'static, R: Send + Sized + 'static> {
    workers: Vec<Worker<T, R>>,
    sender: Sender<Job<T, R>>,
    resultreciever: Receiver<R>,
}
type Job<T, R> = Box<dyn FnOnce(T) -> R + Send + 'static>;
impl<T: Send + Clone + 'static, R: Send + Clone + 'static> ThreadPool<T, R> {
    /// Create a new LocalThreadPool with num_threads threads.
    ///
    /// Errors:
    /// - If num_threads is 0, return an error
    pub fn new(num_threads: usize, input_type: T, return_type: R) -> Result<Self, ThreadPoolError> {
        if num_threads == 0 {
            Err(ThreadPoolError::ZeroThreads)
        } else {
            let mut workers: Vec<Worker<T, R>> = Vec::new();
            let (tx, rx): (Sender<Job<T, R>>, Receiver<Job<T, R>>) = mpsc::channel();
            let (result_sender, result_reciever) = mpsc::channel();
            let rx = Arc::new(Mutex::new(rx));
            for id in 0..num_threads {
                //We could fail to create a new thread if we are out of system resources
                workers.push(Worker::new(
                    id,
                    Arc::clone(&rx),
                    input_type.clone(),
                    return_type.clone(),
                    result_sender.clone(),
                )?);
            }
            Ok(ThreadPool {
                workers: workers,
                sender: tx,
                resultreciever: result_reciever,
            })
        }
    }

    /// Execute the provided function on the thread pool
    ///
    /// Errors:
    /// - If we fail to send a message, report an error
    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce(T) -> R + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).map_err(|_| ThreadPoolError::Send)?;
        Ok(())
    }
    /// Retrieve any results from the thread pool that have been computed
    pub fn get_results(&self) -> Vec<R> {
        let mut output = Vec::new();
        for _ in 0..self.workers.len() {
            if let Ok(result) = self.resultreciever.recv() {
                output.push(result);
            } else {
                break; // Exit if there are no more messages or if an error occurs
            }
        }
        output
    }
}
#[allow(dead_code)]
struct Worker<T, R> {
    id: usize,
    thread: JoinHandle<()>,
    arg: T,
    return_type: R,
}
impl<T: Send + Clone + 'static, R: Send + 'static> Worker<T, R> {
    fn new(
        id: usize,
        rx: Arc<Mutex<Receiver<Job<T, R>>>>,
        arg: T,
        return_type: R,
        result_sender: Sender<R>,
    ) -> Result<Self, ThreadPoolError> {
        let thread_arg = arg.clone();
        let output = Worker {
            id: id,
            thread: thread::Builder::new()
                .spawn(move || loop {
                    {
                        let job = rx.lock().unwrap().recv().unwrap();
                        //preform function and get result
                        let result = job(thread_arg.clone());
                        let _ = result_sender.send(result);
                    }
                })
                .map_err(|_| ThreadPoolError::ThreadOverload)?,
            arg: arg.clone(),
            return_type,
        };
        Ok(output)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_thread_pool_fail() {
        //test if zero threads are made we get an error
        assert!(
            ThreadPool::new(0, 0, 0).is_err(),
            "Expected the Threadpool to return an Erorr if constructed with zero threads"
        )
    }
    #[test]
    fn create_thread_pool_success() {
        let pool = ThreadPool::<i32, i32>::new(4, 0, 0);
        assert!(
            pool.is_ok(),
            "Expected the thread pool to be created successfully."
        );
    }

    #[test]
    fn execute_multiple_jobs_and_get_results() {
        let pool = ThreadPool::<i32, i32>::new(4, 0, 0).expect("Failed to create thread pool");

        // Schedule multiple jobs
        for i in 0..4 {
            pool.execute(move |x| x + i).expect("Failed to execute job");
        }

        // Collect results and verify all expected results are present
        let results = pool.get_results();
        assert_eq!(results.len(), 4, "Expected four results.");
        assert!(results.contains(&0));
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
    }
}
