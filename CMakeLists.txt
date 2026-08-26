cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico2_w CACHE STRING "Board type")
include(pico_sdk_import.cmake)

project(epaper_dashboard C CXX ASM)
set(CMAKE_C_STANDARD 11)

if(NOT EXISTS ${CMAKE_CURRENT_LIST_DIR}/include/config.h)
    message(FATAL_ERROR
        "include/config.h missing: copy config.h.example there first")
endif()

pico_sdk_init()

add_executable(epaper_dashboard
    src/main.c
    src/epd_io.c
    src/epd_driver.c
    src/gfx.c
    src/gfx_text.c
    src/json_str.c
    src/jsmn_util.c
    src/odoo_parse.c
    src/odoo_request.c
    src/time_fmt.c
    src/http_util.c
    src/refresh.c
    src/dashboard.c
    src/net_wifi.c
    src/sys_idle.c
    src/net_time.c
    src/http_conn.c
    src/http_client.c
    src/odoo_client.c
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
    pico_cyw43_arch_lwip_poll
    hardware_watchdog
    pico_lwip_sntp
    pico_aon_timer
    pico_lwip_mbedtls
    pico_mbedtls
)

pico_enable_stdio_usb(epaper_dashboard 1)
pico_enable_stdio_uart(epaper_dashboard 0)
pico_add_extra_outputs(epaper_dashboard)
