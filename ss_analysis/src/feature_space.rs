use std::iter;
use std::mem::take;

use midly::{MetaMessage, MidiMessage, Track, TrackEventKind};
use plotters::{
    prelude::{Rectangle, BLACK},
    style::Color,
};

/// A feature vector represented as a 128-bit unsigned integer.
/// Each entry in the vector is the binary (on/off) state of a
/// corresponding MIDI note.
/// The vector thusly describes which MIDI notes are being played at a given time.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct FeatureVector(u128);

impl FeatureVector {
    /// Computes the similarity between two `FeatureVector`s.
    /// Current implementation is based on the Jaccard index:
    ///     J(A,B) = |A intersect B| / |A union B|
    /// Where A and B are sets of notes that are active in
    /// the respective features.
    ///
    /// The implementation exploits the fact that we represent
    /// the notes as binary features (as being either on or off)
    /// to translate set operations (intersect, union) into corresponding
    /// bitwise logical operations (and, or) yielding significant memory
    /// and performance gains.
    fn similarity_of(&self, other: &Self) -> f64 {
        (self.0 & other.0).count_ones() as f64 / (self.0 | other.0).count_ones() as f64
    }

    /// Applies a MIDI event to a copy of the given feature vector
    /// updating the relevent component to match to change described in the event.
    fn apply(self, message: &MidiMessage) -> Self {
        Self(match message {
            // NoteOff event -> Toggles the corresponding bit off
            MidiMessage::NoteOff { key, .. } => self.0 & !(1u128 << key.as_int()),
            // A NoteOn event with a velocity of zero is the same as a NoteOff event.
            MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => {
                self.0 & !(1u128 << key.as_int())
            }

            // NoteOn event -> Toggle the corresponding bit on
            MidiMessage::NoteOn { key, .. } => self.0 | 1u128 << key.as_int(),
            _ => self.0,
        })
    }
}

type TemporalUnitType = f32;
type TemporalFeature = (FeatureVector, TemporalUnitType, TemporalUnitType);

pub struct TemporalSpace {
    duration: TemporalUnitType,
    features: Vec<TemporalFeature>, // (_, duration, offset)
}

impl TemporalSpace {
    pub fn new(track: &Track) -> Self {
        let mut ts = Self {
            duration: 0.0,
            features: Vec::new(),
        };
        let mut vector = FeatureVector::default();
        let mut duration = 0.0;
        for event in track {
            duration += event.delta.as_int() as f32;
            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    let v = vector.apply(&message);

                    // Once the feature vector has changed we add it to the list
                    if v != vector {
                        ts.features
                            .push((vector, duration - ts.duration, ts.duration));
                        ts.duration = duration;
                        vector = v;
                    }
                }
                // Adds the remaining feature once the end of the track is reached.
                TrackEventKind::Meta(MetaMessage::EndOfTrack) => {
                    ts.features
                        .push((vector, duration - ts.duration, ts.duration));
                    ts.duration = duration;
                }
                _ => (),
            }
        }

        ts
    }

    /// Scales the domain (time axis) of the feature space.
    /// Generally used for the purpose of reduce the size
    /// resulting similarity matrix, reducing memory requirements.
    fn scale_domain(&mut self, scale: TemporalUnitType) {
        self.duration *= scale;
        let features = take(&mut self.features);
        self.features = features
            .into_iter()
            .map(|(vector, duration, offset)| (vector, duration * scale, offset * scale))
            .collect();
    }

    /// Creates an iterator over all unique feature combinations.
    /// (Meaning: if pair (x,y) is encountered, then (y,x) will not be)
    pub fn iter_pairs<'l>(
        &'l self,
    ) -> impl Iterator<Item = (&'l TemporalFeature, &'l TemporalFeature)> {
        let mut skip_offset = 0;
        let iter = self.features.iter();

        iter.clone()
            .map(move |v| {
                let r = iter::repeat(v).zip(iter.clone().skip(skip_offset));
                skip_offset += 1;
                r
            })
            .flatten()
    }

    /// Creates an iterator yielding the Rectangle objects that describes
    /// the self similarity matrix of the TimedSpace.
    pub fn draw<'l>(&'l self) -> impl Iterator<Item = Rectangle<(u32, u32)>> + 'l {
        self.iter_pairs()
            .map(move |(a, b)| {
                let (x, y) = (a.2 as u32, b.2 as u32);
                let (x2, y2) = ((a.2 + a.1) as u32, (b.2 + b.1) as u32);
                let color = BLACK.mix(a.0.similarity_of(&b.0)).filled();

                [
                    Rectangle::new([(x, y), (x2, y2)], color),
                    Rectangle::new([(y, x), (y2, x2)], color),
                ]
            })
            .flatten()
    }

    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    pub fn size(&self) -> u32 {
        self.duration as u32
    }
}
