/*
TODO: consider modelling time more accurately for the matrix
TODO: Setup model and get some samples for comparison
 */

use std::{iter::Peekable, slice::Iter, fs::File, io::Read};

use argh::FromArgs;
use plotters::prelude::*;

use midly::{MidiMessage, Track, TrackEvent, TrackEventKind};

mod feature_space;

type FeatureVector = u128;

fn similarity(a: &FeatureVector, b: &FeatureVector) -> f64 {
    // Equivalent to the innerproduct between two 128 length vector
    // containing only zeroes and ones.
    // (a & b).count_ones() as u8

    // Jaccard index based similarity calculation
    // this could be optimised with SIMD
    (a & b).count_ones() as f64 / (a | b).count_ones() as f64
}

fn apply_event_to_feature(f: &mut FeatureVector, e: &TrackEvent) {
    if let TrackEventKind::Midi { message, .. } = e.kind {
        match message {
            MidiMessage::NoteOff { key, .. } => *f = *f & !(1u128 << key.as_int()),
            MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => *f = *f & !(1u128 << key.as_int()),

            MidiMessage::NoteOn { key, .. } => *f = *f | 1u128 << key.as_int(),
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

/// Application for performing self-similarity and repetition analysis on
/// MIDI files
#[derive(FromArgs)]
struct Options {
    #[argh(positional)]
    input_file: String,

    /// an optional output destination.
    #[argh(option, short = 'o')]
    output_file: Option<String>,

    /// whether to output as an alpha matrix instead of a png
    #[argh(switch, short = 'a')]
    output_mode_alpha: bool, 

    /// dumps the events contained in the meta track
    #[argh(switch, short = 'm')]
    dump_meta_track: bool,

    /// scaling factor
    #[argh(option, short = 's', default = "1f64")]
    scale: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Options = argh::from_env();

    let data = File::open(&args.input_file)
        .and_then(|mut file| {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(data)
        })?;

    let smf = midly::Smf::parse(&data)?;

    println!("MIDI information:");
    println!("\t- {:?}", smf.header);
    println!("\t- Track count: {}", smf.tracks.len());

    let track_lengths = smf.tracks.iter().map(|track| track.len());
    let mut i = 0;
    for len in track_lengths {
        println!("\t- Track[{}] events: {}", i, len);
        i += 1;
    }

    if args.dump_meta_track {
        println!("\nMeta track dump:");
        for e in &smf.tracks[0] {
            println!("{:?}", e)
        }
    }

    println!("\nProcessing MIDI features");

    let features: Vec<FeatureVector> = FeatureStream::new(&smf.tracks[1]).collect();
    let count = features.len();

    let output_path = args.output_file.unwrap_or(format!("{}.png", args.input_file));

    println!("Plotting data {count}x{count}");
    let root =
        BitMapBackend::new(&output_path, (count as u32, count as u32)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
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
