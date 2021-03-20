#include "math.h"

__attribute__ ((dllexport))
int add(int a, int b) {
    return a + b;
}

__attribute__ ((dllexport))
int sub(int a, int b) {
    return a - b;
}

__attribute__ ((dllexport))
int mul(int a, int b) {
    return a * b;
}

__attribute__ ((dllexport))
int div(int a, int b) {
    return a / b;
}