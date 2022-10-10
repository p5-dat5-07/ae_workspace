use std::{io::{Read, Write, stdout}, fs::File, sync::Mutex};

use cpython::{PyResult, Python, py_module_initializer, py_fn};
use ss_analysis::{midly, Lens};

// Register python module/functions
py_module_initializer!(evalpy, |py, m| {
    m.add(py, "__doc__", "Python bindings for evaluation functionality implented in rust.")?;
    m.add(py, "load_midi", py_fn!(py, load_midi(path: String)))?;
    m.add(py, "unload_midi", py_fn!(py, unload_object(i: usize)))?;
    m.add(py, "print_midi_track", py_fn!(py, print_midi_track(i: usize, track: usize)))?;
    Ok(())
});

type MidiLens = Lens<'static, Box<Vec<u8>>, midly::Smf<'static>>;

enum Object {
    Midi(MidiLens),
    None(Option<usize>),
}

struct ModuleContext {
    objects: Vec<Object>,
    next_loc: Option<usize>,
}

impl ModuleContext {
    const fn new() -> Self {
        ModuleContext {
            objects: Vec::new(),
            next_loc: None,
        }
    }

    fn insert_object(&mut self, obj: Object) -> usize {
        if let Some(i) = self.next_loc {
            self.next_loc = match self.objects[i] {
                Object::None(next) => next,
                _ => unreachable!("Object referred to by`ModuleContext::next_loc` must always be a Object::None."),
            };

            self.objects[i] = obj;
            i
        } else {
            let i = self.objects.len();
            self.objects.push(obj);
            i
        }
    }

    fn remove_object(&mut self, i: usize) {
        if let Some(obj) = self.objects.get_mut(i) {
            *obj = Object::None(self.next_loc);
            self.next_loc = Some(i);
        }
    }
}

static MODULE_CTX: Mutex<ModuleContext> = Mutex::new(ModuleContext::new());

fn load_midi(_: Python, path: String) -> PyResult<i32> {
    let data = File::open(&path)
        .and_then(|mut file| {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(data)
        }).unwrap();
    
    let lens = Lens::new(data, |data| {
        let mut smf = midly::Smf::parse(&data).unwrap();
        ss_analysis::reduce_track_domain(&mut smf.tracks[1]);
        smf
    });
    let mut ctx = MODULE_CTX.lock().unwrap();

    Ok(ctx.insert_object(Object::Midi(lens)) as i32)
}

fn unload_object(_: Python, i: usize) -> PyResult<i32> {
    let mut ctx = MODULE_CTX.lock().unwrap();
    ctx.remove_object(i);
    Ok(1)
}

fn print_midi_track(_: Python, i: usize, track: usize) -> PyResult<i32> {
    let ctx = MODULE_CTX.lock().unwrap();
    Ok(ctx.objects.get(i)
            .and_then(|obj| match obj {
                Object::Midi(smf) => Some(smf),
                _ => None,
            })
            .and_then(|smf| smf.tracks.get(track))
            .map(|track| {
                let mut stdout = stdout().lock();
    
                write!(stdout, "Printing MIDI track ({} events)\n", track.len()).unwrap();
                for ele in track {
                    write!(stdout, "{:#?}\n", ele).unwrap();
                }
                1
            })
            .unwrap_or(0)
        )
}
