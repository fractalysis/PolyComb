#[path = "envelope.rs"] mod envelope;
#[path = "delayline.rs"] mod delayline;

use baseplug::Smooth;
use envelope::Envelope;
use delayline::DelayLine;

//const MAX_BLOCKSIZE: usize = 128; // make soto fix this lol

pub struct MidiVoice {
    sample_rate: f32,
    note: u8,
    velocity: f32,
    target_delay: Smooth<f32>, // In seconds
    delay_l: DelayLine<f32>,
    delay_r: DelayLine<f32>,
    env: Envelope,

    //scratch_buffer: [usize; MAX_BLOCKSIZE], // To get the target_bufsize for the delaylines
}

impl MidiVoice {
    pub fn new(max_delay: f32, sample_rate: f32) -> Self {
        let max_bufsize = (max_delay / 1000.0f32 * sample_rate).ceil() as usize;

        MidiVoice {
            sample_rate,
            note: 0,
            velocity: 0.0f32,
            target_delay: Smooth::new(0.0f32),
            delay_l: DelayLine::new(max_bufsize as usize),
            delay_r: DelayLine::new(max_bufsize as usize),
            env: Envelope::new(),

            //scratch_buffer: [0; MAX_BLOCKSIZE],
        }
    }

    pub fn is_playing(&self) -> bool {
        self.env.is_playing()
    }

    pub fn play(&mut self, note: u8, velocity: f32, attack: f32) {
        self.note = note;
        self.velocity = velocity;
        self.target_delay.reset(Self::midi_to_seconds(self.note));

        // Clear the delay lines to prepare
        self.delay_l.clear();
        self.delay_r.clear();

        // Start the attack envelope
        self.env.attack(attack, self.sample_rate);
    }

    pub fn stop(&mut self, release: f32) {
        // Start release envelope
        self.env.release(release, self.sample_rate);
    }

    //pitch_bend should be in semitones e.g. from -24 to +24
    pub fn read(&mut self, pitch_bend: f32) -> (f32, f32) {
        // Thing to multiply the delay by so it bends
        let pitch_bend_multiplier = 2.0f32.powf(pitch_bend / -12.0f32);
        self.target_delay.process(1); // Smoothed so it has portamento
        let target_bufsize = pitch_bend_multiplier * self.target_delay[0] * self.sample_rate;

        let delay_l = self.delay_l.pop(target_bufsize);
        let delay_r = self.delay_r.pop(target_bufsize);

        // Envelope
        let env_val = self.env.next_sample();

        (delay_l*env_val, delay_r*env_val)
    }

    pub fn push(&mut self, l: f32, r: f32) {
        self.delay_l.push(l);
        self.delay_r.push(r);
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

    /*pub fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], pitch_bend: &[f32], portamento: &[f32], feedback: &[f32]) {
        
        let nframes: usize = outputs[0].len();
        debug_assert!(nframes <= pitch_bend.len());
        debug_assert!(nframes <= MAX_BLOCKSIZE);

        // Turn pitch_bend into target_bufsize
        for i in 0..nframes {
            let pitch_bend_multiplier = 2.0f32.powf(pitch_bend[i] / -12.0f32);

            self.target_delay.set_speed_ms(self.sample_rate, portamento[i]);
            self.target_delay.process(1); // Smoothed so it has portamento

            self.scratch_buffer[i] = (pitch_bend_multiplier * self.target_delay[0] * self.sample_rate).ceil() as usize;
        }
        
        for ch in 0..2 {
            debug_assert!(inputs[ch].len() == nframes);
            debug_assert!(outputs[ch].len() == nframes);

            self.delay_l.process(&inputs[ch], outputs[ch], &self.scratch_buffer, feedback);
            self.delay_r.process(&inputs[ch], outputs[ch], &self.scratch_buffer, feedback);

            for i in 0..nframes {
                outputs[ch][i] *= self.env.next_sample();
            }
        }
    }*/


    // Private

    fn midi_to_seconds(midi: u8) -> f32 {
        (2.0 as f32).powf((69.0 - midi as f32) / 12.0) / 440.0
    }
}