#[derive(Debug, Clone)]
pub struct ZeroSizedType;

#[derive(Debug, Clone)]
pub struct DebugDrop;

impl Drop for DebugDrop {
    fn drop(&mut self) {
        println!("DebugDrop with id: {:x} dropped!", &raw const self as usize % 0xffff)
    }
}