use raster_math::{Rect, Vec2};
use raster_ui::layout::{self, Anchor, Axis, Size};

const AREA: Rect = Rect {
    position: Vec2 { x: 0.0, y: 0.0 },
    size: Vec2 { x: 100.0, y: 60.0 },
};

#[test]
fn fixed_sizes_are_served_exactly() {
    let parts = layout::stack(
        AREA,
        Axis::Vertical,
        0.0,
        &[Size::Fixed(10.0), Size::Fixed(20.0)],
    );

    assert_eq!(parts[0].size.y, 10.0);
    assert_eq!(parts[1].size.y, 20.0);
    assert_eq!(
        parts[1].position.y, 10.0,
        "le second doit suivre le premier"
    );
}

#[test]
fn growing_children_share_what_is_left() {
    let parts = layout::stack(
        AREA,
        Axis::Vertical,
        0.0,
        &[Size::Fixed(20.0), Size::Grow(1.0), Size::Grow(1.0)],
    );

    assert_eq!(parts[1].size.y, 20.0);
    assert_eq!(parts[2].size.y, 20.0);
}

#[test]
fn weights_split_the_remainder_in_proportion() {
    let parts = layout::stack(
        AREA,
        Axis::Vertical,
        0.0,
        &[Size::Grow(1.0), Size::Grow(3.0)],
    );

    assert_eq!(parts[0].size.y, 15.0);
    assert_eq!(parts[1].size.y, 45.0);
}

#[test]
fn gaps_come_out_of_the_free_space() {
    let parts = layout::stack(
        AREA,
        Axis::Vertical,
        10.0,
        &[Size::Grow(1.0), Size::Grow(1.0)],
    );

    // Soixante moins dix d'espace, partages en deux.
    assert_eq!(parts[0].size.y, 25.0);
    assert_eq!(parts[1].position.y, 35.0);
}

#[test]
fn a_horizontal_stack_splits_the_width() {
    let parts = layout::stack(
        AREA,
        Axis::Horizontal,
        0.0,
        &[Size::Grow(1.0), Size::Grow(1.0)],
    );

    assert_eq!(parts[0].size.x, 50.0);
    assert_eq!(parts[0].size.y, 60.0, "la hauteur reste celle du parent");
    assert_eq!(parts[1].position.x, 50.0);
}

#[test]
fn children_that_ask_for_more_than_there_is_get_nothing_extra() {
    // Deux tailles fixes de 80 dans 100 : rien ne doit devenir negatif.
    let parts = layout::stack(
        AREA,
        Axis::Vertical,
        0.0,
        &[Size::Fixed(80.0), Size::Fixed(80.0)],
    );

    assert_eq!(parts[0].size.y, 80.0);
    assert!(parts[1].size.y >= 0.0);
}

#[test]
fn an_empty_stack_returns_nothing() {
    assert!(layout::stack(AREA, Axis::Vertical, 4.0, &[]).is_empty());
}

#[test]
fn a_lone_grow_takes_everything() {
    let parts = layout::stack(AREA, Axis::Vertical, 0.0, &[Size::Grow(1.0)]);
    assert_eq!(parts[0].size.y, 60.0);
}

#[test]
fn insetting_shrinks_on_every_side() {
    let inner = layout::inset(AREA, 5.0);

    assert_eq!(inner.position, Vec2::new(5.0, 5.0));
    assert_eq!(inner.size, Vec2::new(90.0, 50.0));
}

#[test]
fn insetting_more_than_half_never_inverts() {
    // Un rectangle retourne dessinerait n'importe ou.
    let inner = layout::inset(Rect::new(0.0, 0.0, 10.0, 10.0), 50.0);

    assert!(inner.size.x >= 0.0 && inner.size.y >= 0.0, "{inner:?}");
}

#[test]
fn anchors_reach_every_corner() {
    let size = Vec2::new(10.0, 10.0);
    let margin = Vec2::ZERO;

    let hg = Anchor::TopLeft.place(AREA, size, margin);
    assert_eq!(hg.position, Vec2::ZERO);

    let hd = Anchor::TopRight.place(AREA, size, margin);
    assert_eq!(hd.position, Vec2::new(90.0, 0.0));

    let bg = Anchor::BottomLeft.place(AREA, size, margin);
    assert_eq!(bg.position, Vec2::new(0.0, 50.0));

    let bd = Anchor::BottomRight.place(AREA, size, margin);
    assert_eq!(bd.position, Vec2::new(90.0, 50.0));
}

#[test]
fn the_centre_anchor_centres() {
    let placed = Anchor::Centre.place(AREA, Vec2::new(20.0, 20.0), Vec2::ZERO);
    assert_eq!(placed.position, Vec2::new(40.0, 20.0));
}

#[test]
fn a_margin_always_pushes_inwards() {
    let size = Vec2::new(10.0, 10.0);
    let margin = Vec2::new(4.0, 4.0);

    let hg = Anchor::TopLeft.place(AREA, size, margin);
    assert_eq!(hg.position, Vec2::new(4.0, 4.0));

    // En haut a droite, la marge doit ecarter du bord droit, pas du gauche.
    let hd = Anchor::TopRight.place(AREA, size, margin);
    assert_eq!(hd.position, Vec2::new(86.0, 4.0));

    let bd = Anchor::BottomRight.place(AREA, size, margin);
    assert_eq!(bd.position, Vec2::new(86.0, 46.0));
}

#[test]
fn a_centred_anchor_ignores_the_margin() {
    let sans = Anchor::Centre.place(AREA, Vec2::new(10.0, 10.0), Vec2::ZERO);
    let avec = Anchor::Centre.place(AREA, Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0));

    assert_eq!(sans.position, avec.position, "une marge ne decentre pas");
}

#[test]
fn everything_placed_stays_inside() {
    let size = Vec2::new(10.0, 10.0);
    for anchor in [
        Anchor::TopLeft,
        Anchor::TopCentre,
        Anchor::TopRight,
        Anchor::CentreLeft,
        Anchor::Centre,
        Anchor::CentreRight,
        Anchor::BottomLeft,
        Anchor::BottomCentre,
        Anchor::BottomRight,
    ] {
        let placed = anchor.place(AREA, size, Vec2::new(2.0, 2.0));
        assert!(
            placed.position.x >= 0.0
                && placed.position.y >= 0.0
                && placed.position.x + size.x <= AREA.size.x
                && placed.position.y + size.y <= AREA.size.y,
            "{anchor:?} sort de la zone : {placed:?}"
        );
    }
}
