use super::*;

fn single(ticks: i64) {
    let current = timer::timer_ticks();

    thread::sleep(ticks);
    assert_eq!(current, timer::timer_ticks());
}

pub mod negative {
    use super::*;

    pub fn main() {
        single(-100);
        pass();
    }
}

pub mod zero {
    use super::*;

    pub fn main() {
        single(0);
        pass();
    }
}
