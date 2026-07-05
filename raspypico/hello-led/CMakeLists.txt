cmake_minimum_required(VERSION 3.13)
include($ENV{PICO_SDK_PATH}/external/pico_sdk_import.cmake)
project(blink C CXX ASM)
pico_sdk_init()

add_executable(blink main.c)
target_link_libraries(blink
    pico_stdlib
    pico_cyw43_arch_none      # ← これが今回の肝
)

pico_enable_stdio_usb(blink 1)
pico_enable_stdio_uart(blink 0)
pico_add_extra_outputs(blink)
