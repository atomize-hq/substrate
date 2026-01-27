//! To print messages while editing a line
//!
//! See example:
//!
//! ``` shell
//! cargo run --example external_printer --features=external_printer
//! ```
#[cfg(feature = "external_printer")]
use {
    crossbeam::channel::{bounded, Receiver, SendError, Sender},
    std::fmt::Display,
};

#[cfg(feature = "external_printer")]
pub const EXTERNAL_PRINTER_DEFAULT_CAPACITY: usize = 20;
#[cfg(feature = "external_printer")]
pub const EXTERNAL_BYTE_PRINTER_DEFAULT_CAPACITY: usize = 256;

/// An ExternalPrinter allows to print messages of text while editing a line.
/// The message is printed as a new line, the line-edit will continue below the
/// output.
///
/// ## Required feature:
/// `external_printer`
#[cfg(feature = "external_printer")]
#[derive(Debug, Clone)]
pub struct ExternalPrinter<T>
where
    T: Display,
{
    sender: Sender<T>,
    receiver: Receiver<T>,
}

#[cfg(feature = "external_printer")]
impl<T> ExternalPrinter<T>
where
    T: Display,
{
    /// Creates an ExternalPrinter to store lines with a max_cap
    pub fn new(max_cap: usize) -> Self {
        let (sender, receiver) = bounded::<T>(max_cap);
        Self { sender, receiver }
    }
    /// Gets a Sender to use the printer externally by sending lines to it
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }
    /// Receiver to get messages if any
    pub fn receiver(&self) -> &Receiver<T> {
        &self.receiver
    }

    /// Convenience method if the whole Printer is cloned, blocks if max_cap is reached.
    ///
    pub fn print(&self, line: T) -> Result<(), SendError<T>> {
        self.sender.send(line)
    }

    /// Convenience method to get a line if any, doesn't block.
    pub fn get_line(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(feature = "external_printer")]
impl<T> Default for ExternalPrinter<T>
where
    T: Display,
{
    fn default() -> Self {
        Self::new(EXTERNAL_PRINTER_DEFAULT_CAPACITY)
    }
}

/// An ExternalBytePrinter allows to print raw bytes while editing a line.
///
/// This is intended for terminal/PTY byte streams which may be non-UTF8.
///
/// ## Required feature:
/// `external_printer`
#[cfg(feature = "external_printer")]
#[derive(Debug, Clone)]
pub struct ExternalBytePrinter {
    sender: Sender<Vec<u8>>,
    receiver: Receiver<Vec<u8>>,
}

#[cfg(feature = "external_printer")]
impl ExternalBytePrinter {
    /// Creates an ExternalBytePrinter to store byte chunks with a max_cap.
    pub fn new(max_cap: usize) -> Self {
        let (sender, receiver) = bounded::<Vec<u8>>(max_cap);
        Self { sender, receiver }
    }

    /// Gets a Sender to use the printer externally by sending byte chunks to it.
    pub fn sender(&self) -> Sender<Vec<u8>> {
        self.sender.clone()
    }

    /// Receiver to get byte chunks if any.
    pub fn receiver(&self) -> &Receiver<Vec<u8>> {
        &self.receiver
    }

    /// Convenience method if the whole printer is cloned, blocks if max_cap is reached.
    pub fn print(&self, bytes: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        self.sender.send(bytes)
    }

    /// Convenience method to get a byte chunk if any, doesn't block.
    pub fn get_bytes(&self) -> Option<Vec<u8>> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(feature = "external_printer")]
impl Default for ExternalBytePrinter {
    fn default() -> Self {
        Self::new(EXTERNAL_BYTE_PRINTER_DEFAULT_CAPACITY)
    }
}
