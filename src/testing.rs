
use std::rand::{Rng, StdRng};
use std::collections::HashMap;

use test::Bencher;

use item_store::ItemStore;
use hamt::HamtMap;

macro_rules! assert_find(
    ($map:ident, $key:expr, None) => (
        assert!($map.find(&$key).is_none());
    );
    ($map:ident, $key:expr, $val:expr) => (
        match $map.find(&$key) {
            Some(&value) => {
                assert_eq!(value, $val);
            }
            _ => panic!()
        };
    );
);

static BENCH_FIND_COUNT: usize = 1000;
static BENCH_INSERT_COUNT: usize = 1000;

pub struct Test;

#[old_impl_check]
impl<IS> Test
    where IS: ItemStore<u64, u64> {

    pub fn test_insert(empty: HamtMap<u64, u64, IS>) {
        let map00 = empty;
        let (map01, new_entry01) = map00.clone().insert(1, 2);
        let (map10, new_entry10) = map00.clone().insert(2, 4);
        let (map11, new_entry11) = map01.clone().insert(2, 4);

        assert_find!(map00, 1, None);
        assert_find!(map00, 2, None);

        assert_find!(map01, 1, 2);
        assert_find!(map01, 2, None);

        assert_find!(map11, 1, 2);
        assert_find!(map11, 2, 4);

        assert_find!(map10, 1, None);
        assert_find!(map10, 2, 4);

        assert_eq!(new_entry01, true);
        assert_eq!(new_entry10, true);
        assert_eq!(new_entry11, true);

        assert_eq!(map00.len(), 0);
        assert_eq!(map01.len(), 1);
        assert_eq!(map10.len(), 1);
        assert_eq!(map11.len(), 2);
    }

    pub fn test_insert_ascending(empty: HamtMap<u64, u64, IS>) {
        let mut map = empty;

        for x in range(0u64, 1000u64) {
            assert_eq!(map.len(), x as usize);
            map = map.insert(x, x).0;
            assert_find!(map, x, x);
        }
    }

    pub fn test_insert_descending(empty: HamtMap<u64, u64, IS>) {
        let mut map = empty;

        for x in range(0u64, 1000u64) {
            let key = 999u64 - x;
            assert_eq!(map.len(), x as usize);
            map = map.insert(key, x).0;
            assert_find!(map, key, x);
        }
    }

    pub fn test_insert_overwrite(empty: HamtMap<u64, u64, IS>) {
        let (map1, new_entry1) = empty.clone().insert(1, 2);
        let (map2, new_entry2) = map1.clone().insert(1, 4);
        let (map3, new_entry3) = map2.clone().insert(1, 6);

        assert_find!(empty, 1, None);
        assert_find!(map1, 1, 2);
        assert_find!(map2, 1, 4);
        assert_find!(map3, 1, 6);

        assert_eq!(new_entry1, true);
        assert_eq!(new_entry2, false);
        assert_eq!(new_entry3, false);

        assert_eq!(empty.len(), 0);
        assert_eq!(map1.len(), 1);
        assert_eq!(map2.len(), 1);
        assert_eq!(map3.len(), 1);
    }

    pub fn test_remove(empty: HamtMap<u64, u64, IS>) {
        let (map00, _) = (empty
            .insert(1, 2)).0
            .insert(2, 4);

        let (map01, _) = map00.clone().remove(&1);
        let (map10, _) = map00.clone().remove(&2);
        let (map11, _) = map01.clone().remove(&2);

        assert_find!(map00, 1, 2);
        assert_find!(map00, 2, 4);

        assert_find!(map01, 1, None);
        assert_find!(map01, 2, 4);

        assert_find!(map11, 1, None);
        assert_find!(map11, 2, None);

        assert_find!(map10, 1, 2);
        assert_find!(map10, 2, None);

        assert_eq!(map00.len(), 2);
        assert_eq!(map01.len(), 1);
        assert_eq!(map10.len(), 1);
        assert_eq!(map11.len(), 0);
    }

    pub fn random_insert_remove_stress_test(empty: HamtMap<u64, u64, IS>) {
        let mut reference: HashMap<u64, u64> = HashMap::new();
        let mut rng = StdRng::new().ok().expect("Could not create random number generator");

        let mut map = empty;

        for _ in range(0us, 500000) {
            let value: u64 = rng.gen();

            if rng.gen_weighted_bool(2) {
                let ref_size_change = reference.remove(&value).is_some();
                let (map1, size_change) = map.remove(&value);
                assert_eq!(ref_size_change, size_change);
                assert_find!(map1, value, None);
                assert_eq!(reference.len(), map1.len());
                assert_eq!(reference.get(&value), map1.find(&value));
                map = map1;
            } else {
                let ref_size_change = reference.insert(value, value).is_none();
                let (map1, size_change) = map.insert(value, value);
                assert_eq!(ref_size_change, size_change);
                assert_find!(map1, value, value);
                assert_eq!(reference.len(), map1.len());
                assert_eq!(reference.get(&value), map1.find(&value));
                map = map1;
            }
        }
    }
}

fn create_random_std_hashmap(count: usize) -> HashMap<u64, u64> {
    let mut hashmap = HashMap::<u64, u64>::new();
    let mut rng = StdRng::new().ok().expect("Could not create random number generator");

    while hashmap.len() < count {
        let value = rng.gen();
        hashmap.insert(value, value);
    }

    return hashmap;
}


fn create_unique_values(count: usize) -> Vec<u64> {
    create_random_std_hashmap(count).keys().map(|x| *x).collect()
}

pub static mut results: [Option<u64>; 1000000] = [None; 1000000];

#[old_impl_check]
impl<IS> Test
    where IS: ItemStore<u64, u64> {

    fn create_random_map(empty: HamtMap<u64, u64, IS>, count: usize) -> (HamtMap<u64, u64, IS>, Vec<u64>) {
        let keys = create_unique_values(count);
        let mut map = empty;

        for &x in keys.iter() {
            map = map.plus(x, x);
        }

        return (map, keys);
    }

    pub fn bench_find(empty: HamtMap<u64, u64, IS>, count: usize, bh: &mut Bencher) {
        let (map, keys) = Test::create_random_map(empty, count);

        bh.iter(|| {
            for i in range(0us, BENCH_FIND_COUNT) {
                let val = keys[i % count];
                // lets make about of half the lookups fail
                let val = val + (i as u64 & 1);

                unsafe {
                    match map.find(&val) {
                        Some(&x) => results[i] = Some(x),
                        None => results[i] = None,
                    }
                }
            }
        })
    }

    pub fn bench_insert(empty: HamtMap<u64, u64, IS>, count: usize, bh: &mut Bencher) {
        let (map, keys) = Test::create_random_map(empty, count + BENCH_INSERT_COUNT);

        bh.iter(|| {
            let mut map1 = map.clone();

            for i in range(0us, BENCH_INSERT_COUNT) {
                let val = keys[count + i];
                map1 = map1.plus(val, val);
            }
        })
    }

    pub fn bench_remove(empty: HamtMap<u64, u64, IS>, count: usize, bh: &mut Bencher) {
        let (map, keys) = Test::create_random_map(empty, count);

        bh.iter(|| {
            let mut map = map.clone();

            for x in ::std::iter::range_step(0, count as usize, 2) {
                map = map.minus(&keys[x]);
            }
        })
    }
}

fn bench_find_hashmap(count: usize, bh: &mut Bencher) {
    let values = create_unique_values(count);
    let mut map = HashMap::new();

    for &x in values.iter() {
        map.insert(x, x);
    }

    bh.iter(|| {
        for i in range(0us, BENCH_FIND_COUNT) {
            let val = values[i % count];

            unsafe {
                match map.get(&val) {
                    Some(&x) => results[i] = Some(x),
                    None => results[i] = None,
                }
            }
        }
    })
}

fn bench_insert_hashmap(count: usize, bh: &mut Bencher) {
    let values = create_unique_values(count + BENCH_INSERT_COUNT);
    let mut map = HashMap::new();

    for &x in values.iter() {
        map.insert(x, x);
    }

    bh.iter(|| {
        let mut map1 = map.clone();

        for i in range(0us, BENCH_INSERT_COUNT) {
            let val = values[count + i];
            map1.insert(val, val);
        }
    })
}

fn bench_clone_hashmap(count: usize, bh: &mut Bencher) {
    let values = create_unique_values(count);
    let mut map = HashMap::new();

    for &x in values.iter() {
        map.insert(x, x);
    }

    bh.iter(|| {
        map.clone();
    })
}

fn bench_remove_hashmap(count: usize, bh: &mut Bencher) {
    let values = create_unique_values(count);
    let mut map = HashMap::new();

    for &x in values.iter() {
        map.insert(x, x);
    }

    bh.iter(|| {
        let mut map1 = map.clone();

        for x in ::std::iter::range_step(0, count, 2) {
            map1.remove(&values[x]);
        }
    })
}




//=-------------------------------------------------------------------------------------------------
// Bench std::HashMap::find()
//=-------------------------------------------------------------------------------------------------

#[bench]
fn std_hashmap_find_10(bh: &mut Bencher) {
    bench_find_hashmap(10, bh);
}

#[bench]
fn std_hashmap_find_1000(bh: &mut Bencher) {
    bench_find_hashmap(1000, bh);
}

#[bench]
fn std_hashmap_find_100000(bh: &mut Bencher) {
    bench_find_hashmap(100000, bh);
}



//=-------------------------------------------------------------------------------------------------
// Bench std::HashMap::insert()
//=-------------------------------------------------------------------------------------------------

#[bench]
fn std_hashmap_insert_10(bh: &mut Bencher) {
    bench_insert_hashmap(10, bh);
}

#[bench]
fn std_hashmap_insert_1000(bh: &mut Bencher) {
    bench_insert_hashmap(1000, bh);
}

#[bench]
fn std_hashmap_insert_100000(bh: &mut Bencher) {
    bench_insert_hashmap(100000, bh);
}

#[bench]
fn std_hashmap_clone_10(bh: &mut Bencher) {
    bench_clone_hashmap(10, bh);
}

#[bench]
fn std_hashmap_clone_1000(bh: &mut Bencher) {
    bench_clone_hashmap(1000, bh);
}

#[bench]
fn std_hashmap_clone_100000(bh: &mut Bencher) {
    bench_clone_hashmap(100000, bh);
}



//=-------------------------------------------------------------------------------------------------
// Bench std::HashMap::remove()
//=-------------------------------------------------------------------------------------------------

#[bench]
fn std_hashmap_remove_10(bh: &mut Bencher) {
    bench_remove_hashmap(10, bh);
}

#[bench]
fn std_hashmap_remove_1000(bh: &mut Bencher) {
    bench_remove_hashmap(1000, bh);
}

#[bench]
fn std_hashmap_remove_10000(bh: &mut Bencher) {
    bench_remove_hashmap(10000, bh);
}
