use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, Read, Write},
    sync::Arc,
};

use pyo3::{
    exceptions::{PyBaseException, PyIOError},
    prelude::*,
};
use ss_analysis::{
    midly::{self, MidiMessage, Track, TrackEvent, TrackEventKind, MetaMessage},
    plotters::prelude::*,
    Lens, TemporalSpace,
};

// Error handling/conversion

#[derive(Debug)]
enum ErrorKind {
    IO,
    Midly,
    Generic,
}

#[derive(Debug)]
struct Error {
    inner: Box<dyn std::error::Error>,
    kind: ErrorKind,
}

impl From<midly::Error> for Error {
    fn from(err: midly::Error) -> Self {
        Self {
            inner: err.into(),
            kind: ErrorKind::Midly,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            inner: err.into(),
            kind: ErrorKind::IO,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[evalpy err: {:?}]: {}",
            self.kind,
            self.inner.to_string()
        )
    }
}

impl std::error::Error for Error {}

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        let msg = err.to_string();
        match err.kind {
            ErrorKind::IO => PyIOError::new_err(msg),
            ErrorKind::Midly | ErrorKind::Generic => PyBaseException::new_err(msg),
        }
    }
}

/// Entrypoint function for the python module.
/// This function is responsible for registering
/// all necesarry python functions/types.
#[pymodule]
fn evalpy(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<MidiObject>()?;
    m.add_class::<TrackObject>()?;
    m.add_class::<TrackIterator>()?;
    m.add_class::<MidiEventKind>()?;
    m.add_class::<MidiEventObject>()?;
    m.add_class::<FeatureSpaceObject>()?;
    Ok(())
}

// The MIDI lens is wrapped in a Arc object to ensure that it lives for as long as it is referenced
// allowing for track lenses to borrow the midi data.
type MidiLens = Arc<Lens<Box<Vec<u8>>, midly::Smf<'static>>>;
type TrackLens = Lens<MidiLens, &'static Track<'static>>;

#[pyclass(sequence)]
struct MidiObject {
    inner: MidiLens,
}

#[pymethods]
impl MidiObject {
    #[new]
    pub fn new(path: String) -> PyResult<Self> {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(
            Lens::try_new::<Error>(data, |data| Ok(midly::Smf::parse(&data)?)).map(|lens| {
                Self {
                    inner: MidiLens::new(lens),
                }
            })?,
        )
    }

    pub fn headers(&self) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();

        map.insert("format", format!("{:?}", self.inner.header.format));
        map.insert("timing", format!("{:?}", self.inner.header.timing));

        map
    }

    fn __len__(&self) -> usize {
        self.inner.tracks.len()
    }

    fn __getitem__(&self, i: isize) -> TrackObject {
        let i = if i < 0 {
            self.inner.tracks.len() as isize + i
        } else {
            i
        };

        TrackObject {
            inner: TrackLens::new_rc(self.inner.clone(), |data| &data.tracks[i as usize]),
        }
    }
}

#[pyclass(sequence)]
struct TrackObject {
    inner: TrackLens,
}

#[pymethods]
impl TrackObject {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<TrackIterator>> {
        let iter = TrackIterator {
            iter: slf.inner.clone_map(|track| track.iter()),
        };
        Py::new(slf.py(), iter)
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __getitem__(&self, i: isize) -> MidiEventObject {
        let i = if i < 0 {
            self.inner.len() as isize + i
        } else {
            i
        };

        let event = &self.inner[i as usize];
        MidiEventObject {
            delta: event.delta.as_int(),
            inner: match event.kind {
                TrackEventKind::Midi { message, .. } => match message {
                    MidiMessage::NoteOn { key, vel } => MidiEventData::NoteData {
                        velocity: vel.as_int(),
                        key: key.as_int(),
                    },
                    MidiMessage::NoteOff { key, .. } => MidiEventData::NoteData {
                        velocity: 0,
                        key: key.as_int(),
                    },
                    _ => MidiEventData::Other,
                },
                TrackEventKind::Meta(MetaMessage::TimeSignature(a, b, c, d)) => MidiEventData::TimeSignature([a,b,c,d]),
                TrackEventKind::Meta(MetaMessage::EndOfTrack) => MidiEventData::EndOfTrack,
                _ => MidiEventData::Other,
            },
        }
    }

    fn dbg_print(&self) {
        println!("{:#?}", *(self.inner))
    }
    
    fn feature_space(&self) -> FeatureSpaceObject {
        FeatureSpaceObject {
            inner: TemporalSpace::new(*self.inner),
        }
    }
}

#[pyclass]
struct TrackIterator {
    iter: Lens<MidiLens, std::slice::Iter<'static, TrackEvent<'static>>>,
}

#[pymethods]
impl TrackIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u32> {
        let a = slf.iter.next();
        a.map(|e| e.delta.as_int())
    }
}

#[pyclass]
#[derive(Clone, Copy)]
enum MidiEventKind {
    Note,
    Tempo,
    TimeSignature,
    EndOfTrack,
    Other,
}

enum MidiEventData {
    NoteData { velocity: u8, key: u8 },
    TimeSignature([u8; 4]),
    EndOfTrack,
    Other,
}

#[pyclass]
struct MidiEventObject {
    #[pyo3(get)]
    delta: u32,
    inner: MidiEventData,
}

#[pymethods]
impl MidiEventObject {
    #[getter]
    fn velocity(&self) -> Option<u8> {
        match self.inner {
            MidiEventData::NoteData { velocity, .. } => Some(velocity),
            _ => None,
        }
    }

    #[getter]
    fn key(&self) -> Option<u8> {
        match self.inner {
            MidiEventData::NoteData { key, .. } => Some(key),
            _ => None,
        }
    }

    #[getter]
    fn time_signature(&self) -> Option<[u8; 4]> {
        match self.inner {
            MidiEventData::TimeSignature(signature) => Some(signature),
            _ => None,
        }
    }
}

#[pyclass]
struct FeatureSpaceObject {
    inner: TemporalSpace,
}

#[pymethods]
impl FeatureSpaceObject {
    fn draw_ssm(&self, path: &str, scale: f32) -> PyResult<()> {
        let size = (self.inner.size() as f32 * scale) as u32;
        let root = BitMapBackend::new(path, (size, size)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|_| PyBaseException::new_err("Unexpected error drawing SSM"))?;

        let size = self.inner.size();
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0..size, size..0)
            .map_err(|_| PyBaseException::new_err("Unexpected error drawing SSM"))?;

        chart.draw_series(self.inner.draw()).unwrap();
        Ok(())
    }
}
