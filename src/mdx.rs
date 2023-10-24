use std::collections::VecDeque;
use std::iter::{Fuse, FusedIterator};

enum Element {

}

struct Stream<I> where I: Iterator {
    iter: Fuse<I>,
    peeked: VecDeque<I::Item>
}

impl<I> Stream<I> where I: Iterator {
    pub fn new(it: I) -> Self { Self { iter: it.fuse(), peeked: VecDeque::new() }}

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_index(0)
    }

    pub fn peek_index(&mut self, n: usize) -> Option<&I::Item> {
        while n >= self.peeked.len() {
            // dbg!(&self.peeked.len());
            if let Some(item) = self.iter.next() {
                self.peeked.push_back(item);
            } else {
                return None;
            }
        }
        Some(&self.peeked[n])
    }

    pub fn peek_slice(&mut self, n: usize) -> &[I::Item] {
        if n == 0 {
            return &[];
        }
        self.peek_index(n - 1);
        if n < self.peeked.len() {
            &self.peeked.make_contiguous()[0..n]
        } else {
            &self.peeked.make_contiguous()[0..]
        }
    }
}

impl<I> Stream<I> where I: Iterator<Item=char> {
    pub fn peek_string(&mut self, len: usize) -> String {
        String::from_iter(self.peek_slice(len))
    }

    pub fn startswith(&mut self, chars: impl AsRef<str>) -> bool {
        let chars = chars.as_ref();
        for (i, chr) in chars.char_indices() {
            if !self.peek_index(i).is_some_and(|c| c == &chr) {
                return false;
            }
        }
        return true;
    }
}

impl<I> Iterator for Stream<I> where I: Iterator {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_empty() {
            self.iter.next()
        } else {
            self.peeked.pop_front()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for Stream<I> where I: Iterator { }
impl<I> ExactSizeIterator for Stream<I> where I: ExactSizeIterator { }


fn parse(text: &str) {
    let mut text = Stream::new(text.chars());

    text.peek_string(3) == "```";
}


#[cfg(test)]
mod test {
    use crate::mdx::Stream;

    #[test]
    fn test_peek() {
        let v = vec![1, 2, 3, 4, 5];
        let mut s = Stream::new(v.into_iter());
        assert_eq!(s.peek(), Some(&1));
        assert_eq!(s.peek(), Some(&1));
        assert_eq!(s.peek_index(1), Some(&2));
        assert_eq!(s.peek_index(1), Some(&2));
        assert_eq!(s.peek_index(0), Some(&1));

        assert_eq!(s.next(), Some(1));
        assert_eq!(s.peek_index(1), Some(&3));
        assert_eq!(s.peek_index(0), Some(&2));
        assert_eq!(s.next(), Some(2));
        assert_eq!(s.next(), Some(3));
        assert_eq!(s.next(), Some(4));
        assert_eq!(s.peek_index(0), Some(&5));
        assert_eq!(s.peek_index(1), None);
        assert_eq!(s.next(), Some(5));
        assert_eq!(s.next(), None);
        assert_eq!(s.peek_index(0), None);
        assert_eq!(s.peek_index(1), None);
    }

    #[test]
    fn test_peek_slice() {
        let v = vec![1, 2, 3, 4, 5];
        let mut s = Stream::new(v.into_iter());

        let empty: &[i32] = &[];
        assert_eq!(s.peek_slice(0), empty);
        assert_eq!(s.peek_slice(0), empty);
        assert_eq!(s.peek_slice(3), &[1, 2, 3]);
        assert_eq!(s.peek_index(2), Some(&3));
        assert_eq!(s.peek_slice(6), &[1, 2, 3, 4, 5]);
        assert_eq!(s.peek_index(2), Some(&3));
        assert_eq!(s.next(), Some(1));
        assert_eq!(s.next(), Some(2));
        assert_eq!(s.next(), Some(3));
        assert_eq!(s.next(), Some(4));
        assert_eq!(s.next(), Some(5));
        assert_eq!(s.peek_slice(1), empty);
    }

    #[test]
    fn test_char_stream() {
        let string = "hullo";
        let mut s = Stream::new(string.chars());

        assert!(s.startswith("hul"));
        s.next();
        assert!(!s.startswith("hul"));
    }
}