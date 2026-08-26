cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico2_w CACHE STRING "Board type")
include(pico_sdk_import.cmake)

project(epaper_dashboard C CXX ASM)
set(CMAKE_C_STANDARD 11)

pico_sdk_init()

add_executable(epaper_dashboard
    src/main.c
)

target_include_directories(epaper_dashboard PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}
    ${CMAKE_CURRENT_LIST_DIR}/include
    ${CMAKE_CURRENT_LIST_DIR}/lib/jsmn
    ${CMAKE_CURRENT_LIST_DIR}/lib/font8x8
)

target_compile_options(epaper_dashboard PRIVATE -Wall -Wextra)

target_link_libraries(epaper_dashboard
    pico_stdlib
)

pico_enable_stdio_usb(epaper_dashboard 1)
pico_enable_stdio_uart(epaper_dashboard 0)
pico_add_extra_outputs(epaper_dashboard)
