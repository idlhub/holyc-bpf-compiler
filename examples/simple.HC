// Simple HolyC program to test compiler

U64 add(U64 a, U64 b) {
    return a + b;
}

U64 xor_deobfuscate(U64 value, U64 key) {
    return value ^ key;
}

U64 check_range(U64 value, U64 min, U64 max) {
    if (value < min) {
        return 0;
    }
    if (value > max) {
        return 0;
    }
    return 1;
}

U64 factorial(U64 n) {
    U64 result = 1;
    U64 i = 2;

    while (i <= n) {
        result = result * i;
        i = i + 1;
    }

    return result;
}
