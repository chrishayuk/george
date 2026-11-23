//! GEORGE, the General Order Generator: C. L. Hamblin's reverse-Polish
//! programming scheme for the English Electric DEUCE, University of New
//! South Wales, 1957-58, reconstructed from its programming manual.

pub mod deuce;
pub mod functions;
pub mod machine;
pub mod number;
pub mod symbols;
pub mod text;

pub use machine::{Event, Machine, Options, Outcome, Stop};
pub use number::{Card, Num};
pub use symbols::Sym;
pub use text::{parse, Program};
