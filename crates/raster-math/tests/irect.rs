use raster_math::{IRect, IVec2};

#[test]
fn constructors_agree() {
    let a = IRect::new(1, 2, 3, 4);
    assert_eq!(
        a,
        IRect::from_position_size(IVec2::new(1, 2), IVec2::new(3, 4))
    );
    assert_eq!(a, IRect::from_corners(IVec2::new(1, 2), IVec2::new(4, 6)));
}

#[test]
fn area_counts_cells() {
    assert_eq!(IRect::new(0, 0, 3, 4).area(), 12);
    assert_eq!(IRect::new(0, 0, 0, 5).area(), 0);
    assert_eq!(IRect::new(0, 0, -3, 4).area(), 0);
}

/// Half-open edges, exactly as for the float version: the far edge is one past
/// the last cell.
#[test]
fn contains_excludes_the_far_edge() {
    let r = IRect::new(0, 0, 3, 3);

    assert!(r.contains(IVec2::new(0, 0)));
    assert!(r.contains(IVec2::new(2, 2)));
    assert!(!r.contains(IVec2::new(3, 0)));
    assert!(!r.contains(IVec2::new(0, 3)));
    assert!(!r.contains(IVec2::new(-1, 0)));
}

#[test]
fn adjacent_rects_never_share_a_cell() {
    let left = IRect::new(0, 0, 4, 4);
    let right = IRect::new(4, 0, 4, 4);

    assert!(!left.intersects(right));
    assert!(!left.contains(IVec2::new(4, 0)));
    assert!(right.contains(IVec2::new(4, 0)));
}

#[test]
fn intersection_is_the_overlap() {
    let a = IRect::new(0, 0, 4, 4);
    let b = IRect::new(2, 2, 4, 4);

    assert!(a.intersects(b));
    assert_eq!(a.intersection(b), Some(IRect::new(2, 2, 2, 2)));
}

#[test]
fn empty_rects_intersect_nothing() {
    let empty = IRect::new(2, 2, 0, 0);
    let other = IRect::new(0, 0, 10, 10);

    assert!(empty.is_empty());
    assert!(!empty.intersects(other));
    assert_eq!(other.intersection(empty), None);
}

#[test]
fn union_contains_both() {
    let a = IRect::new(0, 0, 2, 2);
    let b = IRect::new(5, 5, 2, 2);
    assert_eq!(a.union(b), IRect::new(0, 0, 7, 7));
}

#[test]
fn expanding_grows_on_every_side() {
    assert_eq!(IRect::new(2, 2, 4, 4).expanded(1), IRect::new(1, 1, 6, 6));
}

#[test]
fn cells_yields_every_cell_exactly_once() {
    let r = IRect::new(1, 1, 3, 2);
    let cells: Vec<_> = r.cells().collect();

    assert_eq!(cells.len(), r.area() as usize);
    for c in &cells {
        assert!(r.contains(*c), "{c} should be inside {r}");
    }

    let mut sorted = cells.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), cells.len(), "cells repeated");
}

/// Row-major order keeps chunk reads walking memory forwards instead of
/// jumping between rows.
#[test]
fn cells_iterates_row_by_row() {
    let cells: Vec<_> = IRect::new(0, 0, 2, 2).cells().collect();
    assert_eq!(
        cells,
        vec![
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(0, 1),
            IVec2::new(1, 1)
        ]
    );
}

#[test]
fn cells_of_an_empty_rect_yields_nothing() {
    assert_eq!(IRect::new(0, 0, 0, 5).cells().count(), 0);
    assert_eq!(IRect::new(0, 0, -3, 5).cells().count(), 0);
}
