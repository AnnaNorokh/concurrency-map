
//#[derive(Debug)]
pub mod dataset {

    use std::fmt::Debug;

    use rand::*;
    use rand::distributions::Alphanumeric;

    pub fn create_random_dataset (size: usize) -> Vec<(u32, String)> {
        let mut dataset = Vec::with_capacity(size.try_into().unwrap());
        
        for _ in 0..size {
    
            let key = rand::thread_rng().gen_range(0..10000);
    
            let value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
            
            let block = (key, value);
    
            dataset.push(block);
        }
    
        dataset
    }

    pub fn create_ordered_dataset (size: usize) -> Vec<(usize, String)> {
        let mut dataset = Vec::with_capacity(size.try_into().unwrap());
        
        for i in 0..size {
    
            let key = i;
    
            let value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
            
            let block = (key, value);
    
            dataset.push(block);
        }
    
        dataset
    }

    pub fn print_dataset<T: Debug, S: Debug> (dataset: &Vec<(T, S)>) {
    
        for i in dataset.iter() {
            println!(" {:?}", i);
        }
    }

}
