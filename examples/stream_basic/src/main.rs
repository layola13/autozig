//! Stream Basic Example - Demonstrates Safe Streaming with Zig Integration
//!
//! This example shows AutoZig's streaming capabilities combined with Zig
//! utility functions, all without any unsafe code.

use autozig::{
    include_zig,
    stream::create_stream,
};
use futures::StreamExt;

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

// 引用 Zig 流式工具函数
include_zig!("src/stream.zig", {
    fn generate_sequence(start: u32, count: u32, output: *mut u32) -> ();
    fn sum_array(data: *const u32, len: usize) -> u64;
    fn generate_fibonacci(count: u32, output: *mut u64) -> ();
    fn filter_even(data: *mut u32, len: usize) -> usize;
    fn double_values(data: *mut u32, len: usize) -> ();
});

#[tokio::main]
async fn main() {
    println!("=== AutoZig Stream Basic Example ===\n");
    println!("Demonstrating:");
    println!("  ✓ Safe async streaming");
    println!("  ✓ Zig utility functions via include_zig!");
    println!("  ✓ Zero unsafe code");
    println!("  ✓ Type-safe data transfer\n");

    // Test 1: Simple stream with Zig-generated data
    println!("Test 1: Stream with Zig-Generated Sequence");
    test_zig_sequence_stream().await;
    println!();

    // Test 2: Stream with Fibonacci numbers from Zig
    println!("Test 2: Fibonacci Stream (Zig Generator)");
    test_fibonacci_stream().await;
    println!();

    // Test 3: Stream with error handling
    println!("Test 3: Stream with Error Handling");
    test_stream_with_error().await;
    println!();

    // Test 4: Stream with Zig processing
    println!("Test 4: Stream with Zig Processing");
    test_stream_with_zig_processing().await;
    println!();

    // Test 5: Concurrent streams
    println!("Test 5: Concurrent Streams");
    test_concurrent_streams().await;
    println!();

    println!("=== All stream tests passed! ===");
}

/// Test stream with Zig-generated sequence
async fn test_zig_sequence_stream() {
    let (tx, stream) = create_stream::<U32Value>();

    // Generate data using Zig
    let producer = tokio::spawn(async move {
        let count = 10u32;
        let mut buffer = vec![0u32; count as usize];

        // Call Zig to generate sequence
        generate_sequence(0, count, buffer.as_mut_ptr());

        // Stream the generated data
        for value in buffer {
            let bytes = value.to_le_bytes().to_vec();
            if tx.send(Ok(bytes)).is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    });

    // Consume the stream
    futures::pin_mut!(stream);
    let mut values = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(U32Value(value)) => {
                println!("  Received from Zig: {}", value);
                values.push(value);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    producer.await.unwrap();

    // Verify sequence
    assert_eq!(values.len(), 10);
    for (i, &val) in values.iter().enumerate() {
        assert_eq!(val, i as u32);
    }

    // Use Zig to compute sum
    let sum = sum_array(values.as_ptr(), values.len());
    println!("  ✓ Generated {} values, sum = {} (computed by Zig)", values.len(), sum);
}

/// Test fibonacci stream with Zig generator
async fn test_fibonacci_stream() {
    let (tx, stream) = create_stream::<U32Value>();

    let producer = tokio::spawn(async move {
        let count = 10u32;
        let mut fib_buffer = vec![0u64; count as usize];

        // Generate fibonacci using Zig
        generate_fibonacci(count, fib_buffer.as_mut_ptr());

        // Stream the fibonacci numbers
        for value in fib_buffer {
            let bytes = (value as u32).to_le_bytes().to_vec();
            if tx.send(Ok(bytes)).is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    });

    futures::pin_mut!(stream);
    let mut values = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(U32Value(value)) => {
                println!("  Fib[{}]: {}", values.len(), value);
                values.push(value);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    producer.await.unwrap();

    // Verify fibonacci sequence
    let expected = vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
    assert_eq!(values, expected);
    println!("  ✓ Fibonacci stream test passed ({} values)", values.len());
}

/// Test stream with error handling
async fn test_stream_with_error() {
    let (tx, stream) = create_stream::<U32Value>();

    let producer = tokio::spawn(async move {
        tx.send(Ok(42u32.to_le_bytes().to_vec())).unwrap();
        tx.send(Ok(100u32.to_le_bytes().to_vec())).unwrap();
        tx.send(Err("Simulated stream error".to_string())).unwrap();
        tx.send(Ok(200u32.to_le_bytes().to_vec())).unwrap();
    });

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
            },
        }
    }

    producer.await.unwrap();
    assert!(got_error);
    println!("  ✓ Error handling test passed (received {} values)", count);
}

/// Test stream with Zig data processing
async fn test_stream_with_zig_processing() {
    let (tx, stream) = create_stream::<U32Value>();

    let producer = tokio::spawn(async move {
        // Generate data
        let mut data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        println!("  Original data: {:?}", data);

        // Process with Zig: double all values
        double_values(data.as_mut_ptr(), data.len());
        println!("  After doubling (Zig): {:?}", data);

        // Filter even numbers using Zig
        let new_len = filter_even(data.as_mut_ptr(), data.len());
        data.truncate(new_len);
        println!("  After filtering even (Zig): {:?}", data);

        // Stream processed data
        for value in data {
            let bytes = value.to_le_bytes().to_vec();
            if tx.send(Ok(bytes)).is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    });

    futures::pin_mut!(stream);
    let mut values = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(U32Value(value)) => {
                values.push(value);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    producer.await.unwrap();

    // All values should be even (after doubling, all are even, then we kept only
    // even ones)
    for &val in &values {
        assert_eq!(val % 2, 0, "All values should be even");
    }

    println!("  ✓ Zig processing test passed ({} values, all even)", values.len());
}

/// Test concurrent streams
async fn test_concurrent_streams() {
    let streams: Vec<_> = (0..3)
        .map(|stream_id| {
            let (tx, stream) = create_stream::<U32Value>();

            tokio::spawn(async move {
                // Use Zig to generate sequence for each stream
                let count = 3u32;
                let mut buffer = vec![0u32; count as usize];
                generate_sequence(stream_id * 100, count, buffer.as_mut_ptr());

                for value in buffer {
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

    let mut total_values = 0;
    for handle in handles {
        let (stream_id, values) = handle.await.unwrap();
        assert_eq!(values.len(), 3, "Stream {} should have 3 values", stream_id);
        total_values += values.len();
    }

    assert_eq!(total_values, 9);
    println!("  ✓ Concurrent streams test passed (total {} values)", total_values);
}
