// module NamedAddr::Detector1 {
//     public fun func1(x: u64) {
//         x = x + 1;
//         assert!(x > 10, 0);
//     }
// }

module NamedAddr::Detector1 {
    use NamedAddr::T;
    public fun func1(x: u64): u64 {
        assert!(x > 10, 0);
        assert!(if(x > 10) {true} else {false}, 0);
        T::f1();
        T::f2(x);
        func3();
        func2(x);
        x + 1
    }

    public fun func2(x: u64): u64 {
        let y = x + 1;
        let y = 2;
        assert!(x > 10, 0);
        if (x > 20) {
            x = x + 1;
            y
        } else {
            let x = x + 1;
            y
        }
        // return y;
    }

    public fun func3() {
        use NamedAddr::T;
    }

    public fun func4(x: u64) {
        use NamedAddr::T;
        x + 1;
        let y1: u16;
        assert!(x > 10, 0);
    }
}

module NamedAddr::T {
    public fun f1() {}
    public fun f2(x: u64): u64 {x}
}