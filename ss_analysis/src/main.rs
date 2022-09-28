use std::{iter::Peekable, slice::Iter};

use plotters::prelude::*;

use midly::{MidiMessage, Track, TrackEvent, TrackEventKind};

type FeatureVector = [u8; 128];

fn similarity(a: &FeatureVector, b: &FeatureVector) -> u32 {
    (&a[..])
        .iter()
        .zip(&b[..])
        .map(|(a, b)| (a * b) as u32)
        .reduce(|sum, x| sum + x)
        .unwrap()
}

/*
    TODO / things to consider
     - Read the file instead of including it as a binary blob
*/

fn apply_event_to_feature(f: &mut FeatureVector, e: &TrackEvent) {
    if let TrackEventKind::Midi { message, .. } = e.kind {
        match message {
            MidiMessage::NoteOn { key, .. } => f[key.as_int() as usize] = 1,
            MidiMessage::NoteOff { key, .. } => f[key.as_int() as usize] = 0,
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
            feature: [0; 128],
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

    println!("\n--Track 1 events:");

    let features: Vec<FeatureVector> = FeatureStream::new(&smf.tracks[1]).collect();
    let count = features.len();

    let mut ss_matrix = Vec::<u32>::with_capacity(features.len() * features.len());

    for y in 0..count {
        for x in 0..count {
            ss_matrix.push(similarity(&features[y], &features[x]));
        }
    }

    let root = BitMapBackend::new("ssm.png", (1028, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Self similarity", 80)
        .margin(5)
        .top_x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0i32..15i32, 15i32..0i32)?;

    chart
        .configure_mesh()
        .x_labels(count)
        .y_labels(count)
        .max_light_lines(4)
        .x_label_offset(35)
        .y_label_offset(25)
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", 20))
        .draw()?;

    chart.draw_series(ss_matrix.iter().enumerate().map(|(i, v)| {
        let x = (i % count) as i32;
        let y = (i / count) as i32;
        Rectangle::new(
            [(x, y), (x + 1, y + 1)],
            RGBAColor(0, 0, 0, *v as f64 / 128f64).filled(),
        )
    }))?;

    Ok(())
}
