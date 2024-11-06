use anyhow::Result;
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use thread_pool::ThreadPool;

mod error;
mod thread_pool;

/// Generate a random vector of size capacity filled with random i64s
fn random_vec(capacity: usize) -> Vec<i64> {
    let mut vec = vec![0; capacity];
    rand::thread_rng().fill(&mut vec[..]);
    vec
}

fn main() -> Result<()> {
    let start = Instant::now();
    let threadcount = 10;
    let data = random_vec(10_000_000);
    let pool = ThreadPool::new(threadcount, vec![1], vec![1])?;
    //split our data so we can head it off to a thread
    let data_vec = Arc::new(Mutex::new(split_array(data.len() / threadcount, data)));
    //give a job to each one of the threads
    for i in 0..threadcount {
        let data_vec = Arc::clone(&data_vec); // Clone the Arc to pass to each thread
        pool.execute(move |_| {
            let data_vec = data_vec.lock().unwrap(); // Lock to access the data safely
            let chunk = &data_vec[i];
            sort(chunk.to_vec()) // Sort the chunk in place
        })?;
    }
    //get all of our sorted vectors back
    let sorted = pool.get_results();
    //merge all of our sorted arrays;
    let merged = merge_results(sorted);
    //now we are going to check wheter we did this correctly
    let duration = start.elapsed();
    println!("{:?}", duration);
    assert!(
        merged.windows(2).all(|w| w[0] <= w[1]),
        "Vector is not sorted"
    );
    Ok(())
}
//given the array to be sorted this function splits it into n arrays and passes back
//a vector of n unsorted arrays
fn split_array(n: usize, input: Vec<i64>) -> Vec<Vec<i64>> {
    //define the size of each vector
    let mut result = Vec::with_capacity(n);
    let chunk_size = input.len() / n;
    let remainder = input.len() % n;

    let mut start = 0;
    for i in 0..n {
        // Calculate the current chunk size, adding 1 if within the remainder count
        let end = start + chunk_size + if i < remainder { 1 } else { 0 };
        result.push(input[start..end].to_vec());
        start = end;
    }

    result
}
// given an unsorted array of some length, this function splits the array into pairs
// and then merges the ordered pairs until we have one array
fn sort(input: Vec<i64>) -> Vec<i64> {
    //start by splitting the input array into arrays of size two,
    let mut unsorted: Vec<Vec<i64>> = split_array(input.len(), input);
    //now we want to sort merge each pair of arrays until we have one array
    while unsorted.len() > 1 {
        let mut pos = 0;
        while pos < unsorted.len() - 1 {
            //because of a borrow checker , I have to split the vector to obtain two mutuable referances of non overlapping
            //slices
            //I could do this unsafely which would, in my opinion be a neater implementation
            let (one, two) = unsorted.split_at_mut(pos + 1);
            //replace now merged vector
            unsorted[pos] = merge(&mut one[one.len() - 1], &two[0]).to_vec();
            unsorted.remove(pos + 1);
            pos += 2;
        }
    }
    unsorted[0].to_vec()
}
fn merge_results(mut unsorted: Vec<Vec<i64>>) -> Vec<i64> {
    while unsorted.len() > 1 {
        let mut pos = 0;
        while pos < unsorted.len() - 1 {
            //because of a borrow checker , I have to split the vector to obtain two mutuable referances of non overlapping
            //slices
            //I could do this unsafely which would, in my opinion be a neater implementation
            let (one, two) = unsorted.split_at_mut(pos + 1);
            //replace now merged vector
            unsorted[pos] = merge(&mut one[one.len() - 1], &two[0]).to_vec();
            unsorted.remove(pos + 1);
            pos += 2;
        }
    }
    unsorted[0].to_vec()
}
//given sorted vectors this merges them into one result vector
fn merge<'a>(one: &'a mut Vec<i64>, two: &Vec<i64>) -> &'a Vec<i64> {
    //iterate through each value in the second array to find it's place in the first
    for second in two {
        let mut pos = 0;
        while pos < one.len() {
            //we want to keep moving through the first array until the element in the first array is
            //greater than the element in the second
            if second <= &one[pos] {
                //insert the element in the second array before this
                one.insert(pos, *second);
                break;
            }
            // if this is the final element of the first array add to the end
            else if pos == one.len() - 1 && second > &one[pos] {
                one.push(*second);
                break;
            } else {
                pos += 1
            }
        }
    }
    one
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_split_array() {
        let input = random_vec(10);
        assert_eq!(split_array(2, input).len(), 2);
    }
    #[test]
    fn test_merge_results() {
        assert_eq!(
            merge_results(vec![vec![5, 6], vec![3, 4], vec![1, 2]]),
            vec![1, 2, 3, 4, 5, 6]
        )
    }
    #[test]
    fn test_merge() {
        //test for a simple case of vec of len 2
        assert_eq!(*merge(&mut vec![1, 4], &vec![2, 3]), vec![1, 2, 3, 4]);
        //testing with 3 elements and having an element of the second array be the greatest
        assert_eq!(
            *merge(&mut vec![2, 6, 7], &vec![2, 3, 9]),
            vec![2, 2, 3, 6, 7, 9]
        );
        //test where the second element contains the lowest value
        assert_eq!(
            *merge(&mut vec![3, 6, 7], &vec![2, 3, 9]),
            vec![2, 3, 3, 6, 7, 9]
        );
    }
    #[test]
    fn test_sort() {
        assert_eq!(sort(vec![2, 1, 3]), vec![1, 2, 3]);
        assert_eq!(sort(vec![2, 1, 6, 3]), vec![1, 2, 3, 6]);
    }
}
