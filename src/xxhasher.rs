

pub mod hasher {
   
    type  Hash = u64;
/* 
    pub trait Hashable {
        fn xxhash(&self) -> Hash ;
    }
    
    impl Hashable for u32 {
        fn xxhash(&self) -> Hash {
        
            const PRIME_1: u64 = 11_400_714_785_074_694_791;
            const PRIME_2: u64 = 14_029_467_366_897_019_727;
            const PRIME_3: u64 = 1_609_587_929_392_839_161;
            const PRIME_4: u64 = 9_650_029_242_287_828_579;
            const PRIME_5: u64 = 2_870_177_450_012_600_261;

            let mut k1 = *self as u64;
            let mut h64: u64 = 0x9e3779b97f4a7c13u64;
            h64 = h64.wrapping_add(PRIME_5);
            h64 = h64.wrapping_add(8);

            k1 = k1.wrapping_mul(PRIME_2);
            k1 = k1.rotate_left(31);
            k1 = k1.wrapping_mul(PRIME_1);
            h64 ^= k1;
            h64 = h64.rotate_left(27);
            h64 = h64.wrapping_mul(PRIME_1);
            h64 = h64.wrapping_add(PRIME_4);

            h64 ^= h64 >> 33;
            h64 = h64.wrapping_mul(PRIME_2);
            h64 ^= h64 >> 29;
            h64 = h64.wrapping_mul(PRIME_3);
            h64 ^= h64 >> 32;
            h64  
        }
    }*/
    
    pub fn xxhash(mut k1: u64) -> Hash {
        
        const PRIME_1: u64 = 11_400_714_785_074_694_791;
        const PRIME_2: u64 = 14_029_467_366_897_019_727;
        const PRIME_3: u64 = 1_609_587_929_392_839_161;
        const PRIME_4: u64 = 9_650_029_242_287_828_579;
        const PRIME_5: u64 = 2_870_177_450_012_600_261;

        let mut h64: u64 = 0x9e3779b97f4a7c13u64;
        h64 = h64.wrapping_add(PRIME_5);
        h64 = h64.wrapping_add(8);

        k1 = k1.wrapping_mul(PRIME_2);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(PRIME_1);
        h64 ^= k1;
        h64 = h64.rotate_left(27);
        h64 = h64.wrapping_mul(PRIME_1);
        h64 = h64.wrapping_add(PRIME_4);

        h64 ^= h64 >> 33;
        h64 = h64.wrapping_mul(PRIME_2);
        h64 ^= h64 >> 29;
        h64 = h64.wrapping_mul(PRIME_3);
        h64 ^= h64 >> 32;
        h64  
    }

    pub fn generic_xxhash<K: Sized> (value: &K) -> Hash
    where
        K: std::fmt::Debug + Sized,
        [u64 ; std::mem::size_of::<K>() / std::mem::size_of::<u64>()]: Sized
    {
        //assert!(std::mem::size_of::<K>() % std::mem::size_of::<u64>() == 0);
        //println!("{:?} / {:?}", std::mem::size_of::<K>(), std::mem::size_of::<u64>());
        let mut hash = 0;
        
        if std::mem::size_of::<K>() / std::mem::size_of::<u64>() > 0 {
            let array = unsafe {&*(value as *const K as *const [u64 ; std::mem::size_of::<K>() / std::mem::size_of::<u64>()])};
            
            for a in array {
                hash ^= xxhash(*a);
            }
        } else {
            let value = unsafe {&*(value as *const K as *const u64 )};
            hash = xxhash(*value);
        }

        hash
    }
}
