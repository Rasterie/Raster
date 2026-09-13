use raster_ed_sprite::sheet::{self, Layout, Onion, SheetError};
use raster_ed_visual::Frame;
use raster_math::IVec2;
use raster_render::Colour;

fn at(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

/// Une frame marquee d'une couleur unique, pour la reconnaitre.
fn marked(n: u8) -> Frame {
    Frame::filled(4, 4, Colour::rgb(f32::from(n) / 255.0, 0.0, 0.0))
}

#[test]
fn a_row_puts_the_frames_side_by_side() {
    let sheet = sheet::pack(&[marked(1), marked(2), marked(3)], Layout::Row, 0).unwrap();

    assert_eq!(sheet.width(), 12);
    assert_eq!(sheet.height(), 4);
}

#[test]
fn a_column_stacks_them() {
    let sheet = sheet::pack(&[marked(1), marked(2)], Layout::Column, 0).unwrap();

    assert_eq!(sheet.width(), 4);
    assert_eq!(sheet.height(), 8);
}

#[test]
fn a_grid_wraps_at_the_column_count() {
    let frames: Vec<Frame> = (0..5).map(marked).collect();
    let sheet = sheet::pack(&frames, Layout::Grid, 2).unwrap();

    // Cinq frames sur deux colonnes : trois rangees.
    assert_eq!(sheet.width(), 8);
    assert_eq!(sheet.height(), 12);
}

#[test]
fn frames_land_in_the_right_order() {
    let sheet = sheet::pack(&[marked(10), marked(20)], Layout::Row, 0).unwrap();

    let premier = sheet.get(at(0, 0)).unwrap().to_array()[0];
    let second = sheet.get(at(4, 0)).unwrap().to_array()[0];

    assert!(premier < second, "les frames sont dans le desordre");
}

#[test]
fn packing_nothing_is_refused() {
    assert_eq!(sheet::pack(&[], Layout::Row, 0), Err(SheetError::Empty));
}

#[test]
fn frames_of_different_sizes_are_refused() {
    // Mieux vaut refuser qu'assembler une feuille decalee.
    let frames = vec![Frame::new(4, 4), Frame::new(8, 8)];
    assert_eq!(
        sheet::pack(&frames, Layout::Row, 0),
        Err(SheetError::Mismatched)
    );
}

#[test]
fn a_sheet_cuts_back_into_the_same_frames() {
    let frames: Vec<Frame> = (1..=4).map(marked).collect();
    let sheet = sheet::pack(&frames, Layout::Grid, 2).unwrap();

    let relues = sheet::unpack(&sheet, 4, 4).unwrap();

    assert_eq!(relues.len(), 4);
    for (avant, apres) in frames.iter().zip(relues.iter()) {
        assert_eq!(
            avant.pixels(),
            apres.pixels(),
            "une frame a change en chemin"
        );
    }
}

#[test]
fn a_sheet_that_does_not_divide_evenly_is_refused() {
    let sheet = Frame::new(10, 10);
    assert_eq!(sheet::unpack(&sheet, 4, 4), Err(SheetError::Mismatched));
}

#[test]
fn a_zero_sized_frame_is_refused() {
    assert_eq!(
        sheet::unpack(&Frame::new(8, 8), 0, 4),
        Err(SheetError::Empty)
    );
}

// --- Pelure d'oignon ---

#[test]
fn onion_skinning_is_off_by_default() {
    let onion = Onion::default();

    assert!(!onion.enabled);
    assert_eq!(onion.opacity_at(-1), 0.0);
}

#[test]
fn the_current_frame_is_never_ghosted() {
    let onion = Onion {
        enabled: true,
        ..Onion::default()
    };

    assert_eq!(
        onion.opacity_at(0),
        0.0,
        "la frame courante se dessine normalement"
    );
}

#[test]
fn a_nearby_frame_shows_through() {
    let onion = Onion {
        enabled: true,
        before: 2,
        opacity: 0.4,
        ..Onion::default()
    };

    assert_eq!(onion.opacity_at(-1), 0.4);
    // Chaque frame plus loin s'efface, sinon une longue animation devient
    // une bouillie grise.
    assert!((onion.opacity_at(-2) - 0.2).abs() < 1e-6);
}

#[test]
fn a_frame_beyond_the_reach_is_invisible() {
    let onion = Onion {
        enabled: true,
        before: 1,
        after: 0,
        ..Onion::default()
    };

    assert_eq!(onion.opacity_at(-2), 0.0);
    assert_eq!(onion.opacity_at(1), 0.0, "rien apres si `after` est nul");
}

#[test]
fn before_and_after_are_tinted_differently() {
    // La couleur dit le sens du temps.
    let avant = Onion::tint(-1).to_array();
    let apres = Onion::tint(1).to_array();

    assert!(avant[2] > avant[0], "avant doit tirer vers le froid");
    assert!(apres[0] > apres[2], "apres doit tirer vers le chaud");
}
