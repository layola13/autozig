use std::sync::Arc;

use autozig::stream::{
    create_stream,
    ZigStream,
};
use futures::StreamExt;
use tokio::sync::Mutex;

/// Wrapper type for u32 that implements From<Vec<u8>>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct U32Value(u32);

impl From<Vec<u8>> for U32Value {
    fn from(bytes: Vec<u8>) -> Self {
        if bytes.len() >= 4 {
            U32Value(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            U32Value(0)
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== AutoZig Stream Basic Example ===\n");

    // Test 1: Simple stream with a few values
    println!("Test 1: Simple Stream");
    test_simple_stream().await;
    println!();

    // Test 2: Stream with error handling
    println!("Test 2: Stream with Error");
    test_stream_with_error().await;
    println!();

    // Test 3: Concurrent stream consumption
    println!("Test 3: Concurrent Streams");
    test_concurrent_streams().await;
    println!();

    println!("=== All stream tests passed! ===");
}

async fn test_simple_stream() {
    let (tx, stream) = create_stream::<U32Value>();

    // Spawn a task to produce data
    let producer = tokio::spawn(async move {
        for i in 1u32..=5u32 {
            let value: u32 = i * 10;
            let bytes = value.to_le_bytes().to_vec();
            if tx.send(Ok(bytes)).is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        // Drop tx to close the stream
    });

    // Consume the stream
    futures::pin_mut!(stream);
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(U32Value(value)) => {
                println!("  Received: {}", value);
                count += 1;
                assert_eq!(value, count * 10);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    producer.await.unwrap();
    assert_eq!(count, 5);
    println!("  ✓ Simple stream test passed (received {} items)", count);
}

async fn test_stream_with_error() {
    let (tx, stream) = create_stream::<U32Value>();

    // Spawn a task to produce data with an error
    let producer = tokio::spawn(async move {
        // Send a few values
        tx.send(Ok(42u32.to_le_bytes().to_vec())).unwrap();
        tx.send(Ok(100u32.to_le_bytes().to_vec())).unwrap();

        // Send an error
        tx.send(Err("Simulated error".to_string())).unwrap();

        // Try to send more (should still work, stream decides when to stop)
        tx.send(Ok(200u32.to_le_bytes().to_vec())).unwrap();
    });

    // Consume the stream
    futures::pin_mut!(stream);
    let mut count = 0;
    let mut got_error = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(U32Value(value)) => {
                println!("  Received: {}", value);
                count += 1;
            },
            Err(e) => {
                println!("  Received error: {}", e);
                got_error = true;
                assert_eq!(e, "Simulated error");
            },
        }
    }

    producer.await.unwrap();
    assert!(got_error, "Should have received an error");
    println!("  ✓ Error handling test passed (got error, received {} values)", count);
}

async fn test_concurrent_streams() {
    // Create multiple streams
    let streams: Vec<_> = (0..3)
        .map(|stream_id| {
            let (tx, stream) = create_stream::<U32Value>();

            // Spawn producer for this stream
            tokio::spawn(async move {
                for i in 1u32..=3u32 {
                    let value: u32 = (stream_id as u32 * 100) + i;
                    let bytes = value.to_le_bytes().to_vec();
                    if tx.send(Ok(bytes)).is_err() {
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                }
            });

            stream
        })
        .collect();

    // Consume all streams concurrently
    let handles: Vec<_> = streams
        .into_iter()
        .enumerate()
        .map(|(stream_id, stream)| {
            tokio::spawn(async move {
                futures::pin_mut!(stream);
                let mut values = Vec::new();

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(U32Value(value)) => {
                            values.push(value);
                            println!("  Stream {}: Received {}", stream_id, value);
                        },
                        Err(e) => {
                            eprintln!("  Stream {}: Error: {}", stream_id, e);
                        },
                    }
                }

                (stream_id, values)
            })
        })
        .collect();

    // Wait for all consumers
    let mut total_values = 0;
    for handle in handles {
        let (stream_id, values) = handle.await.unwrap();
        assert_eq!(values.len(), 3, "Stream {} should have 3 values", stream_id);
        total_values += values.len();
    }

    assert_eq!(total_values, 9);
    println!("  ✓ Concurrent streams test passed (total {} values)", total_values);
}
