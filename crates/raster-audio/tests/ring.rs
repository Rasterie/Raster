use raster_audio::ring;

#[test]
fn what_is_written_comes_back_in_order() {
    let (mut producer, mut consumer) = ring(8);

    assert_eq!(producer.write(&[1.0, 2.0, 3.0]), 3);

    let mut out = [0.0f32; 3];
    assert_eq!(consumer.read(&mut out), 3);
    assert_eq!(out, [1.0, 2.0, 3.0]);
}

#[test]
fn an_empty_ring_reads_as_silence() {
    let (_producer, mut consumer) = ring(8);

    let mut out = [9.0f32; 4];
    assert_eq!(consumer.read(&mut out), 0);
    assert_eq!(out, [0.0; 4], "une lecture a vide doit donner du silence");
}

#[test]
fn a_partial_read_is_padded_with_silence() {
    let (mut producer, mut consumer) = ring(8);
    producer.write(&[1.0, 2.0]);

    let mut out = [9.0f32; 4];
    assert_eq!(consumer.read(&mut out), 2, "deux echantillons reels");
    assert_eq!(out, [1.0, 2.0, 0.0, 0.0]);
}

#[test]
fn a_full_ring_refuses_what_does_not_fit() {
    let (mut producer, mut consumer) = ring(4);

    assert_eq!(producer.write(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]), 4);
    assert_eq!(producer.free(), 0);

    // Le surplus est laisse : ecraser ce qui n'est pas lu ferait un saut.
    let mut out = [0.0f32; 4];
    consumer.read(&mut out);
    assert_eq!(out, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn reading_frees_room_to_write_again() {
    let (mut producer, mut consumer) = ring(4);
    producer.write(&[1.0, 2.0, 3.0, 4.0]);

    let mut out = [0.0f32; 2];
    consumer.read(&mut out);
    assert_eq!(producer.free(), 2);

    assert_eq!(producer.write(&[5.0, 6.0]), 2);

    let mut suite = [0.0f32; 4];
    assert_eq!(consumer.read(&mut suite), 4);
    assert_eq!(
        suite,
        [3.0, 4.0, 5.0, 6.0],
        "l'ordre s'est perdu au bouclage"
    );
}

#[test]
fn the_indices_wrap_without_losing_a_sample() {
    let (mut producer, mut consumer) = ring(3);
    let mut attendu = 0.0f32;
    let mut recu = 0.0f32;

    // Plusieurs tours complets : c'est au bouclage que les indices se
    // decalent, pas au premier passage.
    for _ in 0..50 {
        let bloc = [attendu, attendu + 1.0];
        let n = producer.write(&bloc);
        attendu += n as f32;

        let mut out = [0.0f32; 2];
        let lus = consumer.read(&mut out);
        for &s in &out[..lus] {
            assert_eq!(s, recu, "un echantillon a saute");
            recu += 1.0;
        }
    }

    assert!(recu > 50.0, "trop peu d'echantillons ont circule : {recu}");
}

#[test]
fn a_ring_reports_what_it_holds() {
    let (mut producer, consumer) = ring(8);
    assert_eq!(producer.capacity(), 8);
    assert_eq!(producer.filled(), 0);
    assert_eq!(producer.free(), 8);

    producer.write(&[1.0, 2.0, 3.0]);
    assert_eq!(producer.filled(), 3);
    assert_eq!(consumer.filled(), 3);
    assert_eq!(producer.free(), 5);
}

#[test]
fn a_ring_carries_samples_across_threads() {
    let (mut producer, mut consumer) = ring(64);
    const TOTAL: usize = 10_000;

    let ecrivain = std::thread::spawn(move || {
        let mut ecrits = 0usize;
        while ecrits < TOTAL {
            let bloc: Vec<f32> = (ecrits..(ecrits + 16).min(TOTAL))
                .map(|i| i as f32)
                .collect();
            ecrits += producer.write(&bloc);
            std::thread::yield_now();
        }
    });

    let lecteur = std::thread::spawn(move || {
        let mut attendu = 0.0f32;
        let mut out = [0.0f32; 16];
        while (attendu as usize) < TOTAL {
            let n = consumer.read(&mut out);
            for &s in &out[..n] {
                assert_eq!(s, attendu, "les echantillons sont arrives desordonnes");
                attendu += 1.0;
            }
            if n == 0 {
                std::thread::yield_now();
            }
        }
        attendu as usize
    });

    ecrivain.join().unwrap();
    assert_eq!(lecteur.join().unwrap(), TOTAL);
}

#[test]
#[should_panic(expected = "cannot be empty")]
fn a_ring_of_zero_samples_is_refused() {
    let _ = ring(0);
}

/// Ce que fait `Output::pump`, sans peripherique : melanger juste ce qui tient
/// et pousser, frame stereo par frame stereo.
#[test]
fn pumping_fills_the_ring_without_splitting_a_stereo_frame() {
    use raster_audio::{Audio, Play, Sound};
    use raster_core::asset::AssetId;

    let (mut producer, mut consumer) = ring(9);
    let mut audio = Audio::new(44_100);
    let handle = audio.insert(
        AssetId::new("a.wav"),
        Sound::new(vec![1.0; 1000], 1, 44_100),
    );
    audio.play(handle, Play::new().looping(true)).unwrap();

    let mut scratch = [0.0f32; 16];

    // Neuf cases libres, mais une frame stereo en occupe deux : huit passent.
    let wanted = producer.free() & !1;
    assert_eq!(wanted, 8);
    audio.render(&mut scratch[..wanted]);
    assert_eq!(producer.write(&scratch[..wanted]), 8);

    let mut out = [0.0f32; 8];
    assert_eq!(consumer.read(&mut out), 8);
    assert!(
        out.iter().all(|s| *s > 0.0),
        "le melange n'a pas rempli la tranche : {out:?}"
    );
}

#[test]
fn a_full_ring_takes_nothing_more() {
    let (mut producer, _consumer) = ring(4);
    producer.write(&[1.0, 2.0, 3.0, 4.0]);

    assert_eq!(producer.free(), 0);
    assert_eq!(
        producer.write(&[5.0, 6.0]),
        0,
        "un tampon plein a accepte des donnees"
    );
}
