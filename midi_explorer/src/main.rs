// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod iref;

use std::time::SystemTime;
use std::sync::Arc;

use druid::widget::{Align, BackgroundBrush, Button, Flex, Label};
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, FileDialogOptions,
    FileSpec, Handled, Target, Widget, WidgetExt, WindowDesc,
};

use midly::{SmfBytemap, TrackEventKind};

#[derive(Clone)]
struct MidiData {
    _bytes: Box<[u8]>,
    midi_repr: SmfBytemap<'static>,
}

impl MidiData {
    fn from_bytes(bytes: Vec<u8>) -> midly::Result<Self> {
        let bytes = bytes.into_boxed_slice();
        // SAFETY: this is only safe under the assumption that the `bytes` field
        //      never changes during the lifetime of the `MidiData` instance.
        //      I can't be bothered to come up with a proper solution as
        //      `midly` + `druid` makes it akward no matter what approach you take.
        let bytes_ref: &'static [u8] = unsafe { std::mem::transmute(bytes.as_ref()) };
        let midi = SmfBytemap::parse(bytes_ref)?;
        Ok(Self {
            _bytes: bytes,
            midi_repr: midi,
        })
    }
}

impl std::ops::Deref for MidiData {
    type Target = SmfBytemap<'static>;

    fn deref(&self) -> &Self::Target {
        &self.midi_repr
    }
}

/// Our data model for the druid view.
///
/// We use the `SystemTime` to track when the data
/// was last updated to minimise unecessary computations
/// when determining whether the app should redraw.
#[derive(Clone, Data)]
struct AppData(
    SystemTime,
    Arc<MidiData>,
    usize, // Selected track
    usize, // Selected event
);

struct Delegate;

pub fn main() {
    let main_window = WindowDesc::new(ui_builder()).title("MIDI Explorer");
    let data = None::<AppData>;
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<Option<AppData>> {
    let midi = FileSpec::new("MIDI file", &["midi"]);
    let any = FileSpec::new("Any", &["*"]);
    // The options can also be generated at runtime,
    // so to show that off we create a String for the default save name.
    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![midi, any])
        .default_type(midi)
        .name_label("Source")
        .title("Select MIDI file")
        .button_text("Open");

    let open = Button::new("Open File").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    let file_info_box = Label::new(|data: &Option<AppData>, _: &_| {
        if let Some(data) = data {
            format!("Header data:\n{:#?}", data.1.header)
        } else {
            String::from("No file selected")
        }
    });

    let mut left_col = Flex::column();
    left_col.add_child(open);
    left_col.add_child(Align::left(file_info_box));

    let left_col = left_col
        .background(BackgroundBrush::Color(Color::rgb8(33, 33, 33)))
        .fix_width(200f64)
        .expand_height();

    let track_list = Flex::column()
        .on_added(|flex, _, data: &Option<AppData>, _| {
            if let Some(data) = data {
                let mut i = 0;
                for track in &data.1.tracks {
                    let label = Label::new(format!("Track {} ({})", i, track.len())).on_click(
                        move |_, data: &mut Option<AppData>, _| {
                            if let Some(ref mut data) = data {
                                data.2 = i;
                            }
                        },
                    );

                    if i == data.2 {
                        flex.add_child(label.background(Color::rgb8(33, 33, 33)));
                    } else {
                        flex.add_child(label);
                    }

                    i += 1;
                }
            } else {
                flex.add_child(Label::new("No Tracks").center())
            }
        })
        .expand_height()
        .fix_width(200f64);

    let event_list = Flex::column()
        .on_added(|flex, _, data: &Option<AppData>, _| match data {
            Some(data) if data.2 < data.1.tracks.len() => {
                let mut i = 0;
                let track = &data.1.tracks[data.2];

                for (_, event) in track {
                    let kind_id = match event.kind {
                        TrackEventKind::Midi { .. } => "MIDI",
                        TrackEventKind::SysEx(_) => "Sys",
                        TrackEventKind::Escape(_) => "Escape",
                        TrackEventKind::Meta(_) => "Meta",
                    };

                    let label = Label::new(format!("[{}]: {}", i, kind_id)).on_click(
                        move |_, data: &mut Option<AppData>, _| {
                            if let Some(ref mut data) = data {
                                data.3 = i;
                            }
                        },
                    );

                    if i == data.3 {
                        flex.add_child(label.background(Color::rgb8(22, 22, 22)));
                    } else {
                        flex.add_child(label);
                    }

                    i += 1;
                }
            }
            _ => flex.add_child(Label::new("No track selected")),
        })
        .expand_height()
        .fix_width(200f64)
        .background(Color::rgb8(33, 33, 33));

    let event = Label::new(|data: &Option<AppData>, _: &_| match data {
        Some(data)
            if data
                .1
                .tracks
                .get(data.2)
                .and_then(|track| track.get(data.3))
                .is_some() =>
        {
            format!("{:#?}", data.1.tracks[data.2][data.3])
        }
        _ => String::from("No event selected."),
    })
    .expand_height()
    .fix_width(200f64);

    let mut row = Flex::row();
    row.add_child(left_col);
    row.add_child(track_list);
    row.add_child(event_list);
    row.add_child(event);
    Align::left(row)
}

impl AppDelegate<Option<AppData>> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut Option<AppData>,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match std::fs::read(file_info.path()) {
                Ok(s) => {
                    let midi_result = MidiData::from_bytes(s);
                    if let Ok(midi_data) = midi_result {
                        *data = Some(AppData(SystemTime::now(), Arc::new(midi_data), 0, 0));
                    } else {
                        eprintln!("Error parsing MIDI data!!");
                    }
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
            return Handled::Yes;
        }
        Handled::No
    }
}
