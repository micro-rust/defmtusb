//! Buffer of the `defmt` logger.



/// The size of the buffer.
#[cfg(feature = "buffersize-64")]
const BUFFERSIZE: usize = 64;

#[cfg(feature = "buffersize-128")]
const BUFFERSIZE: usize = 128;

#[cfg(feature = "buffersize-256")]
const BUFFERSIZE: usize = 256;

#[cfg(feature = "buffersize-512")]
const BUFFERSIZE: usize = 512;

#[cfg(feature = "buffersize-1024")]
const BUFFERSIZE: usize = 1024;



pub(super) struct LogBuffer {
    /// Current state of the buffer.
    state: BufferState,

    /// Current cursor into the buffer.
    pub(super) cursor: usize,

    /// Buffered data.
    pub(super) data: [u8; BUFFERSIZE],
}

impl LogBuffer {
    /// Static initializer.
    pub const fn new() -> Self {
        Self {
            state: BufferState::Active,
            cursor: 0,
            data: [0u8; BUFFERSIZE],
        }
    }

    /// Marks the buffer to be flushed.
    #[inline]
    pub(super) fn flush(&mut self) {
        self.state = BufferState::Flush
    }

    /// Resets the buffer.
    pub(super) fn reset(&mut self) {
        self.state = BufferState::Active;
        self.cursor = 0;
    }

    /// Writes to the buffer.
    pub(super) fn write(&mut self, bytes: &[u8]) {
        // If not active, return immediately.
        if self.flushing() { return; }

        // Get the minimum size.
        let n = core::cmp::min(BUFFERSIZE - self.cursor, bytes.len());

        // Write the bytes.
        for i in 0..n {
            self.data[self.cursor + i] = bytes[i];
        }

        // Increment the cursor.
        self.cursor += n;

        // If limit reached, set as flush.
        if self.cursor >= (BUFFERSIZE - 2) {
            self.state = BufferState::Flush;
        }
    }

    /// Returns `true` if the given number of bytes can be written to the buffer.
    #[inline]
    pub(super) fn accepts(&self, n: usize) -> bool {
        ((self.cursor + n) < BUFFERSIZE) & self.writable()
    }

    /// Returns `true` if the buffer can be written to.
    #[inline]
    pub(super) fn writable(&self) -> bool {
        self.state == BufferState::Active
    }

    /// Returns `true` if the buffer is being flushed.
    #[inline]
    pub(super) fn flushing(&self) -> bool {
        self.state == BufferState::Flush
    }
}



/// The current state of the buffer.
#[derive(Clone, Copy, Eq, PartialEq)]
enum BufferState {
    /// This buffer can be written to.
    Active = 0,

    /// This buffer is full and must be flushed.
    Flush = 1,
}
