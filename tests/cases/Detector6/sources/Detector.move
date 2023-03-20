module 0x1::NFT {
    public fun register() {}
    public fun register1() {}
}
module NamedAddr::Detector {
    use 0x1::NFT;
    public fun func() {
        NFT::register(); // <Issue:6>
        NFT::register1();
    }
}