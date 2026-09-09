use crate::world::{Room, TILE};
use raster_math::Vec2;

/// Les salles du jeu, dans l'ordre.
///
/// Ecrites en dur plutot qu'en fichiers de scene : une salle est faite de
/// tuiles, et le format de scene decrit des acteurs.
#[must_use]
pub fn all() -> Vec<Room> {
    vec![first(), second(), third()]
}

/// Vingt colonnes sur onze lignes : la taille d'un ecran a 320x176.
const WIDTH: usize = 20;

fn spawn(column: usize, row: usize) -> Vec2 {
    Vec2::new(column as f32 * TILE + 2.0, row as f32 * TILE)
}

/// Apprend a courir et a sauter, sans rien qui puisse tuer.
///
/// Les etages montent de deux lignes a la fois : un saut franchit 2,7 tuiles,
/// donc chaque marche reste atteignable.
fn first() -> Room {
    Room {
        name: "Le seuil".to_owned(),
        tiles: vec![
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#..........======..#".to_owned(),
            "#..................#".to_owned(),
            "#.....======.......#".to_owned(),
            "#..................#".to_owned(),
            "#.=====............#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "####################".to_owned(),
        ],
        spawn: spawn(1, 9),
    }
}

/// Introduit un ennemi qui marche, et un pic a eviter.
fn second() -> Room {
    Room {
        name: "La faille".to_owned(),
        tiles: vec![
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#........=====.....#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#...=====..........#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "####################".to_owned(),
        ],
        spawn: spawn(1, 9),
    }
}

/// Demande de composer avec un voltigeur pour atteindre la clef.
fn third() -> Room {
    Room {
        name: "La clef de voute".to_owned(),
        tiles: vec![
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#..........======..#".to_owned(),
            "#..................#".to_owned(),
            "#....======........#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "#.=====............#".to_owned(),
            "#..................#".to_owned(),
            "#..................#".to_owned(),
            "####################".to_owned(),
        ],
        spawn: spawn(1, 9),
    }
}

/// Vérifie qu'une salle a la forme attendue.
///
/// # Errors
///
/// Décrit ce qui ne va pas : une salle mal formée dessinerait de travers.
pub fn check(room: &Room) -> Result<(), String> {
    if room.tiles.is_empty() {
        return Err(format!("`{}` n'a aucune ligne", room.name));
    }

    for (i, row) in room.tiles.iter().enumerate() {
        let n = row.chars().count();
        if n != WIDTH {
            return Err(format!(
                "`{}` ligne {i} : {n} colonnes au lieu de {WIDTH}",
                room.name
            ));
        }
        if let Some(c) = row.chars().find(|c| !matches!(c, '.' | '#' | '=')) {
            return Err(format!(
                "`{}` ligne {i} : caractere inconnu `{c}`",
                room.name
            ));
        }
    }

    Ok(())
}
