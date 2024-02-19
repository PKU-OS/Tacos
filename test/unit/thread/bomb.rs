#[allow(unused)]
pub fn main() {
    bomb(100);
    unreachable!("should overflow before you run 2^100 cycles!")
}

fn bomb(x: usize) {
    kprintln!("{}", x);

    if x > 0 {
        bomb(x - 1);
        bomb(x - 1);
    }
}
