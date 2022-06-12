
#[allow(dead_code)]
pub mod buftools{

    use std::ops::{AddAssign, MulAssign};

    pub fn set<T: Copy>(arr: &mut [T], val: T) {
        for x in arr.iter_mut() {
            *x = val;
        }
    }

    pub fn copy<T: Copy>(arr1: &mut [T], arr2: &[T]) {
        for (x, y) in arr1.iter_mut().zip(arr2.iter()) {
            *x = *y;
        }
    }


    pub fn add_c<T: Copy + AddAssign>(arr: &mut [T], val: T) {
        for x in arr.iter_mut() {
            *x += val;
        }
    }

    pub fn add<T: Copy + AddAssign>(arr1: &mut [T], arr2: &[T]) {
        for (x, y) in arr1.iter_mut().zip(arr2.iter()) {
            *x += *y;
        }
    }

    pub fn mul_c<T: Copy + MulAssign>(arr: &mut [T], val: T) {
        for x in arr.iter_mut() {
            *x *= val;
        }
    }

    pub fn mul<T: Copy + MulAssign>(arr1: &mut [T], arr2: &[T]) {
        for (x, y) in arr1.iter_mut().zip(arr2.iter()) {
            *x *= *y;
        }
    }

}



/*

// Either a fixed size vector or a reference to memory somewhere

pub struct Buffer<'a, T: Copy> {
    pub data: &'a mut [T],
}

impl<'a, T: Copy> Buffer<'a, T> {
    pub fn new(data: &'a mut [T]) -> Self {
        Self { data }
    }

    pub fn set(&mut self, value: T) {
        for x in self.data.iter_mut() {
            *x = value;
        }
    }

    pub fn copy(&mut self, arr: &[T]) {
        for (x, y) in self.data.iter_mut().zip(arr.iter()) {
            *x = *y;
        }
    }
}

impl<T: Copy> ops::Deref for Buffer<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T: Copy> ops::Index<usize> for Buffer<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Copy> ops::IndexMut<usize> for Buffer<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}

impl<T: ops::AddAssign + Copy> ops::AddAssign<T> for Buffer<'_, T> {
    fn add_assign(&mut self, rhs: T) {
        for x in self.data.iter_mut() {
            *x += rhs;
        }
    }
}

impl<'a, T: ops::AddAssign + Copy> ops::AddAssign<&[T]> for Buffer<'a, T> {
    fn add_assign(&mut self, rhs: &[T]) {
        debug_assert!(self.len() <= rhs.len());

        for i in 0..self.len() {
            self[i] += rhs[i];
        }
    }
}

impl<'a, T: ops::AddAssign + Copy, const S: usize> ops::AddAssign<[T;S]> for Buffer<'a, T> {
    fn add_assign(&mut self, rhs: [T;S]) {
        debug_assert!(self.len() <= rhs.len());

        for i in 0..self.len() {
            self[i] += rhs[i];
        }
    }
}

impl<'a, 'b, T: ops::AddAssign + Copy> ops::AddAssign<Buffer<'b, T>> for Buffer<'a, T> {
    fn add_assign(&mut self, rhs: Buffer<'b, T>) {
        debug_assert!(self.len() <= rhs.len());

        for i in 0..self.data.len() {
            self[i] += rhs[i];
        }
    }
}

impl<T: ops::MulAssign + Copy> ops::MulAssign<T> for Buffer<'_, T> {
    fn mul_assign(&mut self, rhs: T) {
        for x in self.data.iter_mut() {
            *x *= rhs;
        }
    }
}

impl<T: ops::MulAssign + Copy> ops::MulAssign<&[T]> for Buffer<'_, T> {
    fn mul_assign(&mut self, rhs: &[T]) {
        debug_assert!(self.len() <= rhs.len());

        for i in 0..self.len() {
            self[i] *= rhs[i];
        }
    }
} */