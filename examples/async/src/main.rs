use autozig::include_zig;

// 异步FFI示例 - 使用spawn_blocking模式
// Zig侧实现同步函数，Rust侧提供异步包装
include_zig!("src/async_impl.zig", {
    // 模拟重计算任务（Zig侧同步实现）
    async fn heavy_computation(data: i32) -> i32;
    
    // 模拟I/O密集型任务（Zig侧同步实现）
    async fn process_data(input: &[u8]) -> usize;
    
    // 模拟数据库查询（Zig侧同步实现）
    async fn query_database(id: i32) -> i32;
});

#[tokio::main]
async fn main() {
    println!("=== AutoZig Async FFI Example (spawn_blocking) ===\n");

    // 测试1: 重计算任务
    println!("Test 1: Heavy Computation");
    let input = 42;
    println!("  Input: {}", input);
    
    let result = heavy_computation(input).await;
    println!("  Result: {}", result);
    assert_eq!(result, input * 2);
    println!("  ✓ Heavy computation passed\n");

    // 测试2: 数据处理
    println!("Test 2: Process Data");
    let data = vec![1u8, 2, 3, 4, 5];
    println!("  Input: {:?}", data);
    
    let sum = process_data(&data).await;
    println!("  Sum: {}", sum);
    assert_eq!(sum, 15);
    println!("  ✓ Data processing passed\n");

    // 测试3: 并发执行多个异步任务
    println!("Test 3: Concurrent Execution");
    let tasks = vec![
        tokio::spawn(async { heavy_computation(10).await }),
        tokio::spawn(async { heavy_computation(20).await }),
        tokio::spawn(async { heavy_computation(30).await }),
    ];
    
    let results: Vec<i32> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    println!("  Results: {:?}", results);
    assert_eq!(results, vec![20, 40, 60]);
    println!("  ✓ Concurrent execution passed\n");

    // 测试4: 数据库查询
    println!("Test 4: Database Query");
    let id = 123;
    println!("  Query ID: {}", id);
    
    let result = query_database(id).await;
    println!("  Result: {}", result);
    assert_eq!(result, id + 100);
    println!("  ✓ Database query passed\n");

    // 测试5: 混合使用异步和同步
    println!("Test 5: Mixed Async/Sync");
    let async_result = heavy_computation(5).await;
    let data = vec![async_result as u8; 5];
    let processed = process_data(&data).await;
    
    println!("  Async result: {}", async_result);
    println!("  Processed: {}", processed);
    assert_eq!(async_result, 10);
    assert_eq!(processed, 50);
    println!("  ✓ Mixed execution passed\n");

    println!("=== All async tests passed! ===");
    println!("\nArchitecture:");
    println!("  - Rust: Async wrappers using tokio::spawn_blocking");
    println!("  - Zig: Synchronous implementations (no async/await)");
    println!("  - Pattern: Thread pool offload for FFI blocking calls");
}