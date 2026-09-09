use raster_audio::{Sound, WavError, decode};

/// Assemble un WAV minimal, pour tester le decodeur sans fichier joint.
fn wav(channels: u16, sample_rate: u32, bits: u16, float: bool, data: &[u8]) -> Vec<u8> {
    let tag: u16 = if float { 3 } else { 1 };
    let block_align = channels * bits / 8;
    let byte_rate = sample_rate * u32::from(block_align);

    let mut fmt = Vec::new();
    fmt.extend_from_slice(&tag.to_le_bytes());
    fmt.extend_from_slice(&channels.to_le_bytes());
    fmt.extend_from_slice(&sample_rate.to_le_bytes());
    fmt.extend_from_slice(&byte_rate.to_le_bytes());
    fmt.extend_from_slice(&block_align.to_le_bytes());
    fmt.extend_from_slice(&bits.to_le_bytes());

    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
    out.extend_from_slice(&fmt);
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out
}

#[test]
fn a_16_bit_mono_wav_decodes_to_its_samples() {
    let data: Vec<u8> = [0i16, 16384, -16384, 32767]
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();

    let sound = decode(&wav(1, 44_100, 16, false, &data)).unwrap();

    assert_eq!(sound.channels(), 1);
    assert_eq!(sound.sample_rate(), 44_100);
    assert_eq!(sound.frames(), 4);
    assert!((sound.samples()[0] - 0.0).abs() < 1e-6);
    assert!(
        (sound.samples()[1] - 0.5).abs() < 1e-4,
        "{}",
        sound.samples()[1]
    );
    assert!((sound.samples()[2] + 0.5).abs() < 1e-4);
    assert!(sound.samples()[3] > 0.99);
}

#[test]
fn every_sample_stays_within_range() {
    // Les extremes de chaque format doivent rester entre -1 et 1, sinon le
    // melange sature.
    let data: Vec<u8> = [i16::MIN, i16::MAX]
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();
    let sound = decode(&wav(1, 44_100, 16, false, &data)).unwrap();

    for s in sound.samples() {
        assert!((-1.0..=1.0).contains(s), "echantillon hors bornes : {s}");
    }
}

#[test]
fn an_8_bit_wav_is_read_as_unsigned() {
    // Le 8 bits est le seul format WAV non signe : 128 est le zero.
    let sound = decode(&wav(1, 22_050, 8, false, &[128, 255, 0])).unwrap();

    assert!(sound.samples()[0].abs() < 1e-6, "128 doit etre le silence");
    assert!(sound.samples()[1] > 0.9);
    assert!(sound.samples()[2] < -0.9);
}

#[test]
fn a_24_bit_wav_keeps_its_sign() {
    // -1 et +1 en 24 bits, petit-boutiste.
    let data = [0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00];
    let sound = decode(&wav(1, 44_100, 24, false, &data)).unwrap();

    assert!(sound.samples()[0] < 0.0, "un negatif est devenu positif");
    assert!(sound.samples()[1] > 0.0);
}

#[test]
fn a_32_bit_float_wav_passes_through_unchanged() {
    let data: Vec<u8> = [0.0f32, 0.5, -0.25]
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();

    let sound = decode(&wav(1, 48_000, 32, true, &data)).unwrap();

    assert_eq!(sound.samples(), &[0.0, 0.5, -0.25]);
    assert_eq!(sound.sample_rate(), 48_000);
}

#[test]
fn a_stereo_wav_keeps_its_channels_interleaved() {
    let data: Vec<u8> = [1000i16, -1000, 2000, -2000]
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();

    let sound = decode(&wav(2, 44_100, 16, false, &data)).unwrap();

    assert_eq!(sound.channels(), 2);
    assert_eq!(
        sound.frames(),
        2,
        "quatre echantillons font deux frames stereo"
    );

    let (l, r) = sound.frame(0).unwrap();
    assert!(l > 0.0 && r < 0.0, "les canaux ont ete melanges");
}

#[test]
fn a_mono_sound_is_heard_on_both_sides() {
    let sound = Sound::new(vec![0.5], 1, 44_100);
    assert_eq!(sound.frame(0), Some((0.5, 0.5)));
}

#[test]
fn a_truncated_stereo_frame_is_dropped() {
    // Trois echantillons pour deux canaux : le dernier decalerait tout.
    let data: Vec<u8> = [1i16, 2, 3].iter().flat_map(|s| s.to_le_bytes()).collect();
    let sound = decode(&wav(2, 44_100, 16, false, &data)).unwrap();

    assert_eq!(sound.frames(), 1);
    assert_eq!(sound.samples().len(), 2);
}

#[test]
fn an_unknown_chunk_is_skipped() {
    let mut file = wav(1, 44_100, 16, false, &[0, 0, 0, 64]);

    // Un LIST insere entre `fmt ` et `data`, comme en produisent les editeurs.
    let mut avec_list = file[..36].to_vec();
    avec_list.extend_from_slice(b"LIST");
    avec_list.extend_from_slice(&4u32.to_le_bytes());
    avec_list.extend_from_slice(b"INFO");
    avec_list.extend_from_slice(&file[36..]);
    file = avec_list;

    let sound = decode(&file).expect("un morceau inconnu doit etre ignore");
    assert_eq!(sound.frames(), 2);
}

#[test]
fn an_odd_sized_chunk_is_followed_by_its_padding_byte() {
    let mut file = wav(1, 44_100, 16, false, &[0, 0, 0, 64]);

    // Un morceau de taille impaire : le suivant commence un octet plus loin.
    let mut avec = file[..36].to_vec();
    avec.extend_from_slice(b"note");
    avec.extend_from_slice(&3u32.to_le_bytes());
    avec.extend_from_slice(&[b'a', b'b', b'c', 0]);
    avec.extend_from_slice(&file[36..]);
    file = avec;

    let sound = decode(&file).expect("le remplissage d'alignement a decale la lecture");
    assert_eq!(sound.frames(), 2);
}

#[test]
fn something_that_is_not_a_wav_is_refused() {
    assert!(matches!(
        decode(b"pas du tout un wav"),
        Err(WavError::NotWav)
    ));
    assert!(matches!(decode(&[]), Err(WavError::NotWav)));

    let mut png = b"RIFF".to_vec();
    png.extend_from_slice(&100u32.to_le_bytes());
    png.extend_from_slice(b"AVI ");
    assert!(matches!(decode(&png), Err(WavError::NotWav)));
}

#[test]
fn a_wav_without_data_is_refused() {
    let complet = wav(1, 44_100, 16, false, &[0, 0]);
    // Coupe avant le morceau `data`.
    assert!(matches!(decode(&complet[..36]), Err(WavError::NoData)));
}

#[test]
fn a_chunk_claiming_more_than_the_file_holds_is_refused() {
    let mut file = wav(1, 44_100, 16, false, &[0, 0]);
    // Annonce mille octets de donnees, n'en fournit que deux.
    let taille = file.len() - 4;
    file[taille..].copy_from_slice(&1000u32.to_le_bytes());

    assert!(matches!(decode(&file), Err(WavError::Truncated)));
}

#[test]
fn an_unsupported_format_says_what_it_found() {
    // Tag 7 : mu-law.
    let mut file = wav(1, 44_100, 8, false, &[0, 0]);
    file[20..22].copy_from_slice(&7u16.to_le_bytes());

    match decode(&file) {
        Err(WavError::Unsupported(what)) => assert!(what.contains('7'), "{what}"),
        other => panic!("attendu Unsupported, obtenu {other:?}"),
    }
}

#[test]
fn a_wav_with_too_many_channels_is_refused() {
    let file = wav(6, 44_100, 16, false, &[0; 24]);
    assert!(matches!(decode(&file), Err(WavError::Unsupported(_))));
}

#[test]
fn an_extensible_wav_reads_its_real_format() {
    // 0xFFFE : le vrai format vit dans le sous-format, seize octets plus loin.
    let data: Vec<u8> = [0.5f32].iter().flat_map(|s| s.to_le_bytes()).collect();
    let mut fmt = Vec::new();
    fmt.extend_from_slice(&0xFFFEu16.to_le_bytes());
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&44_100u32.to_le_bytes());
    fmt.extend_from_slice(&176_400u32.to_le_bytes());
    fmt.extend_from_slice(&4u16.to_le_bytes());
    fmt.extend_from_slice(&32u16.to_le_bytes());
    fmt.extend_from_slice(&22u16.to_le_bytes());
    fmt.extend_from_slice(&32u16.to_le_bytes());
    fmt.extend_from_slice(&3u32.to_le_bytes());
    // Le sous-format : ses deux premiers octets portent le vrai tag.
    fmt.extend_from_slice(&3u16.to_le_bytes());
    fmt.extend_from_slice(&[0; 14]);

    let mut file = Vec::new();
    file.extend_from_slice(b"RIFF");
    file.extend_from_slice(&(20 + fmt.len() as u32 + data.len() as u32).to_le_bytes());
    file.extend_from_slice(b"WAVE");
    file.extend_from_slice(b"fmt ");
    file.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
    file.extend_from_slice(&fmt);
    file.extend_from_slice(b"data");
    file.extend_from_slice(&(data.len() as u32).to_le_bytes());
    file.extend_from_slice(&data);

    let sound = decode(&file).expect("un WAV extensible doit etre lu");
    assert!((sound.samples()[0] - 0.5).abs() < 1e-6);
}

#[test]
fn a_sounds_duration_follows_its_rate() {
    let sound = Sound::new(vec![0.0; 44_100], 1, 44_100);
    assert!((sound.duration().as_secs_f32() - 1.0).abs() < 1e-6);

    let moitie = Sound::new(vec![0.0; 22_050], 1, 44_100);
    assert!((moitie.duration().as_secs_f32() - 0.5).abs() < 1e-6);
}
