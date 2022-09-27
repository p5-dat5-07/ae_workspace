type FeatureVector = ();

fn similarity(_a: &FeatureVector, _b: &FeatureVector) -> f64 {
    0.0
}

/*
    TODO / things to consider
     - Consider using MAESTRO 3.0.0 dataset instead of 2.0.0
     - Read the file instead of including it as a binary blob
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!("../../data/maestro200/2004/MIDI-Unprocessed_SMF_02_R1_2004_01-05_ORIG_MID--AUDIO_02_R1_2004_05_Track05_wav.midi");
    let smf = midly::Smf::parse(data)?;

    println!("MIDI information:");
    println!("\t- {:?}", smf.header);
    println!("\t- Track count: {}", smf.tracks.len());

    let track_lengths = smf.tracks.iter().map(|track| track.len());
    let mut i = 0;
    for len in track_lengths {
        println!("\t- Track[{}] events: {}", i, len);
        i += 1;
    }

    println!("\n--Track 0 events:");

    for event in &smf.tracks[0] {
        println!("{:?}", event);
    }

    Ok(())
}
