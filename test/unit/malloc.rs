use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::alloc::Layout;

use crate::mem::{kalloc, malloc::Heap};

struct T<const N: usize> {
    data: [u8; N],
    index: usize,
}

impl<const N: usize> T<N> {
    fn new(index: usize) -> T<N> {
        let data = core::array::from_fn(|_| index as u8);
        T { data, index }
    }
}

impl<const N: usize> Drop for T<N> {
    fn drop(&mut self) {
        for id in self.data {
            assert_eq!(id, self.index as u8);
        }
    }
}

#[derive(Clone)]
pub struct Collatz(pub usize);
impl Iterator for Collatz {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let n0 = self.0;
        self.0 = if n0 % 2 == 0 { n0 / 2 } else { n0 * 3 + 1 };
        if n0 == 0 {
            None
        } else {
            Some(n0)
        }
    }
}

fn vec_exhaustive() {
    let mut test: VecDeque<T<97>> = VecDeque::new();
    let before = Heap::get().allocated();

    let mut unit = |n: usize| {
        let size = test.len();

        for i in 0..n {
            test.push_front(T::new(i));
        }
        assert_eq!(test.len(), size + n);

        for i in 0..n {
            test.push_back(T::new(n + i));
        }
        assert_eq!(test.len(), size + n * 2);

        for _ in 0..n {
            drop(test.pop_back());
        }
        assert_eq!(test.len(), size + n);

        for _ in 0..n {
            drop(test.pop_front());
        }
        assert_eq!(test.len(), size);
    };

    for n in (27..).flat_map(Collatz).take(20) {
        unit(n % 997);
    }

    kprintln!(
        "Exhaustive VecDeque {}, {}",
        before,
        Heap::get().allocated()
    );
}

fn vec_simple() {
    let m = Heap::get();
    let before = m.allocated();
    {
        let mut v = Vec::new();
        (0..4).for_each(|i| v.push(i));
        assert!(m.allocated() - before == 16);

        v.push(4);
        assert!(m.allocated() - before == 32);

        (5..8).for_each(|i| v.push(i));
        assert!(m.allocated() - before == 32);

        v.push(8);
        assert!(m.allocated() - before == 64);
    }
    assert_eq!(before, m.allocated());
}

fn layout() {
    fn single_size<const N: usize, const ASIZE: usize>() {
        let m = Heap::get();
        let before = m.allocated();
        unsafe {
            let l = Layout::new::<T<N>>();
            let a = l.align();
            let p = kalloc(l.size(), l.align());
            assert!(m.allocated() - before == ASIZE);
            assert!(p as usize % a == 0);
            m.dealloc(p, Layout::new::<T<N>>());
            assert!(m.allocated() == before);
        }
    }

    single_size::<1, 16>();
    single_size::<23, 32>();
    single_size::<24, 32>();
    single_size::<25, 64>();
    single_size::<1016, 1024>();
    single_size::<1017, 2048>();
    single_size::<4088, 4096>();
    single_size::<4096, 8192>();

    let _ = Box::new([0xccu8; 4096 * 2]);

    // this line will panic due to stack overflow
    // let _ = Box::new([0xccu8; 4096 * 4]);
}

pub fn main() {
    vec_simple();
    vec_exhaustive();

    layout();
}
