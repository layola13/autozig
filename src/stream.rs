//! # Stream Support for Zig FFI
//!
//! This module provides async streaming capabilities for Zig data types in
//! Rust. It enables Zig functions to produce streams of values that Rust can
//! consume asynchronously.
//!
//! ## Architecture
//!
//! - `ZigStream<T>`: Main stream type that implements `futures::Stream`
//! - Callback-based mechanism for Zig to push data to Rust
//! - Thread-safe state management using `Arc<Mutex<StreamState>>`
//!
//! ## Example
//!
//! ```rust,ignore
//! use autozig::stream::ZigStream;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//!     let stream = ZigStream::<u32>::new(rx);
//!
//!     // Simulate Zig pushing data
//!     tx.send(Ok(vec![1, 2, 3, 4])).unwrap();
//!     tx.send(Ok(vec![5, 6, 7, 8])).unwrap();
//!     drop(tx);
//!
//!     futures::pin_mut!(stream);
//!     while let Some(result) = stream.next().await {
//!         match result {
//!             Ok(value) => println!("Received: {}", value),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//! }
//! ```

use std::{
    marker::PhantomData,
    pin::Pin,
    sync::{
        Arc,
        Mutex,
    },
    task::{
        Context,
        Poll,
    },
};

/// Stream state machine
enum StreamState {
    /// Stream is active and receiving data
    Active {
        receiver: tokio::sync::mpsc::UnboundedReceiver<Result<Vec<u8>, String>>,
    },
    /// Stream has completed successfully
    Completed,
    /// Stream failed with an error
    #[allow(dead_code)]
    Failed(String),
}

/// A stream of values from Zig code
///
/// `ZigStream<T>` bridges Zig callbacks to Rust's async stream ecosystem.
/// It receives raw byte data from Zig and deserializes it into type `T`.
///
/// # Type Parameters
///
/// * `T` - The type of items in the stream. Must be deserializable from bytes.
///
/// # Thread Safety
///
/// `ZigStream` is `Send` and `Sync` when `T: Send`, allowing it to be used
/// across threads in async contexts.
pub struct ZigStream<T> {
    state: Arc<Mutex<StreamState>>,
    _phantom: PhantomData<T>,
}

impl<T> ZigStream<T> {
    /// Create a new `ZigStream` from an unbounded receiver
    ///
    /// # Arguments
    ///
    /// * `receiver` - Channel receiver that will receive data from Zig
    ///   callbacks
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use autozig::stream::ZigStream;
    ///
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// let stream = ZigStream::<u32>::new(rx);
    /// ```
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<Result<Vec<u8>, String>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(StreamState::Active { receiver })),
            _phantom: PhantomData,
        }
    }

    /// Check if the stream has completed
    pub fn is_completed(&self) -> bool {
        matches!(*self.state.lock().unwrap(), StreamState::Completed | StreamState::Failed(_))
    }

    /// Get the error message if the stream failed
    pub fn error(&self) -> Option<String> {
        match &*self.state.lock().unwrap() {
            StreamState::Failed(e) => Some(e.clone()),
            _ => None,
        }
    }
}

impl<T> futures::Stream for ZigStream<T>
where
    T: From<Vec<u8>>,
{
    type Item = Result<T, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.state.lock().unwrap();

        match &mut *state {
            StreamState::Active { receiver } => {
                // Poll the receiver for new data
                match receiver.poll_recv(cx) {
                    Poll::Ready(Some(Ok(data))) => {
                        // Successfully received data, convert to T
                        Poll::Ready(Some(Ok(T::from(data))))
                    },
                    Poll::Ready(Some(Err(e))) => {
                        // Received an error, but don't transition state
                        // Just return the error and continue polling
                        Poll::Ready(Some(Err(e)))
                    },
                    Poll::Ready(None) => {
                        // Channel closed, stream completed
                        *state = StreamState::Completed;
                        Poll::Ready(None)
                    },
                    Poll::Pending => {
                        // No data available yet
                        Poll::Pending
                    },
                }
            },
            StreamState::Completed => {
                // Stream already completed
                Poll::Ready(None)
            },
            StreamState::Failed(_e) => {
                // Stream failed, keep returning None
                Poll::Ready(None)
            },
        }
    }
}

// Note: Send and Sync are automatically implemented for ZigStream<T>
// because all its fields (Arc<Mutex<StreamState>>, PhantomData<T>) are
// Send/Sync

impl<T> Drop for ZigStream<T> {
    fn drop(&mut self) {
        // Ensure proper cleanup when stream is dropped
        let mut state = self.state.lock().unwrap();
        if let StreamState::Active { .. } = *state {
            *state = StreamState::Completed;
        }
    }
}

/// Helper function to create a stream callback pair
///
/// Returns a tuple of (sender, stream) where:
/// - `sender` can be used to push data into the stream
/// - `stream` is the `ZigStream` that can be consumed
///
/// # Examples
///
/// ```rust,ignore
/// use autozig::stream::create_stream;
///
/// let (tx, stream) = create_stream::<u32>();
///
/// // In Zig callback: tx.send(Ok(data))
/// // In Rust: stream.next().await
/// ```
pub fn create_stream<T>(
) -> (tokio::sync::mpsc::UnboundedSender<Result<Vec<u8>, String>>, ZigStream<T>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (tx, ZigStream::new(rx))
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;

    // Wrapper type for test purposes
    #[derive(Debug, PartialEq)]
    struct TestU32(u32);

    impl From<Vec<u8>> for TestU32 {
        fn from(bytes: Vec<u8>) -> Self {
            if bytes.len() >= 4 {
                TestU32(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            } else {
                TestU32(0)
            }
        }
    }

    #[tokio::test]
    async fn test_empty_stream() {
        let (_tx, stream) = create_stream::<TestU32>();
        futures::pin_mut!(stream);

        // Drop sender to close stream
        drop(_tx);

        // Stream should complete immediately
        assert!(stream.next().await.is_none());
        assert!(stream.is_completed());
    }

    #[tokio::test]
    async fn test_stream_with_data() {
        let (tx, stream) = create_stream::<TestU32>();
        futures::pin_mut!(stream);

        // Send some data
        let value1 = 42u32;
        tx.send(Ok(value1.to_le_bytes().to_vec())).unwrap();

        let value2 = 100u32;
        tx.send(Ok(value2.to_le_bytes().to_vec())).unwrap();

        // Close the stream
        drop(tx);

        // Receive first value
        let result1 = stream.next().await;
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().unwrap(), TestU32(42));

        // Receive second value
        let result2 = stream.next().await;
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().unwrap(), TestU32(100));

        // Stream should be completed
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_stream_with_error() {
        let (tx, stream) = create_stream::<TestU32>();
        futures::pin_mut!(stream);

        // Send some valid data first
        tx.send(Ok(42u32.to_le_bytes().to_vec())).unwrap();

        // Send an error
        tx.send(Err("Test error".to_string())).unwrap();

        // Close channel
        drop(tx);

        // Should receive the valid data first
        let result1 = stream.next().await;
        assert!(result1.is_some());
        assert!(result1.unwrap().is_ok());

        // Then receive the error
        let result2 = stream.next().await;
        assert!(result2.is_some());
        assert!(result2.unwrap().is_err());

        // Stream completes after error
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_stream_early_drop() {
        let (tx, stream) = create_stream::<TestU32>();

        // Send some data
        tx.send(Ok(vec![1, 2, 3, 4])).unwrap();

        // Drop stream before consuming - this closes the receiver
        drop(stream);

        // Sender will fail because receiver is dropped
        // This is expected behavior for unbounded channels
        assert!(tx.send(Ok(vec![5, 6, 7, 8])).is_err());
    }

    #[tokio::test]
    async fn test_multiple_consumers() {
        let (tx, stream) = create_stream::<TestU32>();

        // Clone the stream's Arc to simulate multiple references
        let stream_ref = Arc::new(tokio::sync::Mutex::new(stream));
        let stream_ref2 = stream_ref.clone();

        // Send data
        tx.send(Ok(42u32.to_le_bytes().to_vec())).unwrap();
        drop(tx);

        // Consume from first reference
        {
            let mut s = stream_ref.lock().await;
            let result = s.next().await;
            assert!(result.is_some());
            assert_eq!(result.unwrap().unwrap(), TestU32(42));
        }

        // Second reference should see stream completed
        {
            let mut s = stream_ref2.lock().await;
            assert!(s.next().await.is_none());
        }
    }
}
