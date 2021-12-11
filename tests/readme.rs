mod first {
    use hackfn::hackfn;

    /// Function object that adds some number to its input.
    struct Plus(u32);

    #[hackfn]
    impl Plus {
        fn call(&self, other: u32) -> u32 {
            self.0 + other
        }
    }

    #[test]
    fn main() {
        let plus_one = Plus(1);
        let sum = plus_one(2);
        assert_eq!(sum, 3);
    }
}

mod second {
    use hackfn::hackfn;

    use std::cell::Cell;
    use std::ops::Add;

    /// Function object that accumulates a pair of values per call.
    #[derive(Default)]
    struct AccumulatePairs<T> {
        first: Cell<T>,
        second: Cell<T>,
    }

    #[hackfn]
    impl<T> AccumulatePairs<T> where T: Copy + Add<Output = T> {
        fn call(&self, first: T, second: T) {
            self.first.set(self.first.get() + first);
            self.second.set(self.second.get() + second);
        }
    }

    #[test]
    fn main() {
        let accumulate = AccumulatePairs::default();
        accumulate(30, 1);
        accumulate(20, 2);
        accumulate(10, 3);
        assert_eq!(accumulate.first.get(), 60);
        assert_eq!(accumulate.second.get(), 6);
    }
}
