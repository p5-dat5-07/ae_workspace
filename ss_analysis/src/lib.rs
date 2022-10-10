use std::{fs::File, io::Read, ops::Deref, marker::PhantomData};

use argh::FromArgs;
use midly::Track;
use plotters::{prelude::*, element::PointCollection};

pub use midly;

mod feature_space;
mod analysis;

pub fn reduce_track_domain(track: &mut Track) {
    let deltas: Vec<u32> = track.iter()
        .map(|event| event.delta.as_int())
        .collect();

    if let Some(divisor) = gcd(&deltas) {
        println!("domain gcd = {divisor}");
        for event in track {
            event.delta = midly::num::u28::from(event.delta.as_int() / divisor);
        }
    }
}

#[cfg(test)]
#[test]
fn test_gcd() {
    let data = File::open("../data/maestro300/2004/MIDI-Unprocessed_SMF_02_R1_2004_01-05_ORIG_MID--AUDIO_02_R1_2004_05_Track05_wav.midi").and_then(|mut file| {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }).unwrap();

    let mut smf = midly::Smf::parse(&data).unwrap();
    reduce_track_domain(&mut smf.tracks[1]);

}

/// An object that allows for an image type `I` to contain
/// to heap allocated data of type `T` owned by the Lens.
pub struct Lens<'l, T, I>
where T: 'l, I: 'l
{
    _data: T,
    image: I,
    _p: PhantomData<&'l ()>,
}

impl<T, I> Lens<'static, Box<T>, I>
where T: 'static
{

    pub fn new(data: T, lens_fn: impl Fn(&'static T) -> I) -> Self {
        let boxed = Box::new(data);
        // safety: we guarantee at the API level that data is not mutated or moved
        //         and that the data and image lives equally as long.
        let image = lens_fn(unsafe { std::mem::transmute(boxed.as_ref()) });
        Self {
            _data: boxed,
            image,
            _p: PhantomData,
        }
    }

}

// Allows us to use the lens as if it was the image object
impl<'l, T, I> Deref for Lens<'l , T, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

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

    /// whether to output as an alpha matrix instead of a png (currently unimplemented)
    #[argh(switch, short = 'a')]
    output_mode_alpha: bool,

    /// dumps the events contained in the meta track
    #[argh(switch, short = 'm')]
    dump_meta_track: bool,

    /// scaling factor
    #[argh(option, short = 's', default = "1f64")]
    scale: f64,

    /// limit the duration that is being analyzed
    #[argh(option, short = 'l')]
    limit: Option<u32>,
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
    let size = args.limit.unwrap_or(timed_space.size());

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

    if let Some(limit) = args.limit {
        chart.draw_series(timed_space.draw().filter(|rect| {
            let a = (&rect).point_iter();
            a[0].0 < limit && a[0].1 < limit
        }))?;
    } else {
        chart.draw_series(timed_space.draw())?;
    }

    Ok(())
}
