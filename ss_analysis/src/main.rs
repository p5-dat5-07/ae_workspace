use std::{fs::File, io::Write, iter::Peekable, slice::Iter};

use plotters::prelude::*;

use midly::{MidiMessage, Track, TrackEvent, TrackEventKind};

type FeatureVector = u128;

fn similarity(a: &FeatureVector, b: &FeatureVector) -> f64 {
    // Equivalent to the innerproduct between two 128 length vector
    // containing only zeroes and ones.
    // (a & b).count_ones() as u8

    // Jaccard index based similarity calculation
    (a & b).count_ones() as f64 / (a | b).count_ones() as f64
}

fn apply_event_to_feature(f: &mut FeatureVector, e: &TrackEvent) {
    if let TrackEventKind::Midi { message, .. } = e.kind {
        match message {
            MidiMessage::NoteOn { key, .. } => *f = *f | 1u128 << key.as_int(),
            MidiMessage::NoteOff { key, .. } => *f = *f & !(1u128 << key.as_int()),
            _ => (),
        }
    }
}

struct FeatureStream<'l> {
    iter: Peekable<Iter<'l, TrackEvent<'l>>>,
    feature: FeatureVector,
}

impl<'l> FeatureStream<'l> {
    fn new(src: &'l Track<'l>) -> Self {
        Self {
            iter: src.iter().peekable(),
            feature: 0,
        }
    }
}

impl<'l> Iterator for FeatureStream<'l> {
    type Item = FeatureVector;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self
            .iter
            .next()
            .map(|e| apply_event_to_feature(&mut self.feature, e));

        while let Some(TrackEvent { delta, .. }) = self.iter.peek() {
            if delta.as_int() != 0 {
                break;
            }

            self.iter
                .next()
                .map(|e| apply_event_to_feature(&mut self.feature, e));
        }

        r.map(|_| self.feature.clone())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!("../../data/maestro300/2004/MIDI-Unprocessed_SMF_02_R1_2004_01-05_ORIG_MID--AUDIO_02_R1_2004_05_Track05_wav.midi");
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

    println!("\nProcessing MIDI features");

    let features: Vec<FeatureVector> = FeatureStream::new(&smf.tracks[1]).collect();
    let count = 500usize; //features.len() / 10;

    println!("Calculating SSM ({count}x{count})");

    let mut f = File::create("out.dat")?;

    for y in 0..count {
        let offset = count * y;
        for x in 0..count {
            write!(f, "\t {}", similarity(&features[y], &features[x]))?;
        }
        write!(f, "\n")?;
    }

    println!("Plotting data");
    let root =
        BitMapBackend::new("ssm.png", (40 + count as u32, 100 + count as u32)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Self similarity", 80)
        .margin(5)
        .top_x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0i32..count as i32, (count as i32)..0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", 20))
        .draw()?;

    for y in 0..count {
        chart.draw_series((0..count).map(|x| {
            let a = similarity(&features[y], &features[x]);
            Rectangle::new(
                [(x as i32, y as i32), (x as i32 + 1, y as i32 + 1)],
                RGBAColor(0, 0, 0, a).filled(),
            )
        }))?;
    }

    Ok(())
}
