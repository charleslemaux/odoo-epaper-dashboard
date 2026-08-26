# == DO NOT EDIT THE FOLLOWING LINES for the Raspberry Pi Pico VS Code Extension to work ==
if(WIN32)
    set(USERHOME $ENV{USERPROFILE})
else()
    set(USERHOME $ENV{HOME})
endif()
set(sdkVersion 2.3.0)
set(toolchainVersion 15_2_Rel1)
set(picotoolVersion 2.3.0)
set(picoVscode ${USERHOME}/.pico-sdk/cmake/pico-vscode.cmake)
if (EXISTS ${picoVscode})
    include(${picoVscode})
endif()
# ====================================================================================
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
    src/app/main.c
    src/app/dashboard.c
    src/app/refresh.c
    src/app/time_fmt.c
    src/app/panic_log.c
    src/app/sys_idle.c
    src/epd/epd_io.c
    src/epd/epd_driver.c
    src/gfx/gfx.c
    src/gfx/gfx_text.c
    src/gfx/font_data.c
    src/json/json_str.c
    src/json/jsmn_util.c
    src/net/net_wifi.c
    src/net/net_time.c
    src/net/http_util.c
    src/net/http_conn.c
    src/net/http_client.c
    src/odoo/odoo_request.c
    src/odoo/odoo_parse.c
    src/odoo/odoo_client.c
)

target_compile_definitions(epaper_dashboard PRIVATE
    PICO_PANIC_FUNCTION=panic_log_store
    PICO_PANIC_FUNCTION_DOES_NOT_RETURN=1
)

target_include_directories(epaper_dashboard PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}
    ${CMAKE_CURRENT_LIST_DIR}/include
    ${CMAKE_CURRENT_LIST_DIR}/include/app
    ${CMAKE_CURRENT_LIST_DIR}/include/epd
    ${CMAKE_CURRENT_LIST_DIR}/include/gfx
    ${CMAKE_CURRENT_LIST_DIR}/include/json
    ${CMAKE_CURRENT_LIST_DIR}/include/net
    ${CMAKE_CURRENT_LIST_DIR}/include/odoo
    ${CMAKE_CURRENT_LIST_DIR}/lib/jsmn
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
