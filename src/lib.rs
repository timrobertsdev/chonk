#![feature(min_specialization)]
#![feature(trusted_len)]
#![feature(trusted_random_access)]

use std::iter::{TrustedLen, FusedIterator, TrustedRandomAccess, TrustedRandomAccessNoCoerce};

/// something something TODO
#[derive(Debug)]
pub struct Chonks<'a, T: 'a> {
    v: &'a [T],
    chunk_size: usize,
    read_size: usize,
}

impl<'a, T> Chonks<'a, T> {
    #[inline]
    pub fn new(slice: &'a [T], chunk_size: usize, read_ahead: usize) -> Self {
        Self { v: slice, chunk_size, read_size: chunk_size + read_ahead }
    }
}

impl<T> Copy for Chonks<'_, T> { }

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<T> Clone for Chonks<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Iterator for Chonks<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() {
            None
        } else if self.v.len() < self.read_size {
            let last = self.v;
            self.v = &[];
            Some(last)
        } else {
            let (fst, snd) = (&self.v[..self.read_size], &self.v[self.chunk_size..]);
            self.v = snd;
            Some(fst)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else if self.v.len() < self.read_size {
            (1, Some(1))
        } else {
            let n = self.v.len() / self.read_size;
            // not the functional remainder, but lets us know that we have more work to do
            let rem = self.v.len() % self.read_size;
            let n = if rem > 0 { n + 1 } else { n };
            (n, Some(n))
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (start, overflow) = n.overflowing_mul(self.read_size);
        if start >= self.v.len() || overflow {
            self.v = &[];
            None
        } else {
            let end = match start.checked_add(self.read_size) {
                Some(i) => std::cmp::min(self.v.len(), i),
                None => self.v.len(),
            };
            let nth = &self.v[start..end];
            self.v = &self.v[end..];
            Some(nth)
        }
    }

    // TODO: __iterator_get_unchecked ?? maybe
}

impl<T> ExactSizeIterator for Chonks<'_, T> { }

unsafe impl<T> TrustedLen for Chonks<'_, T> { }

impl<T> FusedIterator for Chonks<'_, T> { }

unsafe impl<'a, T> TrustedRandomAccess for Chonks<'a, T> { }

unsafe impl<'a, T> TrustedRandomAccessNoCoerce for Chonks<'a, T> {
    const MAY_HAVE_SIDE_EFFECT: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chonks() {
        let mut buf = [0u8; 256];

        for i in 0..256 {
            buf[i] = i as u8;
        }

        let chonks = Chonks::new(&buf, 32, 4);

        for (idx, chonk) in chonks.enumerate() {
            dbg!(idx + 1);
            dbg!(chonk.len());
        }
    }
}