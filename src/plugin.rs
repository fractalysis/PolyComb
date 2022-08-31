#![feature(generic_associated_types)]

mod midivoice;
//mod buftools;

use std::vec;

use baseplug::{Plugin, ProcessContext, MidiReceiver, Smooth};
use serde::{Deserialize, Serialize};

use midivoice::MidiVoice;
//use buftools::buftools as bt;

const MAX_DELAY: f32 = 500.0f32; // in ms
const MAX_POLYPHONY: usize = 16;
const PITCH_BEND_SMOOTHING: f32 = 50.0f32; // in ms
const MAX_BLOCKSIZE: usize = 128; //make soto fix this lol

// todo:
// - release envelopes ✔️
// - pitch bend ✔️
// - damper pedal ✔️
// - monophony / portamento ✔️
// - fix the bug where if you press a note a bunch of times before the release ends it stays on ✔️
// - fix the bug where if you're in mono mode and you press a note before its release ends, the envelope pops
// - do something with portamento if you're in poly mode
// - make velocity do something, per-note feedback?
// - pitch bend jumps fuck it up lol idk
// - optimize the powf maybe
// - still zipper noise when you play a high note and move pitch bend around; resample more ig


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct PhaseyModel {
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "Dry", unit = "Generic")]
        dry: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "Wet", unit = "Generic")]
        wet: f32,

        #[model(min = 1.0, max = 500.0)]
        #[parameter(name = "Attack", unit = "Generic")]
        attack: f32,

        #[model(min = 5.0, max = 500.0)]
        #[parameter(name = "Release", unit = "Generic")]
        release: f32,

        #[model(min = 0.0, max = 24.0)]
        #[parameter(name = "Pitch Bend", unit = "Generic")]
        pitch_bend_range: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "Feedback", unit = "Generic")]
        feedback: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "Poly", unit = "Generic")]
        poly_mode: f32, // Should be a bool?

        #[model(min = 0.0, max = 5000.0)]
        #[parameter(name = "Portamento", unit = "Generic", gradient = "Power(2.0)")]
        portamento: f32,
    }
}

impl Default for PhaseyModel {
    fn default() -> Self {
        PhaseyModel {
            dry: 1.0,
            wet: 0.5,
            attack: 5.0,
            release: 5.0,

            pitch_bend_range: 2.0,
            feedback: 0.2,
            poly_mode: 1.0,
            portamento: 0.0,
        }
    }
}

struct Phasey {
    voices: [MidiVoice; MAX_POLYPHONY],
    damper: bool,
    damper_stop_queue: Vec<u8>,
    pitch_bend: Smooth<f32>,

    //scratch_buffer: [[f32; MAX_BLOCKSIZE]; 2], // baseplug::MAX_BLOCKSIZE
    //temp_output: [[f32; MAX_BLOCKSIZE]; 2], // baseplug::MAX_BLOCKSIZE
}

impl Plugin for Phasey {
    const NAME: &'static str = "Phaseys";
    const PRODUCT: &'static str = "Phaseys";
    const VENDOR: &'static str = "Fractalysoft";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = PhaseyModel;

    #[inline]
    fn new(_sample_rate: f32, _model: &PhaseyModel) -> Self {
        let mut pitch_bend = Smooth::new(0.0);
        pitch_bend.set_speed_ms(_sample_rate, PITCH_BEND_SMOOTHING);

        Phasey {
            voices: [(); MAX_POLYPHONY].map(|_| MidiVoice::new(MAX_DELAY, _sample_rate)), //Idk why but I need this to initialize an array lol
            damper: false,
            damper_stop_queue: vec![],
            pitch_bend,

            //scratch_buffer: [[0.0f32; MAX_BLOCKSIZE]; 2], // Should be baseplug::MAX_BLOCKSIZE
            //temp_output: [[0.0f32; MAX_BLOCKSIZE]; 2], // Should be baseplug::MAX_BLOCKSIZE
        }
    }

    #[inline]
    fn process(&mut self, model: &PhaseyModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        debug_assert!(ctx.nframes <= MAX_BLOCKSIZE);

        self.pitch_bend.process(ctx.nframes);

        /*// Zero out the temp outputs
        bt::set(&mut self.temp_output[0], 0.0);
        bt::set(&mut self.temp_output[1], 0.0);

        // Block-wise processing
        for v in 0..MAX_POLYPHONY {
            if self.voices[v].is_playing() {

                self.voices[v].process(
                    input,
                    output,
                    self.pitch_bend.output().values,
                    model.portamento.values,
                    model.feedback.values,
                );

                bt::add(&mut self.temp_output[0], &self.scratch_buffer[0]);
                bt::add(&mut self.temp_output[1], &self.scratch_buffer[1]);
            }
        }

        // Apply the dry/wet mix
        for i in 0..2 {
            bt::copy(output[i], input[i]); // In case this isn't done automatically
            bt::mul(output[i], model.dry.values);
            bt::mul(&mut self.temp_output[i], model.wet.values);
            bt::add(output[i], &self.temp_output[i]);
        }*/

        for i in 0..ctx.nframes {

            let mut delayed_l = 0.0;
            let mut delayed_r = 0.0;
            for v in 0..MAX_POLYPHONY {
                if self.voices[v].is_playing() {
                    self.voices[v].set_portamento(ctx.sample_rate, model.portamento[i]);

                    let t = self.voices[v].read( self.pitch_bend[i]*model.pitch_bend_range[i] );
                    delayed_l += t.0;
                    delayed_r += t.1;

                    self.voices[v].push(
                        input[0][i] + delayed_l * model.feedback[i],
                        input[1][i] + delayed_r * model.feedback[i]
                    );

                }
            }

            // Mix to outputs
            output[0][i] = input[0][i] * model.dry[i] + delayed_l * model.wet[i];
            output[1][i] = input[1][i] * model.dry[i] + delayed_r * model.wet[i];
        }
    }
}

impl MidiReceiver for Phasey {
    fn midi_input(&mut self, _model: &PhaseyModelProcess, msg: [u8; 3]) {
        match msg[0] {
            // note on
            0x90 => {
                // if we're monophonic, replace a voice
                if _model.poly_mode[0] < 0.5f32{
                    if let Some(i) = self.voices.iter().position(|v| v.is_playing()) {
                        self.voices[i].change_note(msg[1], msg[2] as f32 / 127.0);
                    }
                    // No voice yet
                    else{
                        self.voices[0].play(msg[1], msg[2] as f32 / 127.0, _model.attack[0]);
                    }
                }
                
                // if we're polyphonic, find the first free voice
                else if let Some(i) = self.voices.iter().position(|x| !x.is_playing()) {
                    self.voices[i].play(msg[1], msg[2] as f32 / 127.0, _model.attack[0]);
                }
            },

            // note off
            0x80 => {
                if self.damper {
                    self.damper_stop_queue.push(msg[1]);
                }
                else{
                    for x in self.voices.iter_mut().filter(|x| x.is_playing() && x.get_midi_note() == msg[1]) {
                        x.stop(_model.release[0])
                    }
                }
            },

            // pitch bend
            0xE0 => {
                // v I hope this is -1 to 1 lmao but i am stupid
                self.pitch_bend.set(msg[2] as f32 / 64.0f32 + msg[1] as f32 / 8192.0f32 - 1.0f32);
            },

            // control change
            0xB0 => {
                // damper pedal
                if msg[1] == 0x40 {
                    if msg[2] >= 0x40 {
                        self.damper = true;
                    }
                    else {
                        self.damper = false;
                        for i in self.damper_stop_queue.iter() {
                            for x in self.voices.iter_mut().filter(|x| x.is_playing() && x.get_midi_note() == *i) {
                                x.stop(_model.release[0])
                            }
                        }
                        self.damper_stop_queue.clear();
                    }
                }
            }

            _ => ()
        }
    }
}

#[cfg(not(test))]
baseplug::vst2!(Phasey, b"FRps");