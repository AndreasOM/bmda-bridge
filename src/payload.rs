#[derive(Debug)]
pub enum Payload {
    KeOn { who: u8, index: u8, state: u8 },
}
