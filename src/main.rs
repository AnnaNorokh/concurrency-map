#![allow(unused_imports)]
#![allow(dead_code)]
//#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(incomplete_features)]
#![allow(unused_assignments)]
#![feature(linked_list_remove)]
#![feature(generic_const_exprs)]


use std::collections::LinkedList;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::option::Option;
use rand::Rng;

pub use crate::dataset_gen::dataset;
pub use crate::xxhasher::hasher;
pub use crate::newhasher::*;
mod dataset_gen; 
mod xxhasher; 
mod newhasher;

const DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
struct GuardCell<K, V> {
    guard: AtomicBool,
    list: LinkedList<(K, V)>,
}

struct LocklessMap<K, V> {
    map: Vec<GuardCell<K, V>>,
    capacity: usize,
}


impl<K, V> GuardCell<K, V> {

    pub fn new() -> Self {
        Self {
            guard: AtomicBool::new(false),
            list: LinkedList::new(),
        }
    }

    fn lock(&self) {
        loop {
            if self.guard.compare_exchange_weak(false, true, SeqCst, SeqCst).is_ok() {
                return;
            }
        }
    }

    fn unlock(&self) {
        self.guard.store(false, SeqCst);
    }

}

impl<K, V> LocklessMap< K, V>
where
    K: std::cmp::PartialEq + Sized + Debug + std::fmt::Display, 
    [u64 ; std::mem::size_of::<K>() / std::mem::size_of::<u64>()]: Sized,
    V: std::cmp::PartialEq + Debug
{
    pub fn new() -> Self {
        let mut vec = Vec::with_capacity(DEFAULT_CAPACITY);

        for _ in 0..DEFAULT_CAPACITY {
            vec.push(GuardCell::new());
        }
        Self { map: vec, capacity: DEFAULT_CAPACITY}
    }

    pub fn with_capacity(capacity: usize ) -> Self {
        let mut vec = Vec::with_capacity(capacity);
       
        for _ in 0..capacity {
            vec.push(GuardCell::new());
        }
        Self { map: vec, capacity: capacity}
    }

    fn size(&self) -> usize {
        let mut list_size = 0;

        for node in self.map.iter() {
            list_size += node.list.len(); 
        }
 
        list_size
    }

    fn contains_key(&self, key: &K) -> bool {  
        let hash = newhasher::hash_adapter(&key);
        let index = (hash as usize) % self.capacity;
        self.map[index].lock();
        
        for x in self.map[index].list.iter() {
            if &x.0 == key {
                self.map[index].unlock();
               return true;
           } 
        }

        self.map[index].unlock();
        false
    }

    fn remove(&self, key: &K) -> Option<V>{ 
        let hash = newhasher::hash_adapter(&key);
        let index = (hash as usize) % self.capacity;
        self.map[index].lock();
        let mut_self = unsafe { &mut *(self as *const LocklessMap< K, V> as *mut LocklessMap< K, V>)};
        let mut i = 0;

        for x in self.map[index].list.iter() {
            if x.0 == *key {
                let return_val = mut_self.map[index].list.remove(i);
                self.map[index].unlock();
                return Some(return_val.1);
            } 
            i+=1;
        }

        self.map[index].unlock();
         None
    }

    fn put_data_into_map (&self, dataset: Vec<(K, V)>){
        for x in dataset {
            Self::insert(&self, x.0, x.1);
        }
    }

    fn print (&self) {
        for node in self.map.iter() {
            let list = &node.list;
            for i in list.iter() {
                println!("{:?}", i);
            }
        }
    }

    fn get(&self, key: &K) -> Option<&V> {    
        let hash = newhasher::hash_adapter(&key);
        let index = (hash as usize) % self.capacity;
        self.map[index].lock();

        for x in self.map[index].list.iter() {
           if x.0 == *key {
                self.map[index].unlock();
                return Some(&x.1);
           } 
        }

        self.map[index].unlock();
        None
    }

    fn insert(&self, key: K, value: V) -> Option<&V> {
        let hash = newhasher::hash_adapter(&key);
        let index = (hash as usize) % self.capacity;
        self.map[index].lock();

        let mut_self = unsafe { &mut *(self as *const LocklessMap< K, V> as *mut LocklessMap< K, V>)};
        let set = (key, value);
        let mut i = 0;

        self.map[index].unlock();
        if Self::contains_key(&self, &set.0) {
            self.map[index].lock();
            for x in self.map[index].list.iter() {                                      // iter in node list 
                if x.0 == set.0 {                                                                //if keys are equal  
                    mut_self.map[index].list.remove(i);
                    mut_self.map[index].list.push_front(set);
                    self.map[index].unlock();
                    return Some(&x.1);
                }
                i+=1;
            }
        }

        mut_self.map[index].list.push_front(set);

        self.map[index].unlock();
        None
    }
}

fn main() {
    
}


#[cfg(test)]
mod tests {

    use std::sync::atomic::*;
    use super::*;
    use rand::*;
    use rand::prelude::*;
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Barrier};

    const THREADS: usize = 10;
    const ITERATIONS: usize = 1000;
    const CAPACITY: usize = 100;
    const CAPACITY1: usize = 10000;
    const CAPACITY2: usize = 1000;

    #[test]
    fn test_basic_methods () {
        const CAPACITY: usize = 66;
        let map = LocklessMap::<u32, u32>::with_capacity(CAPACITY);

        map.insert(6, 6666);
        map.insert(6, 6666);
        assert_eq!(1, map.size());
        assert_eq!(*map.get(&6).unwrap(), 6666);
        assert_eq!(map.contains_key(&6), true);
        map.remove(&6);
        assert_eq!(map.contains_key(&6), false);
        
        map.insert(10,1000);
        map.insert(11,1001);
        map.insert(12,1002);
        map.insert(13,1002);
        map.insert(14,1002);
        assert_eq!(5, map.size());
  
    }

    #[test]
    fn test_multithread_insert () {
        let barrier = Arc::new(Barrier::new(THREADS));
        let mut threads = Vec::with_capacity(THREADS);
        let mut map = Arc::new(LocklessMap::<u32, String>::with_capacity(CAPACITY2));           //1000
        let dataset = Arc::new(dataset_gen::dataset::create_ordered_dataset(CAPACITY1));          //10000
                       
        for thread in 0..THREADS {
            let shared_map = Arc::clone(&mut map);
            let shared_dataset = dataset.clone();
            let loc_barrier = barrier.clone();
            threads.push(thread::spawn( move || {
                loc_barrier.wait();
                let x = thread * 100;
                let y: usize = x + 100;

                for x in x..y {
                    let node = shared_dataset.get(x).unwrap();
                    shared_map.insert(node.0 as u32, node.1.clone());
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

    }

    #[test]
    fn test_read_write () {
        let barrier = Arc::new(Barrier::new(THREADS));
        let mut threads = Vec::with_capacity(THREADS);
        let mut map = Arc::new(LocklessMap::<u32, String>::with_capacity(CAPACITY2));               //1000
        let dataset = Arc::new(dataset_gen::dataset::create_ordered_dataset(CAPACITY1));              //10000

        for thread in 0..THREADS {
            let shared_map = Arc::clone(&mut map);
            let shared_dataset = dataset.clone();
            let loc_barrier = barrier.clone();

            threads.push(thread::spawn( move || {
                loc_barrier.wait();

                if thread % 2 == 0 {
                    let node = shared_dataset.get(thread).unwrap();
                    shared_map.insert(node.0 as u32, node.1.clone());
                } else {
                    let mut rng = thread_rng();
                    let index: u32 = rng.gen_range(0..10_000);
                    match shared_map.get(&index) {
                        Some(_) => println!("{:?}", shared_map.get(&index).unwrap()),
                        None => println!("no such value"),
                        //Ok() => ,
                        //Err() =>  {
                    }
                }

            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_insert () {
        let map: LocklessMap<u32,u32> = LocklessMap::new();

        map.insert(10,1000);
        map.insert(11,1001);
        map.insert(12,1002);
        assert_eq!(Some(&1000u32), map.get(&10));
        
        map.insert(10,1001);
        assert_eq!(Some(&1001u32), map.get(&10));

        map.insert(10,1001);
        assert_eq!(Some(&1001u32), map.get(&10));
            
        assert_eq!(3, map.size());
        assert_eq!(false, map.contains_key(&1));
        assert_eq!(true, map.contains_key(&10));
        
    }

    #[test]
    fn test_hasher () {
        #[derive(Debug)]
        //#[repr(packed)]
        struct Key {
            value: u64,
            value2: u64,
        }
    
        let key: Key = Key {value: 155, value2: 155};
        let array: [u64;2] = [15555555555555,12345634567];

        let key_hash = xxhasher::hasher::generic_xxhash::<Key>(&key);
        let array_hash = xxhasher::hasher::generic_xxhash::<[u64;2]>(&array);
        let hashu8 = xxhasher::hasher::generic_xxhash::<u8>(&10u8);
        let hashu16 = xxhasher::hasher::generic_xxhash::<u16>(&10u16);
        let hashu32 = xxhasher::hasher::generic_xxhash::<u32>(&10u32);
        let hashu64 = xxhasher::hasher::generic_xxhash::<u64>(&10u64);

        assert_eq!(hashu8, xxhasher::hasher::generic_xxhash::<u8>(&10u8));
        assert_eq!(hashu16, xxhasher::hasher::generic_xxhash::<u16>(&10u16));
        assert_eq!(hashu32, xxhasher::hasher::generic_xxhash::<u32>(&10u32));
        assert_eq!(hashu64, xxhasher::hasher::generic_xxhash::<u64>(&10u64));
        assert_eq!(array_hash, xxhasher::hasher::generic_xxhash::<[u64;2]>(&array));
        assert_eq!(key_hash, xxhasher::hasher::generic_xxhash::<Key>(&key));
    }

}

