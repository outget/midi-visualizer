use midly::{MidiMessage, Smf, TrackEventKind};
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct VisualizerNote {
    pub pitch: u8,
    pub start_tick: u64,
    pub duration_tick: u64,
}

pub fn extract_notes(midi_bytes: &[u8]) -> Vec<VisualizerNote> {
    let smf = Smf::parse(midi_bytes).expect("failed to parse midi");

    let mut all_notes = Vec::new();

    for track in smf.tracks {
        let mut absolute_tick: u64 = 0;

        let mut active_notes: HashMap<u8, u64> = HashMap::new();

        for event in track {
            absolute_tick += event.delta.as_int() as u64;

            if let TrackEventKind::Midi { message, .. } = event.kind {
                match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let pitch = key.as_int();
                        let velocity = vel.as_int();

                        if velocity > 0 {
                            active_notes.insert(pitch, absolute_tick);
                        } else {
                            if let Some(start_tick) = active_notes.remove(&pitch) {
                                all_notes.push(VisualizerNote {
                                    pitch,
                                    start_tick,
                                    duration_tick: absolute_tick - start_tick,
                                })
                            }
                        }
                    }

                    MidiMessage::NoteOff { key, .. } => {
                        let pitch = key.as_int();

                        if let Some(start_tick) = active_notes.remove(&pitch) {
                            all_notes.push(VisualizerNote {
                                pitch,
                                start_tick,
                                duration_tick: absolute_tick - start_tick,
                            })
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    all_notes
}

fn main() {
    let bytes = fs::read("darude-sandstorm.mid").unwrap();
    let notes = extract_notes(&bytes);

    println!("{:#?}", notes);
}
