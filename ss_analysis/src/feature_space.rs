use std::slice::Iter;

use midly::{Track, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use plotters::prelude::Rectangle;

#[derive(Clone, Copy, Default, PartialEq)]
struct FeatureVector(u128);

impl FeatureVector {
    fn apply(self, message: &MidiMessage) -> Self {
        Self(match message {
            MidiMessage::NoteOff { key, .. } => self.0 & !(1u128 << key.as_int()),
            MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => self.0 & !(1u128 << key.as_int()),

            MidiMessage::NoteOn { key, .. } => self.0 | 1u128 << key.as_int(),
            _ => self.0,
        })
    }
}


struct TimedSpace {
    duration: u32,
    features: Vec<(FeatureVector, u32)>,
}

impl TimedSpace {
    fn new(track: &Track) -> Self {
        let mut ts = Self {
            duration: 0,
            features: Vec::new(),
        };
        let mut vector = FeatureVector::default();
        let mut duration = 0u32;

        for event in track {
            match event.kind {
                TrackEventKind::Midi { message , .. } => {
                    let v = vector.apply(&message);
                    
                    // the feature remains unchaged
                    if v == vector {
                        duration += event.delta.as_int();
                    } else {
                        ts.features.push((vector, duration));
                        ts.duration += duration;
                        vector = v;
                        duration = 0;
                    }
                },
                TrackEventKind::Meta(MetaMessage::EndOfTrack) => {
                    ts.features.push((vector, duration));
                    ts.duration += duration;
                }
                _ => {
                    duration += event.delta.as_int();
                }
            }
        }

        ts
    }
}

