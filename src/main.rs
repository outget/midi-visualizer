use clap::Parser;
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use nannou::prelude::*;
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct VisualizerNote {
    pub pitch: u8,
    pub start_time: f32,
    pub duration: f32,
    pub track: usize,
}

struct Model {
    notes: Vec<VisualizerNote>,
}

pub fn extract_notes(midi_bytes: &[u8]) -> Vec<VisualizerNote> {
    let smf = Smf::parse(midi_bytes).expect("failed to parse midi");

    let ppq = match smf.header.timing {
        Timing::Metrical(ticks) => ticks.as_int() as f32,
        _ => panic!("god knows what you did"),
    };

    let mut all_notes = Vec::new();

    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut absolute_secs: f32 = 0.0;

        let mut current_tempo: f32 = 500_000.0;

        let mut active_notes: HashMap<u8, f32> = HashMap::new();

        for event in track {
            let delta_ticks = event.delta.as_int() as f32;
            let seconds_per_tick = (current_tempo / 1_000_000.0) / ppq;

            absolute_secs += delta_ticks * seconds_per_tick;

            match event.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                    current_tempo = t.as_int() as f32;
                }

                TrackEventKind::Midi { message, .. } => match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let pitch = key.as_int();
                        if vel.as_int() > 0 {
                            active_notes.insert(pitch, absolute_secs);
                        } else {
                            // velocity is 0 so noteoff
                            if let Some(start_time) = active_notes.remove(&pitch) {
                                all_notes.push(VisualizerNote {
                                    pitch,
                                    start_time,
                                    duration: absolute_secs - start_time,
                                    track: track_idx,
                                });
                            }
                        }
                    }

                    MidiMessage::NoteOff { key, .. } => {
                        let pitch = key.as_int();
                        if let Some(start_time) = active_notes.remove(&pitch) {
                            all_notes.push(VisualizerNote {
                                pitch,
                                start_time,
                                duration: absolute_secs - start_time,
                                track: track_idx,
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    all_notes
}

fn model(_app: &App) -> Model {
    let args = Args::parse();
    let bytes = fs::read(args.filename).unwrap();
    let notes = extract_notes(&bytes);

    Model { notes }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    let win = app.window_rect();
    let playhead_x = -win.w() / 3.0;
    let scroll_speed = 300.0;
    let t = app.time;

    draw.line()
        .start(pt2(playhead_x, win.bottom()))
        .end(pt2(playhead_x, win.top()))
        .color(SLATEGRAY)
        .weight(2.0);

    let palette = [
        rgb(0.9, 0.3, 0.4),
        rgb(0.3, 0.8, 0.9),
        rgb(0.5, 0.9, 0.4),
        rgb(0.9, 0.8, 0.2),
        rgb(0.7, 0.4, 0.9),
    ];

    for note in &model.notes {
        let y_pos = map_range(
            note.pitch as f32,
            30.0,
            90.0,
            win.bottom() + 50.0,
            win.top() - 50.0,
        );

        let raw_width = note.duration * scroll_speed;
        let note_width = raw_width.max(2.0);

        let dt = note.start_time - t;

        let x_pos = playhead_x + (dt * scroll_speed) + (note_width / 2.0);

        if x_pos + (note_width / 2.0) < win.left() || x_pos - (note_width / 2.0) > win.right() {
            continue;
        }

        let track_color = palette[note.track % palette.len()];

        draw.rect()
            .x_y(x_pos, y_pos)
            .w_h(note_width, 12.0)
            .color(track_color);
    }
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    filename: String,
}
