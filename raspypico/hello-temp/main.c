#include <stdio.h>
#include "pico/stdlib.h"
#include "hardware/adc.h"

int main() {
    stdio_init_all();
    adc_init();
    adc_set_temp_sensor_enabled(true);
    adc_select_input(4);                  // 4番 = 内蔵温度センサ

    const float conv = 3.3f / (1 << 12);  // 12bit, 3.3V基準
    while (true) {
        float volt  = adc_read() * conv;
        float tempC = 27.0f - (volt - 0.706f) / 0.001721f;
        printf("temp = %.2f C\n", tempC);
        sleep_ms(1000);
    }
}
