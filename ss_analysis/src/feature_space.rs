use std::iter;

use midly::{MetaMessage, MidiMessage, Track, TrackEventKind};
use plotters::{
    prelude::{Rectangle, BLACK},
    style::Color,
};

use crate::scale_tuple;

#[derive(Clone, Copy, Default, PartialEq)]
struct FeatureVector(u128);

impl FeatureVector {
    fn similarity_of(&self, other: &Self) -> f64 {
        (self.0 & other.0).count_ones() as f64 / (self.0 | other.0).count_ones() as f64
    }

    fn apply(self, message: &MidiMessage) -> Self {
        Self(match message {
            MidiMessage::NoteOff { key, .. } => self.0 & !(1u128 << key.as_int()),
            MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => {
                self.0 & !(1u128 << key.as_int())
            }

            MidiMessage::NoteOn { key, .. } => self.0 | 1u128 << key.as_int(),
            _ => self.0,
        })
    }
}

pub struct TimedSpace {
    duration: u32,
    features: Vec<(FeatureVector, u32, u32)>, // (_, duration, offset)
}

impl TimedSpace {
    pub fn new(track: &Track) -> Self {
        let mut ts = Self {
            duration: 0,
            features: Vec::new(),
        };
        let mut vector = FeatureVector::default();
        let mut duration = 0;
        for event in track {
            duration += event.delta.as_int();
            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    let v = vector.apply(&message);

                    if v != vector {
                        ts.features
                            .push((vector, duration - ts.duration, ts.duration));
                        ts.duration = duration;
                        vector = v;
                    }
                }
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

    pub fn draw<'l>(&'l self, scale: f64) -> impl Iterator<Item = Rectangle<(u32, u32)>> + 'l {
        let mut skip_offset = 0;
        let iter = self
            .features
            .iter()
            .filter(|v| v.0 == FeatureVector::default());

        iter.clone()
            .map(move |v| {
                let r = iter::repeat(v).zip(iter.clone().skip(skip_offset));
                skip_offset += 1;
                r
            })
            .flatten()
            .map(move |(a, b)| {
                let (x, y) = scale_tuple((a.2, b.2), scale);
                let (x2, y2) = scale_tuple((a.2 + a.1, b.2 + b.1), scale);
                let color = BLACK.mix(a.0.similarity_of(&b.0));

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
        self.duration
    }
}
