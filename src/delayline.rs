extern crate num;
extern crate ringbuf;

use baseplug::Smooth;
use ringbuf::{Consumer, Producer, RingBuffer};


fn slice_pair_index(slice_1: &[f32], slice_2: &[f32], index: usize) -> f32{
    let use_slice_2 = index >= slice_1.len();
    let slice = if use_slice_2 { slice_2 } else { slice_1 };
    let index_mod = index % slice_1.len();
    return slice[index_mod];
}

fn lerp(x: f32, y: f32, t: f32) -> f32 {
    x + (y - x) * t
}

pub struct DelayLine<T> {
    cx: Consumer<T>,
    px: Producer<T>,

    last_bufsize: f32,
    speed: Smooth<f32>,
    max_speed: f32
}

impl DelayLine<f32> {
    // max_speed is the amount of samples we're allowed to skip in 1 read
    // 0 means no max speed
    pub fn new(bufsize: usize, max_speed: f32) -> Self {
        let (px, cx) = RingBuffer::new(bufsize + 1).split();
        
        Self { cx, px, 
            last_bufsize: 0.0f32, 
            speed: Smooth::new(0.0f32),
            max_speed
        }
    }
    
    pub fn push(&mut self, x: f32) {
        let _ = self.px.push(x);
    }

    pub fn pop(&mut self, target_bufsize: f32) -> f32{
        if self.cx.is_full() {
            let _ = self.cx.pop();
        }

        let target_bufsize_ceil = target_bufsize.ceil() as usize;
        
        // CASE 1: buffer still accumulating, first sample has not played yet
        if self.last_bufsize == 0.0f32 && self.cx.len() < target_bufsize_ceil { return 0.0f32 }
        // Or it might be done accumulating
        else if self.last_bufsize == 0.0f32 { self.last_bufsize = target_bufsize } 

        let mut delta_bufsize = target_bufsize - self.last_bufsize;
        
        // CASE 2: delta_bufsize < 0, buffer shrinking, we have plenty of samples but don't go too fast
        if delta_bufsize < -self.max_speed { delta_bufsize = -self.max_speed; }
        // CASE 3: delta_bufsize > 0, buffer increasing
        else if delta_bufsize > self.max_speed { delta_bufsize = self.max_speed }
        // CASE 4: delta_bufsize = 0, we chillin

        // some wack stuff happens if we don't have enough samples for case 3
        // but they should be pushing one sample every time before calling this
        if self.last_bufsize + delta_bufsize > self.cx.len() as f32 {
            delta_bufsize = 1.0f32;
        }

        let next_bufsize = self.last_bufsize + delta_bufsize;
        let next_bufsize_ceil = next_bufsize.ceil() as usize;
        let next_index = self.cx.len() - next_bufsize_ceil;

        let slices = self.cx.as_slices();
        let relevant_samples = (
            slice_pair_index(slices.0, slices.1, next_index),
            slice_pair_index(slices.0, slices.1, next_index + 1)
        );

        self.last_bufsize = next_bufsize;
        return lerp(relevant_samples.1, relevant_samples.0, next_bufsize_ceil as f32 - next_bufsize);
    }

    /*pub fn process(&mut self, inputs: &[T], outputs: &mut [T], target_bufsize: &[usize], feedback: &[T]) {
        for (i, input) in inputs.iter().enumerate() {
            let delayed = self.pop(target_bufsize[i]);
            outputs[i] = *input + delayed * feedback[i];
            self.push(outputs[i]);
        }
    }*/

    pub fn clear(&mut self) {
        let _ = self.cx.pop_each(|_x| true, None);
        self.last_bufsize = 0.0f32; // Should this be atomic?
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;
    use wav;

    #[test]
    fn it_works() {
        let mut delay_1 = DelayLine::new(44100, 2.0f32);
        let mut samples = Vec::new();

        for _i in 0..4000 {
            samples.push( delay_1.pop(1000f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(1000f32) );
            delay_1.push(1.0f32);
            samples.push( delay_1.pop(1000f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(1000f32) );
            delay_1.push(-1.0f32);
        }
        for _i in 0..4000 {
            samples.push( delay_1.pop(500f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(500f32) );
            delay_1.push(1.0f32);
            samples.push( delay_1.pop(500f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(500f32) );
            delay_1.push(-1.0f32);
        }
        for _i in 0..4000 {
            samples.push( delay_1.pop(44100f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(44100f32) );
            delay_1.push(1.0f32);
            samples.push( delay_1.pop(44100f32) );
            delay_1.push(0.0f32);
            samples.push( delay_1.pop(44100f32) );
            delay_1.push(-1.0f32);
        }

        let mut out_file = File::create(Path::new("test.wav")).unwrap();
        wav::write( wav::header::Header::new(
            wav::WAV_FORMAT_IEEE_FLOAT,
            1,
            44100,
            32
        ), &wav::BitDepth::from(samples), &mut out_file ).unwrap();
    }
}