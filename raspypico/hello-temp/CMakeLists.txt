cmake_minimum_required(VERSION 3.13)
include($ENV{PICO_SDK_PATH}/external/pico_sdk_import.cmake)
project(hello_temp C CXX ASM)
pico_sdk_init()

add_executable(hello_temp main.c)
target_link_libraries(hello_temp
    pico_stdlib
    hardware_adc
)

pico_enable_stdio_usb(hello_temp 1)
pico_enable_stdio_uart(hello_temp 0)
pico_add_extra_outputs(hello_temp)
