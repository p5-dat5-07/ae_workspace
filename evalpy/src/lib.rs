use std::{
    fs::File,
    io::{stdout, Read, Write},
    sync::Arc, collections::HashMap,
};

use pyo3::{prelude::*, exceptions::{PyIOError, PyBaseException}};
use ss_analysis::{midly::{self, TrackEvent, Track}, plotters::prelude::*, Lens, TemporalSpace};

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
            ErrorKind::Midly
            | ErrorKind::Generic => PyBaseException::new_err(msg),
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
    Ok(())
}

// The MIDI lens is wrapped in a Arc object to ensure that it lives for as long as it is referenced
// allowing for track lenses to borrow the midi data.
type MidiLens = Arc<Lens<Box<Vec<u8>>, midly::Smf<'static>>>;
type TrackLens = Lens<MidiLens, &'static Track<'static>>;

#[pyclass]
struct MidiObject {
    data: MidiLens,
}

#[pymethods]
impl MidiObject {
    #[new]
    pub fn new(path: String) -> PyResult<Self> {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(
            Lens::try_new::<Error>(data, |data| Ok(midly::Smf::parse(&data)?))
                .map(|lens| Self { data: MidiLens::new(lens) })?,
        )
    }
    
    pub fn headers(&self) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();
        
        map.insert("format", format!("{:?}", self.data.header.format));
        map.insert("timing", format!("{:?}", self.data.header.timing));
        
        map
    }
    
    pub fn track_count(&self) -> usize {
        self.data.tracks.len()
    }
}

#[pyclass]
// TODO implement sequencing
struct TrackObject {
    data: TrackLens,
}

#[pymethods]
impl TrackObject {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<TrackIterator>> {
        let iter = TrackIterator {
            iter: slf.data.clone_map(|track| track.iter()),
        };
        Py::new(slf.py(), iter)
    }
    
    fn feature_space(&self) -> FeatureSpaceObject {
        FeatureSpaceObject { inner: TemporalSpace::new(*self.data) }
    }
}

#[pyclass]
struct TrackIterator {
    iter: Lens<MidiLens, std::slice::Iter<'static, TrackEvent<'static>>>,
}

#[pymethods]
impl TrackIterator {
    fn __item__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u32> {
        let a = slf.iter.next();
        a.map(|e| {e.delta.as_int()})
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
        root.fill(&WHITE).map_err(|_| PyBaseException::new_err("Unexpected error drawing SSM"))?;
        
        let size = self.inner.size();
        let mut chart = ChartBuilder::on(&root).build_cartesian_2d(0..size, size..0)
            .map_err(|_| PyBaseException::new_err("Unexpected error drawing SSM"))?;
        
        chart.draw_series(self.inner.draw()).unwrap();
        Ok(())
    }
    
}
