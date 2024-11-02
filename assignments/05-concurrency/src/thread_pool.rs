use std::thread::{self, Thread,Builder};
use crate::error::ThreadPoolError;



pub struct ThreadPool {
    builders: Vec<Builder>
}

impl ThreadPool {
    /// Create a new LocalThreadPool with num_threads threads.
    ///
    /// Errors:
    /// - If num_threads is 0, return an error
    /// Build the thread by using Thread Build
    pub fn new(num_threads: usize) -> Result<Self,ThreadPoolError>{
       // check if the amount of desired threads is 0
        let mut output = ThreadPool{
            builders:vec![]
        };
        if num_threads == 0{
            Err(ThreadPoolError::ZeroThreads)
        }else{
            let mut i = 0;
            let mut builders:Vec<Builder> = vec![];
            while i < num_threads{
                //create a new thread builder
                builders.push(thread::Builder::new());
                i +=1;
            }
            output.builders = builders;
            Ok(output)
        }
    }

    /// Execute the provided function on the thread pool
    ///
    /// Errors:
    /// - If we fail to send a message, report an error
    pub fn execute<F:Fn>(&self, f: F) {
        //first we are going to define a channel for our threads to report their results
        
        //First we are going to spawn our built threads
        for builder in &self.builders{
            builder.spawn(f)
        }
        todo!()
    }
    /// Retrieve any results from the thread pool that have been computed
    pub fn get_results(&self) {
        todo!()
    }
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_new_threads(){
        //check error case
        assert_eq!(ThreadPool::new(0).is_err_and(|x| x==ThreadPoolError::ZeroThreads), true);
        //check to see if we created the right number of thread builders
        assert_eq!(ThreadPool::new(5).unwrap().builders.len(),5);
    }
}