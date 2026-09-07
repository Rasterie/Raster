use raster_math::{IVec2, Vec2};

#[test]
fn arithmetic_is_component_wise() {
    let a = IVec2::new(3, 4);
    let b = IVec2::new(1, 2);

    assert_eq!(a + b, IVec2::new(4, 6));
    assert_eq!(a - b, IVec2::new(2, 2));
    assert_eq!(a * b, IVec2::new(3, 8));
    assert_eq!(a * 2, IVec2::new(6, 8));
    assert_eq!(-a, IVec2::new(-3, -4));
}

#[test]
fn assignment_operators_match_their_binary_forms() {
    let mut v = IVec2::new(1, 2);
    v += IVec2::new(3, 4);
    assert_eq!(v, IVec2::new(4, 6));

    v -= IVec2::new(1, 1);
    assert_eq!(v, IVec2::new(3, 5));

    v *= 2;
    assert_eq!(v, IVec2::new(6, 10));
}

/// World-to-tile conversion must floor, not truncate: -0.5 belongs to tile -1.
/// Truncation would fold the two cells either side of the origin together.
#[test]
fn floor_conversion_differs_from_truncation_below_zero() {
    assert_eq!(IVec2::from_vec2(Vec2::new(-0.5, -0.5)), IVec2::ZERO);
    assert_eq!(
        IVec2::from_vec2_floor(Vec2::new(-0.5, -0.5)),
        IVec2::new(-1, -1)
    );

    // Above zero the two agree.
    assert_eq!(IVec2::from_vec2(Vec2::new(1.7, 2.9)), IVec2::new(1, 2));
    assert_eq!(
        IVec2::from_vec2_floor(Vec2::new(1.7, 2.9)),
        IVec2::new(1, 2)
    );
}

#[test]
fn conversion_to_vec2_round_trips() {
    let v = IVec2::new(3, -4);
    assert_eq!(IVec2::from_vec2(v.as_vec2()), v);
}

#[test]
fn manhattan_counts_cardinal_steps() {
    assert_eq!(IVec2::ZERO.manhattan_distance(IVec2::new(3, 4)), 7);
    assert_eq!(IVec2::ZERO.manhattan_distance(IVec2::new(-3, -4)), 7);
}

#[test]
fn chebyshev_counts_steps_with_diagonals() {
    assert_eq!(IVec2::ZERO.chebyshev_distance(IVec2::new(3, 4)), 4);
    assert_eq!(IVec2::ZERO.chebyshev_distance(IVec2::new(-5, 2)), 5);
}

/// Chunk lookup depends on this: cell -1 with chunk size 16 is in chunk -1,
/// and its offset within that chunk is 15, not -1.
#[test]
fn euclidean_division_floors_towards_negative_infinity() {
    let size = IVec2::splat(16);

    assert_eq!(IVec2::new(-1, -1).div_euclid(size), IVec2::new(-1, -1));
    assert_eq!(IVec2::new(-1, -1).rem_euclid(size), IVec2::new(15, 15));

    assert_eq!(IVec2::new(16, 33).div_euclid(size), IVec2::new(1, 2));
    assert_eq!(IVec2::new(16, 33).rem_euclid(size), IVec2::new(0, 1));
}

#[test]
fn euclidean_remainder_is_never_negative() {
    let size = IVec2::splat(16);
    for i in -40..40 {
        let r = IVec2::splat(i).rem_euclid(size);
        assert!(r.x >= 0 && r.x < 16, "cell {i} gave offset {r}");
    }
}

#[test]
fn cardinal_neighbours_are_the_four_edge_sharing_cells() {
    assert_eq!(IVec2::CARDINAL.len(), 4);
    for n in IVec2::CARDINAL {
        assert_eq!(n.manhattan_distance(IVec2::ZERO), 1);
    }
}

/// Autotiling reads bit `i` as neighbour `i`, so the order is part of the API.
#[test]
fn neighbours_are_eight_distinct_cells_clockwise_from_up() {
    assert_eq!(IVec2::NEIGHBOURS.len(), 8);
    assert_eq!(IVec2::NEIGHBOURS[0], IVec2::UP);
    assert_eq!(IVec2::NEIGHBOURS[2], IVec2::RIGHT);
    assert_eq!(IVec2::NEIGHBOURS[4], IVec2::DOWN);
    assert_eq!(IVec2::NEIGHBOURS[6], IVec2::LEFT);

    for n in IVec2::NEIGHBOURS {
        assert_eq!(n.chebyshev_distance(IVec2::ZERO), 1);
    }
}

#[test]
fn signum_reports_direction_per_axis() {
    assert_eq!(IVec2::new(-5, 3).signum(), IVec2::new(-1, 1));
    assert_eq!(IVec2::ZERO.signum(), IVec2::ZERO);
}

#[test]
fn up_is_negative_y_in_screen_space() {
    assert_eq!(IVec2::UP, IVec2::new(0, -1));
    assert_eq!(IVec2::DOWN, IVec2::new(0, 1));
}

/// IVec2 keys chunk maps and tile lookups, so Hash and Eq must behave.
#[test]
fn works_as_a_hash_map_key() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(IVec2::new(1, 2), "a");
    map.insert(IVec2::new(-1, -2), "b");

    assert_eq!(map.get(&IVec2::new(1, 2)), Some(&"a"));
    assert_eq!(map.get(&IVec2::new(-1, -2)), Some(&"b"));
    assert_eq!(map.get(&IVec2::new(2, 1)), None);
}

#[test]
fn conversions_round_trip() {
    let v = IVec2::new(3, 4);
    assert_eq!(IVec2::from((3, 4)), v);
    assert_eq!(IVec2::from([3, 4]), v);
    assert_eq!(v.to_array(), [3, 4]);
}
