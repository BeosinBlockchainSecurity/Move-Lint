module NamedAddr::Detector {
    const ERROR_NUM: u64 = 2;
    public fun func1(x: u64) {
        assert!(x > 0, 0); // <Issue:2>
        assert(x > 0, 0); // <Issue:2>
        assert!(x > 0, ERROR_NUM);
    }
}