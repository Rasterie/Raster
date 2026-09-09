use raster_audio::{Audio, Bus, Mixer, Play, Sound, Spatial, pan_gains};
use raster_core::asset::AssetId;
use raster_math::Vec2;

/// Un son constant : chaque frame vaut `value`, ce qui rend le gain lisible
/// directement dans la sortie.
fn constant(value: f32, frames: usize) -> Sound {
    Sound::new(vec![value; frames], 1, 44_100)
}

fn audio_with(sound: Sound) -> (Audio, raster_core::asset::Handle<Sound>) {
    let mut audio = Audio::new(44_100);
    let handle = audio.insert(AssetId::new("test.wav"), sound);
    (audio, handle)
}

#[test]
fn a_silent_mix_writes_zeros() {
    let mut audio = Audio::new(44_100);
    let mut out = [1.0f32; 8];

    audio.render(&mut out);
    assert_eq!(out, [0.0; 8], "le melange doit effacer la tranche");
}

#[test]
fn a_played_sound_reaches_the_output() {
    let (mut audio, handle) = audio_with(constant(0.5, 4));
    audio.play(handle, Play::new()).unwrap();

    let mut out = [0.0f32; 8];
    audio.render(&mut out);

    // Pan centre : chaque cote recoit 0,5 x 0,707.
    let (gl, gr) = pan_gains(0.0);
    assert!((out[0] - 0.5 * gl).abs() < 1e-6, "gauche : {}", out[0]);
    assert!((out[1] - 0.5 * gr).abs() < 1e-6, "droite : {}", out[1]);
}

#[test]
fn a_finished_sound_frees_its_voice() {
    let (mut audio, handle) = audio_with(constant(0.5, 2));
    let id = audio.play(handle, Play::new()).unwrap();
    assert_eq!(audio.playing(), 1);

    // Huit frames pour un son qui en compte deux.
    let mut out = [0.0f32; 16];
    audio.render(&mut out);

    assert_eq!(audio.playing(), 0, "la voix aurait du se liberer");
    assert!(!audio.is_playing(id));
    assert_eq!(out[4], 0.0, "le son continue apres sa fin");
}

#[test]
fn a_looping_sound_never_finishes() {
    let (mut audio, handle) = audio_with(constant(0.5, 2));
    audio.play(handle, Play::new().looping(true)).unwrap();

    let mut out = [0.0f32; 16];
    audio.render(&mut out);

    assert_eq!(audio.playing(), 1);
    assert!(out[14].abs() > 0.0, "la boucle s'est arretee");
}

#[test]
fn two_voices_add_up() {
    let (mut audio, handle) = audio_with(constant(0.25, 4));
    audio.play(handle, Play::new()).unwrap();
    audio.play(handle, Play::new()).unwrap();

    let mut out = [0.0f32; 4];
    audio.render(&mut out);

    let (gl, _) = pan_gains(0.0);
    assert!(
        (out[0] - 2.0 * 0.25 * gl).abs() < 1e-6,
        "deux voix doivent s'additionner : {}",
        out[0]
    );
}

#[test]
fn a_bus_volume_scales_what_it_carries() {
    let (mut audio, handle) = audio_with(constant(1.0, 4));
    audio.mixer_mut().set(Bus::Music, 0.5);

    audio.play(handle, Play::new().on(Bus::Music)).unwrap();
    audio.play(handle, Play::new().on(Bus::Sfx)).unwrap();

    let mut out = [0.0f32; 2];
    audio.render(&mut out);

    let (gl, _) = pan_gains(0.0);
    assert!(
        (out[0] - 1.5 * gl).abs() < 1e-6,
        "music a 0,5 plus sfx a 1 : {}",
        out[0]
    );
}

#[test]
fn muting_silences_everything_without_losing_the_volumes() {
    let (mut audio, handle) = audio_with(constant(1.0, 4));
    audio.mixer_mut().set(Bus::Sfx, 0.8);
    audio.play(handle, Play::new()).unwrap();
    audio.mixer_mut().mute();

    let mut out = [0.0f32; 4];
    audio.render(&mut out);

    assert_eq!(out, [0.0; 4]);
    assert_eq!(audio.mixer().volume(Bus::Sfx), 0.8);
}

#[test]
fn a_volume_is_kept_within_range() {
    let mut mixer = Mixer::new();
    mixer.set(Bus::Sfx, -2.0);
    assert_eq!(
        mixer.volume(Bus::Sfx),
        0.0,
        "un volume negatif inverserait la phase"
    );

    mixer.set(Bus::Sfx, 50.0);
    assert_eq!(mixer.volume(Bus::Sfx), 2.0);
}

#[test]
fn a_stopped_voice_falls_silent() {
    let (mut audio, handle) = audio_with(constant(1.0, 100));
    let id = audio.play(handle, Play::new()).unwrap();

    assert!(audio.stop(id));
    assert!(!audio.is_playing(id));

    let mut out = [0.0f32; 4];
    audio.render(&mut out);
    assert_eq!(out, [0.0; 4]);
}

#[test]
fn stopping_a_finished_voice_does_not_hit_the_one_that_took_its_place() {
    let (mut audio, handle) = audio_with(constant(1.0, 2));
    let premier = audio.play(handle, Play::new()).unwrap();

    let mut out = [0.0f32; 8];
    audio.render(&mut out);
    assert!(!audio.is_playing(premier));

    let second = audio.play(handle, Play::new()).unwrap();

    assert!(
        !audio.stop(premier),
        "arreter une voix terminee a coupe celle qui a repris sa place"
    );
    assert!(audio.is_playing(second));
}

#[test]
fn stopping_a_bus_leaves_the_others_alone() {
    let (mut audio, handle) = audio_with(constant(1.0, 100));
    let musique = audio.play(handle, Play::new().on(Bus::Music)).unwrap();
    let effet = audio.play(handle, Play::new().on(Bus::Sfx)).unwrap();

    audio.stop_bus(Bus::Music);

    assert!(!audio.is_playing(musique));
    assert!(audio.is_playing(effet));
}

#[test]
fn the_quietest_voice_is_stolen_when_none_is_free() {
    let mut audio = Audio::with_voices(44_100, 2);
    let handle = audio.insert(AssetId::new("a.wav"), constant(1.0, 100));

    let fort = audio.play(handle, Play::new().volume(1.0)).unwrap();
    let faible = audio.play(handle, Play::new().volume(0.1)).unwrap();
    assert_eq!(audio.playing(), 2);

    let nouveau = audio.play(handle, Play::new()).unwrap();

    assert!(
        !audio.is_playing(faible),
        "la voix la plus faible aurait du partir"
    );
    assert!(audio.is_playing(fort), "la voix forte a ete volee");
    assert!(audio.is_playing(nouveau));
    assert_eq!(audio.playing(), 2);
}

#[test]
fn a_loop_is_never_stolen() {
    let mut audio = Audio::with_voices(44_100, 1);
    let handle = audio.insert(AssetId::new("a.wav"), constant(1.0, 100));

    let musique = audio
        .play(handle, Play::new().on(Bus::Music).looping(true).volume(0.1))
        .unwrap();

    // Aucune voix libre, et la seule occupee est une boucle : le declenchement
    // echoue plutot que de couper la musique.
    assert!(audio.play(handle, Play::new().volume(1.0)).is_none());
    assert!(audio.is_playing(musique));
}

#[test]
fn an_unknown_sound_plays_nothing() {
    let mut audio = Audio::new(44_100);
    let vide = audio.insert(AssetId::new("vide.wav"), Sound::silence(0, 44_100));

    assert!(audio.play(vide, Play::new()).is_none());
    assert_eq!(audio.playing(), 0);
}

#[test]
fn panning_keeps_the_power_constant() {
    for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        let (l, r) = pan_gains(pan);
        let puissance = l * l + r * r;
        assert!(
            (puissance - 1.0).abs() < 1e-6,
            "pan {pan} : puissance {puissance}"
        );
    }
}

#[test]
fn panning_hard_sends_everything_to_one_side() {
    let (l, r) = pan_gains(-1.0);
    assert!((l - 1.0).abs() < 1e-6 && r.abs() < 1e-6);

    let (l, r) = pan_gains(1.0);
    assert!(l.abs() < 1e-6 && (r - 1.0).abs() < 1e-6);
}

#[test]
fn a_pan_beyond_the_range_is_clamped() {
    assert_eq!(pan_gains(-5.0), pan_gains(-1.0));
    assert_eq!(pan_gains(5.0), pan_gains(1.0));
}

#[test]
fn a_sound_at_the_listener_plays_at_full_volume() {
    let spatial = Spatial::new(64.0, 512.0);
    let (volume, pan) = spatial.at(Vec2::ZERO, Vec2::ZERO);

    assert_eq!(volume, 1.0);
    assert_eq!(pan, 0.0);
}

#[test]
fn a_sound_beyond_the_far_distance_is_silent() {
    let spatial = Spatial::new(64.0, 512.0);
    let (volume, _) = spatial.at(Vec2::ZERO, Vec2::new(1000.0, 0.0));

    assert_eq!(volume, 0.0);
}

#[test]
fn volume_falls_off_between_near_and_far() {
    let spatial = Spatial::new(0.0, 100.0);

    let (proche, _) = spatial.at(Vec2::ZERO, Vec2::new(25.0, 0.0));
    let (milieu, _) = spatial.at(Vec2::ZERO, Vec2::new(50.0, 0.0));
    let (loin, _) = spatial.at(Vec2::ZERO, Vec2::new(75.0, 0.0));

    assert!((milieu - 0.5).abs() < 1e-6, "a mi-distance : {milieu}");
    assert!(
        proche > milieu && milieu > loin,
        "l'attenuation n'est pas monotone"
    );
}

#[test]
fn a_sound_to_the_right_is_panned_right() {
    let spatial = Spatial::new(0.0, 100.0);
    let (_, droite) = spatial.at(Vec2::ZERO, Vec2::new(50.0, 0.0));
    let (_, gauche) = spatial.at(Vec2::ZERO, Vec2::new(-50.0, 0.0));

    assert!(
        droite > 0.0,
        "un son a droite doit paner a droite : {droite}"
    );
    assert!(
        gauche < 0.0,
        "un son a gauche doit paner a gauche : {gauche}"
    );
}

#[test]
fn moving_the_listener_re_places_the_voices() {
    let (mut audio, handle) = audio_with(constant(1.0, 1000));
    audio.set_spatial(Spatial::new(0.0, 100.0));

    let id = audio
        .play(handle, Play::new().at(Vec2::new(50.0, 0.0)))
        .unwrap();

    let mut avant = [0.0f32; 2];
    audio.render(&mut avant);

    // L'auditeur se place sur le son : il doit devenir plus fort.
    audio.set_listener(Vec2::new(50.0, 0.0));
    audio.update_spatial();

    let mut apres = [0.0f32; 2];
    audio.render(&mut apres);

    let force_avant = avant[0].abs() + avant[1].abs();
    let force_apres = apres[0].abs() + apres[1].abs();
    assert!(
        force_apres > force_avant,
        "avant {force_avant}, apres {force_apres}"
    );
    assert!(audio.is_playing(id));
}

#[test]
fn a_voice_can_follow_something_that_moves() {
    let (mut audio, handle) = audio_with(constant(1.0, 1000));
    audio.set_spatial(Spatial::new(0.0, 100.0));

    let id = audio
        .play(handle, Play::new().at(Vec2::new(90.0, 0.0)))
        .unwrap();

    let mut loin = [0.0f32; 2];
    audio.render(&mut loin);

    assert!(audio.move_voice(id, Vec2::new(5.0, 0.0)));

    let mut proche = [0.0f32; 2];
    audio.render(&mut proche);

    assert!(
        proche[0].abs() + proche[1].abs() > loin[0].abs() + loin[1].abs(),
        "la voix ne s'est pas rapprochee"
    );
}

#[test]
fn a_stereo_sound_keeps_its_two_sides_apart() {
    // Gauche a 1, droite a 0 : si les canaux se melangeaient, les deux cotes
    // sortiraient identiques.
    let sound = Sound::new(vec![1.0, 0.0, 1.0, 0.0], 2, 44_100);
    let (mut audio, handle) = audio_with(sound);
    audio.play(handle, Play::new()).unwrap();

    let mut out = [0.0f32; 4];
    audio.render(&mut out);

    assert!(out[0] > 0.0, "le canal gauche est muet");
    assert_eq!(out[1], 0.0, "le canal droit devrait etre muet");
}

#[test]
fn a_long_render_keeps_its_voices_and_its_capacity() {
    // Des boucles : elles ne se terminent pas, donc toute voix disparue serait
    // un vrai defaut et non la fin normale d'un son.
    let (mut audio, handle) = audio_with(constant(0.5, 64));
    for _ in 0..8 {
        audio.play(handle, Play::new().looping(true)).unwrap();
    }

    let mut out = vec![0.0f32; 512];
    let capacite = out.capacity();

    for _ in 0..100 {
        audio.render(&mut out);
    }

    assert_eq!(audio.playing(), 8, "des voix ont disparu en cours de route");
    assert_eq!(
        out.capacity(),
        capacite,
        "la tranche de sortie a ete reallouee"
    );
    assert_eq!(out.len(), 512);
}

#[test]
fn a_stolen_voice_id_no_longer_controls_the_voice_that_replaced_it() {
    let mut audio = Audio::with_voices(44_100, 1);
    let handle = audio.insert(AssetId::new("a.wav"), constant(1.0, 1000));

    let vole = audio.play(handle, Play::new().volume(0.1)).unwrap();
    let nouveau = audio.play(handle, Play::new().volume(1.0)).unwrap();

    assert!(
        !audio.is_playing(vole),
        "l'identifiant vole se croit encore vivant"
    );
    assert!(
        !audio.stop(vole),
        "arreter une voix volee a coupe celle qui a pris sa place"
    );
    assert!(audio.is_playing(nouveau));
}
