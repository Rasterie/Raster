use raster_math::Rng;

/// The reason this generator exists rather than the standard library's: world
/// generation and replays need the same seed to give the same result, every
/// run and every platform.
#[test]
fn the_same_seed_gives_the_same_sequence() {
    let a: Vec<u32> = (0..50).map(|_| Rng::new(42).next_u32()).take(1).collect();
    let b: Vec<u32> = (0..50).map(|_| Rng::new(42).next_u32()).take(1).collect();
    assert_eq!(a, b);

    let mut x = Rng::new(12345);
    let mut y = Rng::new(12345);
    for i in 0..1000 {
        assert_eq!(x.next_u32(), y.next_u32(), "diverged at {i}");
    }
}

#[test]
fn different_seeds_give_different_sequences() {
    let mut a = Rng::new(1);
    let mut b = Rng::new(2);
    let differences = (0..100).filter(|_| a.next_u32() != b.next_u32()).count();
    assert!(
        differences > 90,
        "sequences too similar: {differences}/100 differed"
    );
}

/// Systems seeded from one world seed must not produce correlated sequences.
#[test]
fn streams_decorrelate_the_same_seed() {
    let mut a = Rng::with_stream(42, 0);
    let mut b = Rng::with_stream(42, 1);
    let differences = (0..100).filter(|_| a.next_u32() != b.next_u32()).count();
    assert!(
        differences > 90,
        "streams too similar: {differences}/100 differed"
    );
}

#[test]
fn floats_stay_in_the_unit_range() {
    let mut rng = Rng::new(7);
    for _ in 0..10_000 {
        let f = rng.next_f32();
        assert!((0.0..1.0).contains(&f), "out of range: {f}");
    }
}

#[test]
fn floats_are_roughly_uniform() {
    let mut rng = Rng::new(7);
    let mut buckets = [0u32; 10];
    for _ in 0..100_000 {
        buckets[(rng.next_f32() * 10.0) as usize] += 1;
    }
    // Each bucket should hold about 10,000; allow a generous margin.
    for (i, count) in buckets.iter().enumerate() {
        assert!((8_000..12_000).contains(count), "bucket {i} held {count}");
    }
}

#[test]
fn ranges_respect_their_bounds() {
    let mut rng = Rng::new(99);
    for _ in 0..10_000 {
        let i = rng.range_i32(-5, 5);
        assert!((-5..5).contains(&i), "out of range: {i}");

        let f = rng.range_f32(2.0, 3.0);
        assert!((2.0..3.0).contains(&f), "out of range: {f}");
    }
}

#[test]
fn an_empty_range_yields_its_lower_bound() {
    let mut rng = Rng::new(1);
    assert_eq!(rng.range_i32(5, 5), 5);
    assert_eq!(rng.range_i32(5, 2), 5);
}

/// A naive `% bound` would favour the low end for bounds that do not divide
/// 2^32. This checks the rejection sampling actually works.
#[test]
fn bounded_integers_are_not_biased() {
    let mut rng = Rng::new(3);
    let mut buckets = [0u32; 3];
    for _ in 0..90_000 {
        buckets[rng.range_i32(0, 3) as usize] += 1;
    }
    for (i, count) in buckets.iter().enumerate() {
        assert!((29_000..31_000).contains(count), "bucket {i} held {count}");
    }
}

#[test]
fn chance_honours_its_probability() {
    let mut rng = Rng::new(5);
    let hits = (0..10_000).filter(|_| rng.chance(0.25)).count();
    assert!(
        (2_300..2_700).contains(&hits),
        "got {hits} hits out of 10000"
    );

    assert!(!rng.chance(0.0));
    assert!(rng.chance(1.0));
}

#[test]
fn unit_vectors_are_unit_length() {
    let mut rng = Rng::new(11);
    for _ in 0..1_000 {
        assert!((rng.unit_vec2().length() - 1.0).abs() < 1e-5);
    }
}

#[test]
fn pick_returns_an_element_or_none() {
    let mut rng = Rng::new(13);
    let items = [1, 2, 3, 4, 5];

    for _ in 0..100 {
        assert!(items.contains(rng.pick(&items).expect("non-empty")));
    }

    let empty: [i32; 0] = [];
    assert_eq!(rng.pick(&empty), None);
}

#[test]
fn shuffle_keeps_every_element() {
    let mut rng = Rng::new(17);
    let mut items: Vec<i32> = (0..100).collect();
    rng.shuffle(&mut items);

    assert_eq!(items.len(), 100);
    let mut sorted = items.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..100).collect::<Vec<_>>());
    assert_ne!(items, sorted, "shuffle left the order untouched");
}

#[test]
fn shuffle_handles_degenerate_slices() {
    let mut rng = Rng::new(19);
    let mut empty: Vec<i32> = vec![];
    rng.shuffle(&mut empty);
    assert!(empty.is_empty());

    let mut one = vec![42];
    rng.shuffle(&mut one);
    assert_eq!(one, vec![42]);
}

/// An unseeded generator differing between runs would make a bug impossible to
/// reproduce, so the default is fixed.
#[test]
fn the_default_is_deterministic() {
    assert_eq!(Rng::default().next_u32(), Rng::default().next_u32());
}
