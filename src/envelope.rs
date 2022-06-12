extern crate num;

#[derive(PartialEq)]
enum EnvelopeState {
    Attack,
    Sustain,
    Release,
    Off,
}

pub struct Envelope {
    state: EnvelopeState,
    env_size: usize,
    samples_left: usize,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            state: EnvelopeState::Off,
            env_size: 0,
            samples_left: 0,
        }
    }

    pub fn attack(&mut self, time: f32, sample_rate: f32) {
        self.state = EnvelopeState::Attack;
        self.env_size = (time / 1000.0f32 * sample_rate).ceil() as usize;
        self.samples_left = self.env_size;
    }

    pub fn release(&mut self, size: f32, sample_rate: f32) {
        if self.state == EnvelopeState::Release || self.state == EnvelopeState::Off {
            return;
        }

        self.state = EnvelopeState::Release;
        self.env_size = (size / 1000.0f32 * sample_rate).ceil() as usize;
        self.samples_left = self.env_size;
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Attack => {
                self.samples_left -= 1;
                if self.samples_left == 0 {
                    self.state = EnvelopeState::Sustain;
                    self.samples_left = 0;
                }
                (self.env_size - self.samples_left) as f32 / self.env_size as f32
            }
            EnvelopeState::Sustain => 1.0f32,
            EnvelopeState::Release => {
                self.samples_left -= 1;
                if self.samples_left == 0 {
                    self.state = EnvelopeState::Off;
                    self.samples_left = 0;
                }
                1.0f32 - ((self.env_size - self.samples_left) as f32 / self.env_size as f32)
            }
            EnvelopeState::Off => 0.0f32,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state != EnvelopeState::Off
    }

    pub fn legato(&mut self) {
        self.state = EnvelopeState::Sustain; // For now
    }
}
