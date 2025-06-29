//! Fork-specific opcode implementations

pub mod berlin;
pub mod byzantium;
pub mod cancun;
pub mod constantinople;
pub mod frontier;
pub mod homestead;
pub mod istanbul;
pub mod london;
pub mod shanghai;

pub use berlin::Berlin;
pub use byzantium::Byzantium;
pub use cancun::Cancun;
pub use constantinople::Constantinople;
pub use frontier::Frontier;
pub use homestead::Homestead;
pub use istanbul::Istanbul;
pub use london::London;
pub use shanghai::Shanghai;
