use crate::sync::Semaphore;

use super::*;

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Start = 0,
    Acquire = 1,
    Down = 2,
    Finish = 4,
}

use self::State::*;

static mut STATUS: [State; 4] = [State::Start; 4];

fn low(pair: Arc<(Sleep, Semaphore)>) {
    let (lock, sema) = &*pair;
    unsafe {
        lock.acquire();
        STATUS[1] = Acquire;
        let expected = [Start, Acquire, Start, Start];
        assert_eq!(
            expected, STATUS,
            "When low acquires, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        sema.down();
        STATUS[1] = Down;
        let expected = [Start, Down, Start, Start];
        assert_eq!(
            expected, STATUS,
            "When low downs, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        lock.release();
        let expected = [Start, Down, Finish, Finish];
        assert_eq!(
            expected, STATUS,
            "When low releases, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        STATUS[1] = Finish;
    }
}

fn medium(pair: Arc<(Sleep, Semaphore)>) {
    let (_, sema) = &*pair;

    unsafe {
        sema.down();
        STATUS[2] = Down;

        let expected = [Start, Down, Down, Finish];
        assert_eq!(
            expected, STATUS,
            "When medium exits, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        STATUS[2] = Finish;
    }
}

fn high(pair: Arc<(Sleep, Semaphore)>) {
    let (lock, sema) = &*pair;

    unsafe {
        lock.acquire();
        STATUS[3] = Acquire;
        let expected = [Start, Down, Start, Acquire];
        assert_eq!(
            expected, STATUS,
            "When high acquires, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        sema.up();
        assert_eq!(
            expected, STATUS,
            "When high ups, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        lock.release();
        assert_eq!(
            expected, STATUS,
            "When high exits, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );

        STATUS[3] = Finish;
    }
}

pub fn main() {
    let pair = Arc::new((Sleep::default(), Semaphore::new(0)));

    let create = |f: fn(Arc<(Sleep, Semaphore)>), priority| {
        let p = Arc::clone(&pair);
        Builder::new(move || f(p))
            .name("child")
            .priority(priority)
            .spawn();
    };

    create(low, PRI_DEFAULT + 1);
    create(medium, PRI_DEFAULT + 3);
    create(high, PRI_DEFAULT + 5);

    let (_, sema) = &*pair;
    sema.up();

    let expected = [Start, Finish, Finish, Finish];
    unsafe {
        assert_eq!(
            expected, STATUS,
            "When main exits, expected states are {:?}. Actual states: {:?}",
            expected, STATUS
        );
    }

    pass();
}
