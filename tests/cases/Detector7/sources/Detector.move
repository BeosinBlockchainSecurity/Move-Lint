module NamedAddr::Detector {
    public fun func1(x: u64, y: u64, z:u64) {
        let _a = x * y / z;
        let _b = x / z * y; // <Issue:7>
    }
}