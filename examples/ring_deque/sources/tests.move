/// Test harness for ring_deque — exercises the real ring_deque module's public API.
module ring_deque::tests {
    use ring_deque::ring_deque::{Self, RingDeque};

    public entry fun test_end_to_end(_s: &signer) {
        let capacity = 5u64;
        let rd: RingDeque<u8> = ring_deque::new<u8>(capacity);
        assert!(ring_deque::capacity(&rd) == capacity, 0);
        assert!(ring_deque::length(&rd) == 0, 1);
        assert!(ring_deque::is_empty(&rd), 2);
        assert!(!ring_deque::is_full(&rd), 3);

        // Push back 9
        ring_deque::push_back(&mut rd, 9u8);
        assert!(ring_deque::length(&rd) == 1, 4);
        assert!(!ring_deque::is_empty(&rd), 5);
        assert!(*ring_deque::borrow_front(&rd) == 9, 6);
        assert!(*ring_deque::borrow_back(&rd) == 9, 7);

        // Push front 7, push back 8
        ring_deque::push_front(&mut rd, 7u8);
        ring_deque::push_back(&mut rd, 8u8);
        assert!(ring_deque::length(&rd) == 3, 8);

        // Mutate back to 8 (already 8, but exercises borrow_back_mut)
        *ring_deque::borrow_back_mut(&mut rd) = 8;
        assert!(*ring_deque::borrow_back(&rd) == 8, 9);

        // Mutate front to 5
        *ring_deque::borrow_front_mut(&mut rd) = 5;
        assert!(*ring_deque::borrow_front(&rd) == 5, 10);

        // Pop front (should be 5)
        assert!(ring_deque::pop_front(&mut rd) == 5, 11);
        assert!(*ring_deque::borrow_front(&rd) == 9, 12);
        assert!(*ring_deque::borrow_back(&rd) == 8, 13);

        // Pop front (should be 9), pop back (should be 8)
        assert!(ring_deque::pop_front(&mut rd) == 9, 14);
        assert!(ring_deque::pop_back(&mut rd) == 8, 15);
        assert!(ring_deque::is_empty(&rd), 16);

        // Fill to capacity: push front 5,4,3 and push back 6,7
        ring_deque::push_front(&mut rd, 5u8);
        ring_deque::push_front(&mut rd, 4u8);
        ring_deque::push_front(&mut rd, 3u8);
        ring_deque::push_back(&mut rd, 6u8);
        ring_deque::push_back(&mut rd, 7u8);
        assert!(ring_deque::length(&rd) == 5, 17);
        assert!(ring_deque::is_full(&rd), 18);

        // Pop all from back: 7,6,5,4,3
        assert!(ring_deque::pop_back(&mut rd) == 7, 19);
        assert!(ring_deque::pop_back(&mut rd) == 6, 20);
        assert!(ring_deque::pop_back(&mut rd) == 5, 21);
        assert!(ring_deque::pop_back(&mut rd) == 4, 22);
        assert!(ring_deque::pop_back(&mut rd) == 3, 23);

        // Push back 1,2,3 then pop front 1,2,3
        ring_deque::push_back(&mut rd, 1u8);
        ring_deque::push_back(&mut rd, 2u8);
        ring_deque::push_back(&mut rd, 3u8);
        assert!(ring_deque::length(&rd) == 3, 24);
        assert!(ring_deque::pop_front(&mut rd) == 1, 25);
        assert!(ring_deque::pop_front(&mut rd) == 2, 26);
        assert!(ring_deque::pop_front(&mut rd) == 3, 27);
        assert!(ring_deque::length(&rd) == 0, 28);
        assert!(ring_deque::is_empty(&rd), 29);
    }

    public entry fun test_single_element(_s: &signer) {
        let rd = ring_deque::new<u64>(1);
        ring_deque::push_back(&mut rd, 42u64);
        assert!(ring_deque::is_full(&rd), 0);
        assert!(*ring_deque::borrow_front(&rd) == 42, 1);
        assert!(*ring_deque::borrow_back(&rd) == 42, 2);
        assert!(ring_deque::pop_front(&mut rd) == 42, 3);
        assert!(ring_deque::is_empty(&rd), 4);
    }

    public entry fun test_wrap_around(_s: &signer) {
        // Capacity 3 — fill, drain, refill to force index wrapping
        let rd = ring_deque::new<u64>(3);
        ring_deque::push_back(&mut rd, 1);
        ring_deque::push_back(&mut rd, 2);
        ring_deque::push_back(&mut rd, 3);
        assert!(ring_deque::pop_front(&mut rd) == 1, 0);
        assert!(ring_deque::pop_front(&mut rd) == 2, 1);
        // Now front has wrapped. Push more:
        ring_deque::push_back(&mut rd, 4);
        ring_deque::push_back(&mut rd, 5);
        assert!(ring_deque::pop_front(&mut rd) == 3, 2);
        assert!(ring_deque::pop_front(&mut rd) == 4, 3);
        assert!(ring_deque::pop_front(&mut rd) == 5, 4);
        assert!(ring_deque::is_empty(&rd), 5);
    }

    public entry fun test_large_capacity(_s: &signer) {
        let rd = ring_deque::new<u64>(100);
        let i = 0u64;
        while (i < 100) {
            ring_deque::push_back(&mut rd, i);
            i = i + 1;
        };
        assert!(ring_deque::is_full(&rd), 0);
        let i = 0u64;
        while (i < 100) {
            assert!(ring_deque::pop_front(&mut rd) == i, i + 1);
            i = i + 1;
        };
        assert!(ring_deque::is_empty(&rd), 101);
    }
}
