//! Utility functions for processing MIDI files.

use midly::{TrackEventKind, MidiMessage};

pub fn trim_midi(mut midi: midly::Smf) -> midly::Smf {
    let mut delta = 0;
    let track = midi.tracks.remove(1)
        .into_iter()
        .filter_map(|mut event| {
            delta += event.delta.as_int();
            match event.kind {
                TrackEventKind::Midi { message: MidiMessage::NoteOn { .. } | MidiMessage::NoteOff { .. }, ..} => {
                    event.delta = delta.into();
                    delta = 0;
                    Some(event)
                },
                _ => None,
            }
        }).collect();
    midi.tracks.push(track);
    midi
}
