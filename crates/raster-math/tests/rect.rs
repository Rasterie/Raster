use raster_math::{Rect, Vec2};

const EPS: f32 = 1e-5;

#[test]
fn constructors_agree() {
    let a = Rect::new(1.0, 2.0, 3.0, 4.0);
    let b = Rect::from_position_size(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
    let c = Rect::from_corners(Vec2::new(1.0, 2.0), Vec2::new(4.0, 6.0));

    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn from_corners_accepts_either_order() {
    let a = Rect::from_corners(Vec2::new(4.0, 6.0), Vec2::new(1.0, 2.0));
    assert_eq!(a, Rect::new(1.0, 2.0, 3.0, 4.0));
}

#[test]
fn from_center_size_centres_the_rectangle() {
    let r = Rect::from_center_size(Vec2::ZERO, Vec2::splat(10.0));
    assert_eq!(r, Rect::new(-5.0, -5.0, 10.0, 10.0));
    assert!(r.center().approx_eq(Vec2::ZERO, EPS));
}

#[test]
fn edges_and_corners_are_consistent() {
    let r = Rect::new(1.0, 2.0, 3.0, 4.0);

    assert_eq!(r.left(), 1.0);
    assert_eq!(r.top(), 2.0);
    assert_eq!(r.right(), 4.0);
    assert_eq!(r.bottom(), 6.0);

    assert_eq!(r.top_left(), Vec2::new(1.0, 2.0));
    assert_eq!(r.top_right(), Vec2::new(4.0, 2.0));
    assert_eq!(r.bottom_left(), Vec2::new(1.0, 6.0));
    assert_eq!(r.bottom_right(), Vec2::new(4.0, 6.0));

    assert_eq!(
        r.corners(),
        [
            r.top_left(),
            r.top_right(),
            r.bottom_right(),
            r.bottom_left()
        ]
    );
}

#[test]
fn area_is_width_times_height() {
    assert!((Rect::new(0.0, 0.0, 3.0, 4.0).area() - 12.0).abs() < EPS);
}

/// Half-open edges: two rectangles tiling the plane must never both claim a
/// point on their shared boundary.
#[test]
fn contains_includes_top_left_and_excludes_bottom_right() {
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);

    assert!(r.contains(Vec2::new(0.0, 0.0)));
    assert!(r.contains(Vec2::new(5.0, 5.0)));
    assert!(r.contains(Vec2::new(9.99, 9.99)));

    assert!(!r.contains(Vec2::new(10.0, 5.0)));
    assert!(!r.contains(Vec2::new(5.0, 10.0)));
    assert!(!r.contains(Vec2::new(-0.01, 5.0)));
}

#[test]
fn tiling_rectangles_never_share_a_point() {
    let left = Rect::new(0.0, 0.0, 10.0, 10.0);
    let right = Rect::new(10.0, 0.0, 10.0, 10.0);
    let boundary = Vec2::new(10.0, 5.0);

    assert!(!left.contains(boundary));
    assert!(right.contains(boundary));
}

#[test]
fn contains_rect_requires_full_containment() {
    let outer = Rect::new(0.0, 0.0, 10.0, 10.0);

    assert!(outer.contains_rect(Rect::new(2.0, 2.0, 3.0, 3.0)));
    assert!(outer.contains_rect(outer));
    assert!(!outer.contains_rect(Rect::new(5.0, 5.0, 10.0, 10.0)));
}

#[test]
fn overlapping_rectangles_intersect() {
    let a = Rect::new(0.0, 0.0, 10.0, 10.0);
    let b = Rect::new(5.0, 5.0, 10.0, 10.0);

    assert!(a.intersects(b));
    assert!(b.intersects(a));
    assert_eq!(a.intersection(b), Some(Rect::new(5.0, 5.0, 5.0, 5.0)));
}

/// A body resting exactly on the ground touches it without overlapping. If
/// touching counted as intersecting, every grounded body would report a
/// collision every frame.
#[test]
fn touching_edges_do_not_intersect() {
    let ground = Rect::new(0.0, 10.0, 10.0, 10.0);
    let body = Rect::new(0.0, 0.0, 10.0, 10.0);

    assert!(!body.intersects(ground));
    assert_eq!(body.intersection(ground), None);
}

#[test]
fn separated_rectangles_do_not_intersect() {
    let a = Rect::new(0.0, 0.0, 5.0, 5.0);
    let b = Rect::new(100.0, 100.0, 5.0, 5.0);

    assert!(!a.intersects(b));
    assert_eq!(a.intersection(b), None);
}

#[test]
fn union_contains_both() {
    let a = Rect::new(0.0, 0.0, 5.0, 5.0);
    let b = Rect::new(10.0, 10.0, 5.0, 5.0);
    let u = a.union(b);

    assert_eq!(u, Rect::new(0.0, 0.0, 15.0, 15.0));
    assert!(u.contains_rect(a));
    assert!(u.contains_rect(b));
}

#[test]
fn expanding_grows_on_every_side() {
    let r = Rect::new(5.0, 5.0, 10.0, 10.0);
    assert_eq!(r.expanded(2.0), Rect::new(3.0, 3.0, 14.0, 14.0));
    assert_eq!(
        r.expanded_by(Vec2::new(1.0, 2.0)),
        Rect::new(4.0, 3.0, 12.0, 14.0)
    );
}

#[test]
fn shrinking_past_zero_yields_an_empty_rect() {
    let r = Rect::new(0.0, 0.0, 4.0, 4.0);
    assert!(r.expanded(-3.0).is_empty());
}

#[test]
fn empty_rectangles_contain_and_intersect_nothing() {
    let empty = Rect::new(5.0, 5.0, 0.0, 0.0);
    let other = Rect::new(0.0, 0.0, 10.0, 10.0);

    assert!(empty.is_empty());
    assert!(!empty.contains(Vec2::new(5.0, 5.0)));
    assert!(!empty.intersects(other));
    assert_eq!(other.intersection(empty), None);
}

#[test]
fn normalizing_fixes_a_negative_size() {
    let backwards = Rect::from_position_size(Vec2::new(10.0, 10.0), Vec2::new(-4.0, -6.0));
    assert!(backwards.is_empty());

    let fixed = backwards.normalized();
    assert!(!fixed.is_empty());
    assert_eq!(fixed, Rect::new(6.0, 4.0, 4.0, 6.0));
}

#[test]
fn translating_keeps_the_size() {
    let r = Rect::new(1.0, 2.0, 3.0, 4.0);
    let moved = r.translated(Vec2::new(10.0, 20.0));

    assert_eq!(moved.position, Vec2::new(11.0, 22.0));
    assert_eq!(moved.size, r.size);
}

#[test]
fn closest_point_clamps_to_the_boundary() {
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);

    assert_eq!(r.closest_point(Vec2::new(5.0, 5.0)), Vec2::new(5.0, 5.0));
    assert_eq!(r.closest_point(Vec2::new(-5.0, 5.0)), Vec2::new(0.0, 5.0));
    assert_eq!(
        r.closest_point(Vec2::new(50.0, 50.0)),
        Vec2::new(10.0, 10.0)
    );
}

/// Snapping outward must never crop: a rectangle covering part of a pixel has
/// to grow to cover the whole pixel.
#[test]
fn snap_outward_never_crops() {
    let r = Rect::new(1.3, 2.7, 4.2, 3.1);
    let snapped = r.snap_outward();

    assert!(snapped.contains_rect(r), "{snapped} should contain {r}");
    assert_eq!(snapped, Rect::new(1.0, 2.0, 5.0, 4.0));
}

#[test]
fn display_is_readable() {
    assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).to_string(), "[(1, 2) 3x4]");
}
