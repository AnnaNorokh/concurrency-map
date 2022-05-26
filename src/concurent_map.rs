#![allow(unused_parens)]
#![allow(incomplete_features)]
#![allow(unused_assignments)]


use std::collections::LinkedList;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::option::Option;

pub use crate::dataset_gen::dataset;
pub use crate::xxhasher::hasher;
pub use crate::newhasher::*;


const DEFAULT_CAPACITY: usize = 16;

pub struct GuardCell<K, V> {
    guard: AtomicBool,
    list: LinkedList<(K, V)>,
}

pub struct LocklessMap<K, V> {
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

    pub fn lock(&self) {
        loop {
            if self.guard.compare_exchange_weak(false, true, SeqCst, SeqCst).is_ok() {
                return;
            }
        }
    }

    pub fn unlock(&self) {
        self.guard.store(false, SeqCst);
    }

}

impl<K, V> LocklessMap<K, V>
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
/* 
    fn update_capacity(&self) {
        let new_capacity = self.capacity * 2;
        let new_map: LocklessMap<K, V> = Self::with_capacity(new_capacity);
        
        for node in self.map.iter() {
            for i in node.list.iter() {
                new_map.insert(i.0, i.1);
            }
        }

        self.capacity = new_capacity;

        for node in new_map.map.iter() {
            for pair in node.list.iter() {

                let hash = newhasher::hash_adapter(&pair.0);
                let index = (hash as usize) % self.capacity;
                self.map[index].lock();
        
                let mut_self = unsafe { &mut *(self as *const LocklessMap< K, V> as *mut LocklessMap< K, V>)};
                let set = (pair.0, pair.1);
                       
                mut_self.map[index].list.push_front(set);
                self.map[index].unlock();
            }
        }

    }
*/
    pub fn size(&self) -> usize {
        let mut list_size = 0;

        for node in self.map.iter() {
            list_size += node.list.len(); 
        }
 
        list_size
    }

    pub fn contains_key(&self, key: &K) -> bool {  
        let hash = hash_adapter(&key);
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

    pub fn remove(&self, key: &K) -> Option<V>{ 
        let hash = hash_adapter(&key);
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

    pub fn put_data_into_map (&self, dataset: Vec<(K, V)>){
        for x in dataset {
            Self::insert(&self, x.0, x.1);
        }
    }

    pub fn print (&self) {
        for node in self.map.iter() {
            for i in node.list.iter() {
                println!("{:?}", i);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {    
        let hash = hash_adapter(&key);
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

    pub fn insert(&self, key: K, value: V) -> Option<&V> {
        // if self.capacity < Self::size(&self) {
        //     Self::update_capacity(&self);
        // }
        
        let hash = hash_adapter(&key);
        let index = (hash as usize) % self.capacity;
        self.map[index].lock();

        let mut_self = unsafe { &mut *(self as *const LocklessMap< K, V> as *mut LocklessMap< K, V>)};
        let set = (key, value);
        let mut list_index = 0;

        for x in self.map[index].list.iter() {                                      // iter in node list 
            if x.0 == set.0 {                                                                //if keys are equal  
                mut_self.map[index].list.remove(list_index);
                mut_self.map[index].list.push_front(set);
                self.map[index].unlock();
                return Some(&x.1);
            }
            list_index+=1;
        }

        mut_self.map[index].list.push_front(set);
        self.map[index].unlock();
        None
    }
}
