module NamedAddr::Detector {
    const ERROR_NUM: u64 = 2;
    public fun func1(x: u64) {
        let y = x + 1;
        assert(x > 10, ERROR_NUM); // <Issue:1>
    }
    public fun func2(x: u64) {
        let y = x + 1;
        assert!(x > 10, ERROR_NUM); // <Issue:1>
    }
    public fun func3(x: u64) {
        let y = x + 1;
        assert!(if(x > 10) {true} else {false}, ERROR_NUM); // <Issue:1>
    }
    public fun func4(x: u64) {
        assert!(x > 10, ERROR_NUM);
        let y = x + 1;
    }
}