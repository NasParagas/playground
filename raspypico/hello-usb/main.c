#include <stdio.h>
#include "pico/stdlib.h"

int main() {
    stdio_init_all();
    while (true) {
        printf("Hello from Pico 2 W!\n");
        sleep_ms(1000);
    }
}
