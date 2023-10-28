#[path = "envelope.rs"] mod envelope;

use baseplug::Smooth;
use envelope::Envelope;
use fundsp::{delay, audionode::Frame, prelude::AudioNode};
use typenum::U1;


pub struct MidiVoice {
    sample_rate: f32,
    note: u8,
    velocity: f32,
    target_delay: Smooth<f32>, // In seconds
    delay_l: delay::Tap<U1,f32>,
    delay_r: delay::Tap<U1,f32>,
    env: Envelope,

    //scratch_buffer: [usize; MAX_BLOCKSIZE], // To get the target_bufsize for the delaylines
}

impl MidiVoice {
    pub fn new(max_delay: f32, sample_rate: f32) -> Self {

        let mut ret = MidiVoice {
            sample_rate,
            note: 0,
            velocity: 0.0f32,
            target_delay: Smooth::new(0.0f32),
            delay_l: delay::Tap::new(0.001, max_delay),
            delay_r: delay::Tap::new(0.001, max_delay),
            env: Envelope::new(),

            //scratch_buffer: [0; MAX_BLOCKSIZE],
        };

        ret.delay_l.set_sample_rate(sample_rate as f64);
        ret.delay_r.set_sample_rate(sample_rate as f64);

        ret
    }

    pub fn is_playing(&self) -> bool {
        self.env.is_playing()
    }

    pub fn play(&mut self, note: u8, velocity: f32, attack: f32) {
        self.note = note;
        self.velocity = velocity;
        self.target_delay.reset(Self::midi_to_seconds(self.note));

        // Clear the delay lines to prepare
        self.delay_l.reset();
        self.delay_r.reset();

        // Start the attack envelope
        self.env.attack(attack, self.sample_rate);
    }

    pub fn stop(&mut self, release: f32) {
        // Start release envelope
        self.env.release(release, self.sample_rate);
    }

    pub fn tick(&mut self, l: f32, r: f32, pitch_bend: f32) -> (f32, f32) {
        // Thing to multiply the delay by so it bends
        let pitch_bend_multiplier = 2.0f32.powf(pitch_bend / -12.0f32);
        self.target_delay.process(1); // Smoothed so it has portamento
        let bent_delay = pitch_bend_multiplier * self.target_delay[0]; // In seconds

        let delay_l = self.delay_l.tick(&Frame::from([
            l, bent_delay
        ]))[0];
        let delay_r = self.delay_l.tick(&Frame::from([
            r, bent_delay
        ]))[0];

        let env_val = self.env.next_sample();

        (delay_l*env_val, delay_r*env_val)
    }

    pub fn get_midi_note(&self) -> u8 {
        self.note
    }

    pub fn change_note(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.velocity = velocity; // Smooth this too?
        self.env.legato();
        self.target_delay.set(Self::midi_to_seconds(note));
    }

    pub fn set_portamento(&mut self, sample_rate: f32, portamento: f32) {
        self.target_delay.set_speed_ms(sample_rate, portamento);
    }


    // Private

    fn midi_to_seconds(midi: u8) -> f32 {
        (2.0 as f32).powf((69.0 - midi as f32) / 12.0) / 440.0
    }
}