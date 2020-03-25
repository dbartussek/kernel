use allocators::utils::bitset::BitSet;

#[test]
pub fn test_first_free() {
    const CAPACITY: usize = 1000;

    let mut set = BitSet::<{ CAPACITY }>::new();

    for i in 0..CAPACITY {
        assert_eq!(set.find_first_unset(), Some(i));
        let inserted = set.insert(i);
        assert!(inserted.is_some());
    }

    assert_eq!(set.find_first_unset(), None);
}

#[test]
pub fn test_insert_and_contains() {
    const CAPACITY: usize = 1000;

    let mut set = BitSet::<{ CAPACITY }>::new();

    for i in 0..CAPACITY {
        for t in 0..i {
            assert_eq!(set.contains(t), Some(true));
        }
        for t in i..CAPACITY {
            assert_eq!(set.contains(t), Some(false));
        }

        let inserted = set.insert(i);
        assert!(inserted.is_some());
    }

    assert_eq!(set.find_first_unset(), None);
}
