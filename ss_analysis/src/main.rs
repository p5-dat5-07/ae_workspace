use std::{fs::File, io::Read};

use argh::FromArgs;
use plotters::prelude::*;

mod feature_space;

/*
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
}*/

/// Application for performing self-similarity and repetition analysis on
/// MIDI files.
/// 
/// For any MIDI file that is longer than a few seconds, it is recomended
/// to set the scale parameter to a value < 1.0. You will otherwise be likely
/// to run out of memory running the program.
#[derive(FromArgs)]
struct Options {
    #[argh(positional)]
    input_file: String,

    /// an optional output destination.
    #[argh(option, short = 'o')]
    output_file: Option<String>,

    /// whether to output as an alpha matrix instead of a png (currently unavailable)
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

    let data = File::open(&args.input_file).and_then(|mut file| {
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

    let timed_space = feature_space::TimedSpace::new(&smf.tracks[1]);
    let count = timed_space.feature_count();
    let size = timed_space.size();

    let output_path = args
        .output_file
        .unwrap_or(format!("{}.png", args.input_file));

    println!(
        "Plotting data <features = {count} ; matrix size = {size} ; scale = {}>",
        args.scale
    );

    let image_size = (size as f64 * args.scale) as u32;
    let root = BitMapBackend::new(&output_path, (image_size, image_size)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root).build_cartesian_2d(0..size, size..0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    chart.draw_series(timed_space.draw())?;

    /*
    for y in 0..count {
        chart.draw_series((0..count).map(|x| {
            let a = similarity(&features[y], &features[x]);
            Rectangle::new(
                [(x as i32, y as i32), (x as i32 + 1, y as i32 + 1)],
                RGBAColor(0, 0, 0, a).filled(),
            )
        }))?;
    }*/

    Ok(())
}
