//! Le decodage PNG, verifie sur de vrais fichiers plutot que sur des octets
//! ecrits a la main : c'est la conversion vers RGBA qui compte, et elle differe
//! selon le format d'origine.

use raster_render::{Image, ImageError};
use std::io::Cursor;

/// Encode une image et rend les octets du PNG, pour les redecoder ensuite.
fn encoder(
    pixels: &[u8],
    largeur: u32,
    hauteur: u32,
    couleur: png::ColorType,
    profondeur: png::BitDepth,
) -> Vec<u8> {
    let mut sortie = Vec::new();
    {
        let mut encodeur = png::Encoder::new(Cursor::new(&mut sortie), largeur, hauteur);
        encodeur.set_color(couleur);
        encodeur.set_depth(profondeur);
        let mut ecrivain = encodeur.write_header().expect("en-tete");
        ecrivain.write_image_data(pixels).expect("donnees");
    }
    sortie
}

fn decoder(octets: &[u8]) -> Result<Image, ImageError> {
    Image::decode(Cursor::new(octets))
}

#[test]
fn un_png_rgba_se_decode_tel_quel() {
    let pixels = vec![
        255, 0, 0, 255, // rouge opaque
        0, 255, 0, 128, // vert a moitie transparent
        0, 0, 255, 0, // bleu invisible
        255, 255, 255, 255, // blanc
    ];
    let png = encoder(&pixels, 2, 2, png::ColorType::Rgba, png::BitDepth::Eight);

    let image = decoder(&png).expect("decodage");
    assert_eq!(image.width, 2);
    assert_eq!(image.height, 2);
    assert_eq!(image.pixels, pixels);
}

/// Un PNG sans canal alpha doit devenir opaque, pas transparent : oublier ce
/// point rend toute l'image invisible.
#[test]
fn un_png_rgb_recoit_un_alpha_opaque() {
    let pixels = vec![255, 0, 0, 0, 255, 0];
    let png = encoder(&pixels, 2, 1, png::ColorType::Rgb, png::BitDepth::Eight);

    let image = decoder(&png).expect("decodage");
    assert_eq!(image.pixels, vec![255, 0, 0, 255, 0, 255, 0, 255]);
}

#[test]
fn un_png_en_niveaux_de_gris_se_repete_sur_les_trois_canaux() {
    let pixels = vec![0, 128, 255];
    let png = encoder(
        &pixels,
        3,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
    );

    let image = decoder(&png).expect("decodage");
    assert_eq!(
        image.pixels,
        vec![0, 0, 0, 255, 128, 128, 128, 255, 255, 255, 255, 255]
    );
}

#[test]
fn un_gris_avec_alpha_conserve_sa_transparence() {
    let pixels = vec![200, 255, 100, 0];
    let png = encoder(
        &pixels,
        2,
        1,
        png::ColorType::GrayscaleAlpha,
        png::BitDepth::Eight,
    );

    let image = decoder(&png).expect("decodage");
    assert_eq!(image.pixels, vec![200, 200, 200, 255, 100, 100, 100, 0]);
}

/// Le pixel art n'a aucun usage des 16 bits : on garde l'octet de poids fort.
#[test]
fn un_png_16_bits_est_ramene_a_8() {
    // 0xFFFF -> 0xFF, 0x8000 -> 0x80.
    let pixels: Vec<u8> = vec![0xFF, 0xFF, 0x80, 0x00, 0x00, 0x00];
    let png = encoder(&pixels, 1, 1, png::ColorType::Rgb, png::BitDepth::Sixteen);

    let image = decoder(&png).expect("decodage");
    assert_eq!(image.pixels, vec![0xFF, 0x80, 0x00, 255]);
}

#[test]
fn la_taille_est_rapportee_fidelement() {
    let pixels = vec![0u8; 7 * 3 * 4];
    let png = encoder(&pixels, 7, 3, png::ColorType::Rgba, png::BitDepth::Eight);

    let image = decoder(&png).expect("decodage");
    assert_eq!((image.width, image.height), (7, 3));
    assert_eq!(image.size(), raster_math::Vec2::new(7.0, 3.0));
    assert_eq!(image.pixels.len(), 7 * 3 * 4);
}

// --- erreurs ------------------------------------------------------------------

#[test]
fn un_fichier_qui_n_est_pas_un_png_est_refuse() {
    let erreur = decoder(b"ceci n'est pas une image").unwrap_err();
    assert!(matches!(erreur, ImageError::Decode(_)), "{erreur:?}");
    // Le message doit dire quoi faire, pas seulement que c'est casse.
    assert!(erreur.to_string().contains("PNG"), "{erreur}");
}

#[test]
fn un_fichier_tronque_est_refuse() {
    let pixels = vec![0u8; 16 * 16 * 4];
    let png = encoder(&pixels, 16, 16, png::ColorType::Rgba, png::BitDepth::Eight);

    // La moitie du fichier : l'en-tete passe, les donnees non.
    let erreur = decoder(&png[..png.len() / 2]).unwrap_err();
    assert!(matches!(erreur, ImageError::Decode(_)), "{erreur:?}");
}

#[test]
fn un_fichier_absent_donne_une_erreur_d_acces() {
    let erreur = Image::load("/ce/chemin/n/existe/pas.png").unwrap_err();
    assert!(matches!(erreur, ImageError::Io(_)), "{erreur:?}");
}

#[test]
fn les_messages_d_erreur_sont_lisibles() {
    let erreur = Image::load("/absent.png").unwrap_err();
    let message = erreur.to_string();

    assert!(!message.is_empty());
    assert!(message.contains("read"), "message peu clair : {message}");
}

// --- disque -------------------------------------------------------------------

#[test]
fn un_png_se_lit_depuis_le_disque() {
    let pixels = vec![10, 20, 30, 255, 40, 50, 60, 255];
    let png = encoder(&pixels, 2, 1, png::ColorType::Rgba, png::BitDepth::Eight);

    let chemin = std::env::temp_dir().join("raster_test_lecture.png");
    std::fs::write(&chemin, &png).expect("ecriture");

    let image = Image::load(&chemin).expect("lecture");
    assert_eq!(image.pixels, pixels);

    let _ = std::fs::remove_file(&chemin);
}
