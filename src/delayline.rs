extern crate num;
extern crate ringbuf;

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
    max_speed: f32
}

impl DelayLine<f32> {
    // max_speed is the amount of samples we're allowed to skip in 1 read
    // 0 means no max speed
    pub fn new(bufsize: usize, max_speed: f32) -> Self {
        let (px, cx) = RingBuffer::new(bufsize).split();
        
        Self { cx, px, last_bufsize: 0.0f32, max_speed }
    }
    
    pub fn push(&mut self, x: f32) {
        let _ = self.px.push(x);
    }

    /*pub fn pop(&mut self, target_bufsize: f32) -> f32 {
        let target_bufsize_int = target_bufsize.ceil() as usize;
        let mut current_bufsize = self.px.len();

        while current_bufsize > target_bufsize_int {

            let _ = self.cx.pop();

            // For some reason this has even more zipper noise??
            /*self.last_output = match self.cx.pop() {
                Some(x) => x,
                None => 0.0f32,
            };*/
            current_bufsize -= 1;
        }
        let mut delayed = 0.0f32;
        if current_bufsize == target_bufsize_int {
            delayed = match self.cx.pop() {
                Some(x) => x,
                None => 0.0f32,
            };
        }

        let return_value = self.lerp(self.last_output, delayed, target_bufsize_int as f32 - target_bufsize);
        self.last_output = delayed;

        return_value
    }*/

    pub fn pop(&mut self, target_bufsize: f32) -> f32{

        let mut current_bufsize = self.px.len();

        let mut delta_bufsize = target_bufsize - self.last_bufsize;
        // CASE 1: buffer still accumulating, first sample has not played yet
        if self.last_bufsize == 0.0f32 && current_bufsize < target_bufsize.ceil() as usize { return 0.0f32 }
        // CASE 2: delta_bufsize < 0, buffer shrinking, we have plenty of samples but don't go too fast
        else if delta_bufsize < -self.max_speed { delta_bufsize = -self.max_speed; }
        // CASE 3: delta_bufsize > 0, buffer increasing, we don't have enough samples so just move over by 1
        else if delta_bufsize > 1.0f32 { delta_bufsize = 1.0f32 }
        // CASE 4: delta_bufsize = 0, we chillin

        let next_bufsize = self.last_bufsize + delta_bufsize;
        let next_bufsize_ceil = next_bufsize.ceil() as usize;

        // ASSERTIONS
        if delta_bufsize < 0.0f32 {
            assert!( (current_bufsize - 1 - next_bufsize_ceil) as f32 <= self.max_speed, "CASE 2" );
        }
        else if delta_bufsize > 0.0f32 {
            assert!( next_bufsize_ceil - (current_bufsize - 1) <= 1, "CASE 3" );
        }
        else if delta_bufsize == 0.0f32 {
            assert!( next_bufsize_ceil - current_bufsize - 1 == 0, "CASE 4" );
        }

        while current_bufsize > next_bufsize_ceil {
            let _ = self.cx.pop();
            current_bufsize -= 1;
        }

        let slices = self.cx.as_slices();
        let relevant_samples = (
            slice_pair_index(slices.0, slices.1, 0),
            slice_pair_index(slices.0, slices.1, 1)
        );
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

    #[test]
    fn it_works() {
        let mut delay_1 = DelayLine::new(44100, 2.0f32);
        
        for _i in 0..4000 {
            delay_1.pop(1000f32);
            delay_1.push(1.0f32);
        }
        for _i in 0..4000 {
            delay_1.pop(500f32);
            delay_1.push(1.0f32);
        }
        for _i in 0..4000 {
            delay_1.pop(2000f32);
            delay_1.push(1.0f32);
        }
    }
}