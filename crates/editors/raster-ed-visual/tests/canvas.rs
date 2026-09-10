use raster_ed_visual::canvas::{self, Canvas};
use raster_math::{IVec2, Rect, Vec2};

const PANEL: Rect = Rect {
    position: Vec2 { x: 100.0, y: 50.0 },
    size: Vec2 { x: 640.0, y: 480.0 },
};

#[test]
fn the_top_left_of_the_panel_is_the_origin() {
    let canvas = Canvas::new(32, 32);
    assert_eq!(canvas.to_pixel(PANEL, PANEL.position), IVec2::ZERO);
}

#[test]
fn a_pixel_covers_a_square_of_the_zoom_size() {
    let mut canvas = Canvas::new(32, 32);
    canvas.zoom = 8;
    canvas.origin = Vec2::ZERO;

    let rect = canvas.pixel_rect(PANEL, IVec2::new(2, 3));
    assert_eq!(rect.size, Vec2::new(8.0, 8.0));
    assert_eq!(rect.position, PANEL.position + Vec2::new(16.0, 24.0));
}

#[test]
fn a_point_lands_in_the_pixel_that_contains_it() {
    let mut canvas = Canvas::new(32, 32);
    canvas.zoom = 10;
    canvas.origin = Vec2::ZERO;

    // Toute la case, du bord gauche au dernier point avant la suivante.
    assert_eq!(
        canvas.to_pixel(PANEL, PANEL.position + Vec2::new(0.0, 0.0)),
        IVec2::ZERO
    );
    assert_eq!(
        canvas.to_pixel(PANEL, PANEL.position + Vec2::new(9.9, 9.9)),
        IVec2::ZERO
    );
    assert_eq!(
        canvas.to_pixel(PANEL, PANEL.position + Vec2::new(10.0, 0.0)),
        IVec2::new(1, 0),
        "la moitie droite d'un pixel ne doit pas peindre le suivant"
    );
}

#[test]
fn screen_and_pixel_are_inverses() {
    let mut canvas = Canvas::new(64, 64);
    canvas.zoom = 4;
    canvas.origin = Vec2::new(3.0, 7.0);

    for pixel in [IVec2::ZERO, IVec2::new(10, 20), IVec2::new(63, 63)] {
        let back = canvas.to_pixel(PANEL, canvas.to_screen(PANEL, pixel));
        assert_eq!(back, pixel);
    }
}

#[test]
fn zooming_keeps_the_pixel_under_the_cursor() {
    let mut canvas = Canvas::new(64, 64);
    let curseur = PANEL.position + Vec2::new(300.0, 200.0);
    let avant = canvas.to_pixel(PANEL, curseur);

    canvas.zoom_at(PANEL, curseur, 1);
    let apres = canvas.to_pixel(PANEL, curseur);

    assert_eq!(apres, avant, "le pixel vise a bouge en zoomant");
}

#[test]
fn zoom_stays_within_bounds() {
    let mut canvas = Canvas::new(32, 32);

    for _ in 0..20 {
        canvas.zoom_at(PANEL, PANEL.position, 1);
    }
    assert_eq!(canvas.zoom, Canvas::MAX_ZOOM);

    for _ in 0..20 {
        canvas.zoom_at(PANEL, PANEL.position, -1);
    }
    assert_eq!(canvas.zoom, 1, "le zoom ne doit jamais tomber a zero");
}

#[test]
fn panning_moves_the_view_the_other_way() {
    let mut canvas = Canvas::new(32, 32);
    canvas.zoom = 4;
    canvas.origin = Vec2::new(10.0, 10.0);

    // Tirer la vue vers la droite montre ce qui est a gauche.
    canvas.pan(Vec2::new(8.0, 0.0));
    assert_eq!(canvas.origin.x, 8.0);
}

#[test]
fn fitting_shows_the_whole_image() {
    let mut canvas = Canvas::new(32, 32);
    canvas.fit(PANEL);

    let image = canvas.image_rect(PANEL);
    assert!(
        image.size.x <= PANEL.size.x + 0.01 && image.size.y <= PANEL.size.y + 0.01,
        "l'image deborde du panneau : {image:?}"
    );
    // Et le zoom reste entier.
    assert_eq!(canvas.zoom, 15, "480 / 32 = 15");
}

#[test]
fn fitting_a_huge_image_still_shows_something() {
    let mut canvas = Canvas::new(4096, 4096);
    canvas.fit(PANEL);

    assert!(canvas.zoom >= 1, "le zoom ne doit pas tomber a zero");
}

#[test]
fn centring_puts_the_image_in_the_middle() {
    let mut canvas = Canvas::new(32, 32);
    canvas.zoom = 4;
    canvas.centre(PANEL);

    let image = canvas.image_rect(PANEL);
    let ecart_gauche = image.position.x - PANEL.position.x;
    let ecart_droite = (PANEL.position.x + PANEL.size.x) - (image.position.x + image.size.x);

    assert!(
        (ecart_gauche - ecart_droite).abs() < 0.01,
        "{ecart_gauche} a gauche, {ecart_droite} a droite"
    );
}

#[test]
fn only_pixels_inside_the_image_count() {
    let canvas = Canvas::new(16, 16);

    assert!(canvas.contains(IVec2::ZERO));
    assert!(canvas.contains(IVec2::new(15, 15)));
    assert!(!canvas.contains(IVec2::new(16, 0)));
    assert!(!canvas.contains(IVec2::new(-1, 0)));
}

#[test]
fn the_visible_range_is_clamped_to_the_image() {
    let mut canvas = Canvas::new(8, 8);
    canvas.zoom = 1;
    canvas.origin = Vec2::ZERO;

    // Le panneau est bien plus grand que l'image : parcourir tout le panneau
    // ferait boucler sur des milliers de cases vides.
    let (first, last) = canvas.visible_pixels(PANEL);
    assert_eq!(first, IVec2::ZERO);
    assert_eq!(last, IVec2::new(8, 8));
}

#[test]
fn a_view_scrolled_past_the_image_yields_nothing_to_draw() {
    let mut canvas = Canvas::new(8, 8);
    canvas.origin = Vec2::new(100.0, 100.0);

    let (first, last) = canvas.visible_pixels(PANEL);
    assert!(first.x >= last.x || first.y >= last.y, "{first:?} {last:?}");
}

#[test]
fn the_grid_hides_itself_when_it_would_eat_the_image() {
    let mut canvas = Canvas::new(32, 32);

    canvas.zoom = 1;
    assert!(
        !canvas.grid_visible(),
        "a zoom 1, la grille mangerait l'image"
    );

    canvas.zoom = Canvas::GRID_FROM;
    assert!(canvas.grid_visible());

    canvas.show_grid = false;
    assert!(
        !canvas.grid_visible(),
        "l'utilisateur peut toujours la couper"
    );
}

#[test]
fn the_checker_alternates_in_screen_space() {
    // De taille constante quel que soit le zoom, sinon il deviendrait un motif
    // de l'image elle-meme.
    assert!(canvas::checker_light(0.0, 0.0));
    assert!(!canvas::checker_light(Canvas::CHECKER, 0.0));
    assert!(canvas::checker_light(Canvas::CHECKER, Canvas::CHECKER));
    // L'alternance continue en negatif : sans `rem_euclid`, le motif se
    // dedoublerait de part et d'autre de l'origine.
    assert!(!canvas::checker_light(-Canvas::CHECKER, 0.0));
    assert!(canvas::checker_light(-2.0 * Canvas::CHECKER, 0.0));
}

#[test]
fn a_canvas_cannot_be_zero_sized() {
    let canvas = Canvas::new(0, -5);
    assert_eq!(canvas.size, IVec2::new(1, 1));
}
