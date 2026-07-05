#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"

int main() {
    stdio_init_all();

    if (cyw43_arch_init()) {        // cyw43チップを起こす（失敗しうるのでチェック）
        printf("cyw43 init failed\n");
        return -1;
    }
    printf("cyw43 init ok, blinking...\n");

    // 点滅
    while (true) {
        cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, 1);
        sleep_ms(250);
        cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, 0);
        sleep_ms(250);
    }
}
