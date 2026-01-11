// AutoZig WASM64 加载器
// 针对 WASM64 (Memory64) 优化，支持 BigInt 指针

/**
 * 加载并初始化 AutoZig WASM64 模块
 * @param {string} wasmUrl - WASM 文件的 URL
 * @returns {Promise<WebAssembly.Instance>} WASM 实例
 */
async function loadAutoZigWasm(wasmUrl) {
    console.log('🚀 Loading AutoZig WASM64 module...');
    
    // 1. 初始化 WASM64 内存
    // 注意：index: 'i64' 声明这是 64 位寻址内存
    const memory = new WebAssembly.Memory({
        initial: 10,      // 初始页数 (10 * 64KB = 640KB)
        maximum: 100,     // 最大页数 (100 * 64KB = 6.4MB)
        index: 'i64'      // 🔑 关键：声明这是 64 位寻址内存
    });
    
    console.log('✅ WASM64 Memory created (64-bit addressing)');

    // 2. 定义环境 Imports
    const imports = {
        env: {
            memory: memory,
            
            // 实现 Zig 定义的 extern "env" fn js_log
            // WASM64 传出来的指针是 BigInt 类型
            js_log: (ptrBigInt, lenBigInt) => {
                try {
                    // WASM64: 指针和长度都是 BigInt，需要转换为 Number
                    const ptr = Number(ptrBigInt);
                    const len = Number(lenBigInt);
                    
                    // 直接从共享内存读取 (Zero-Copy)
                    const bytes = new Uint8Array(memory.buffer, ptr, len);
                    const text = new TextDecoder("utf-8").decode(bytes);
                    
                    console.log(`[AutoZig] ${text}`);
                } catch (e) {
                    console.error('[AutoZig] js_log error:', e);
                }
            },

            // 实现 Zig 定义的 extern "env" fn js_error
            js_error: (ptrBigInt, lenBigInt) => {
                try {
                    const ptr = Number(ptrBigInt);
                    const len = Number(lenBigInt);
                    
                    const bytes = new Uint8Array(memory.buffer, ptr, len);
                    const text = new TextDecoder("utf-8").decode(bytes);
                    
                    console.error(`[AutoZig Error] ${text}`);
                } catch (e) {
                    console.error('[AutoZig] js_error error:', e);
                }
            }
        }
    };

    try {
        // 3. 实例化 WASM 模块
        console.log(`📦 Fetching WASM from: ${wasmUrl}`);
        
        const { instance } = await WebAssembly.instantiateStreaming(
            fetch(wasmUrl), 
            imports
        );
        
        console.log('✅ WASM module instantiated');
        
        // 4. 调用初始化函数（如果存在）
        if (typeof instance.exports.init === 'function') {
            console.log('🔧 Calling init()...');
            instance.exports.init();
        }
        
        console.log('✅ AutoZig WASM64 module ready!');
        return instance;
        
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        throw error;
    }
}

/**
 * 辅助函数：从 WASM 内存读取字符串
 * @param {WebAssembly.Memory} memory - WASM 内存对象
 * @param {bigint|number} ptr - 字符串指针
 * @param {bigint|number} len - 字符串长度
 * @returns {string} 解码后的字符串
 */
function readString(memory, ptr, len) {
    const ptrNum = typeof ptr === 'bigint' ? Number(ptr) : ptr;
    const lenNum = typeof len === 'bigint' ? Number(len) : len;
    
    const bytes = new Uint8Array(memory.buffer, ptrNum, lenNum);
    return new TextDecoder("utf-8").decode(bytes);
}

/**
 * 辅助函数：将字符串写入 WASM 内存
 * @param {WebAssembly.Memory} memory - WASM 内存对象
 * @param {number} ptr - 目标指针
 * @param {string} str - 要写入的字符串
 * @returns {number} 实际写入的字节数
 */
function writeString(memory, ptr, str) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    const view = new Uint8Array(memory.buffer, ptr, bytes.length);
    view.set(bytes);
    return bytes.length;
}

// 导出函数供外部使用
export { loadAutoZigWasm, readString, writeString };