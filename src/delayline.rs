extern crate num;
extern crate ringbuf;

use ringbuf::{Consumer, Producer, RingBuffer};


pub struct DelayLine<T> {
    cx: Consumer<T>,
    px: Producer<T>,

    last_output: T,
}

impl DelayLine<f32> {
    pub fn new(bufsize: usize) -> Self {
        let (px, cx) = RingBuffer::new(bufsize).split();
        
        Self { cx, px, last_output: 0.0f32 }
    }
    
    pub fn push(&mut self, x: f32) {
        let _ = self.px.push(x);
    }

    pub fn pop(&mut self, target_bufsize: f32) -> f32 {
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
    }

    fn lerp(&self, x: f32, y: f32, t: f32) -> f32 {
        x + (y - x) * t
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
    }
}