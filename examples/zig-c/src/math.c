// math.c - Simple C math library
#include <stdint.h>

// 基础算术运算
int32_t c_add(int32_t a, int32_t b) {
    return a + b;
}

int32_t c_multiply(int32_t a, int32_t b) {
    return a * b;
}

// 数组求和
int32_t c_sum_array(const int32_t* arr, uint32_t len) {
    int32_t sum = 0;
    for (uint32_t i = 0; i < len; i++) {
        sum += arr[i];
    }
    return sum;
}

// 字符串长度计算（不使用标准库）
uint32_t c_string_length(const char* str) {
    uint32_t len = 0;
    while (str[len] != '\0') {
        len++;
    }
    return len;
}