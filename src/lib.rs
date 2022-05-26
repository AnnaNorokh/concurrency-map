#![feature(linked_list_remove)]
#![feature(generic_const_exprs)]
#![feature(test)]


pub(crate) mod concurent_map;
pub(crate) mod newhasher;
pub(crate) mod dataset_gen;
pub(crate) mod xxhasher;

pub use concurent_map::*;
pub use dataset_gen::*;


#[cfg(test)]
pub mod tests {

    use super::*;
    use rand::*;
    use std::thread;
    use std::sync::{Arc, Barrier};

    const THREADS: usize = 10;
    const ITERATIONS: usize = 1_000;
    const CAPACITY: usize = 100;
    const CAPACITY1: usize = 10_000;
    const CAPACITY2: usize = 1_000;

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
        let mut map = Arc::new(LocklessMap::<u32, String>::with_capacity(CAPACITY2));               //1_000
        let dataset = Arc::new(dataset_gen::dataset::create_ordered_dataset(CAPACITY1));          //10_000

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
                    }
                }

            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
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

#[cfg(test)]
mod benchmarks {

    extern crate test;
    use core::sync::atomic::AtomicBool;
    use super::*;
    use std::sync::{Arc, Barrier, atomic::Ordering};

    #[bench]
    fn insert_into_map(b: &mut test::Bencher) {
        let map: LocklessMap<u32,u32> = LocklessMap::with_capacity(1_000_000);

        let mut key: u32 = 0;
        b.iter(|| {
            test::black_box(map.insert(key, key));
            key += 1;
        });
        println!("did {} measurements", key); // key increases by one with each measurement
    }

    #[bench]
    fn insert_into_map_identical_keys(b: &mut test::Bencher) {
        let map: LocklessMap<u32,u32> = LocklessMap::with_capacity(1_000_000);

        let mut key: u32 = 0;
        b.iter(|| {
            test::black_box(map.insert(0, key));
            key += 1;
        });
        println!("did {} measurements", key); // key increases by one with each measurement
    }

    #[bench]
    fn insert_into_map_concurrent(b: &mut test::Bencher) {
        let map: Arc<LocklessMap<u32,u32>> = Arc::new(LocklessMap::with_capacity(1_000_000));

        const INSERTERS_CNT: usize = 2;

        let mut inserters = Vec::with_capacity(INSERTERS_CNT);
        let barrier = Arc::new(Barrier::new(INSERTERS_CNT + 1));
        let is_done = Arc::new(AtomicBool::new(false));

        for _ in 0..INSERTERS_CNT {
            let map = map.clone();
            let barrier = barrier.clone();
            let is_done = is_done.clone();
            inserters.push(std::thread::spawn(move || {
                barrier.wait(); // Wait until benchmark starts
                let mut key: u32 = 0;
                while !is_done.load(Ordering::Acquire) {
                    // Loop until benchmark stops
                    map.insert(key, key);
                }
            }));
        }

        barrier.wait(); // Tell inserter threads to start

        let mut key: u32 = 0;
        b.iter(|| {
            test::black_box(map.insert(key, key));
            key += 1;
        });

        is_done.store(true, Ordering::Release); // Tell inserter threads to stop
        for inserter in inserters {
            inserter.join().unwrap();
        }

        println!("did {} measurements", key); // key increases by one with each measurement
    }
}
