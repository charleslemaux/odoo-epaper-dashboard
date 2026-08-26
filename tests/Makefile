##
## Charles Le Maux, 2026
## epaper_dashboard
## File description:
## Host-side unit test build and runner (gcc + make from w64devkit)
##

CC ?= gcc
CFLAGS = -Wall -Wextra -Werror -std=gnu11 -Itest_config -I../include \
	-I../include/app -I../include/epd -I../include/gfx -I../include/json \
	-I../include/net -I../include/odoo -I../lib/jsmn -I../lib/font8x8

TESTS = test_harness test_json_str test_odoo_parse test_odoo_request \
	test_odoo_request_todos test_time_fmt test_http_util test_refresh \
	test_gfx test_gfx_text test_dashboard

all: $(addsuffix .exe,$(TESTS))

test_harness.exe: test_harness.c
	$(CC) $(CFLAGS) -o $@ $^

test_json_str.exe: test_json_str.c ../src/json/json_str.c
	$(CC) $(CFLAGS) -o $@ $^

test_odoo_parse.exe: test_odoo_parse.c ../src/odoo/odoo_parse.c \
	../src/json/jsmn_util.c ../src/json/json_str.c
	$(CC) $(CFLAGS) -o $@ $^

test_odoo_request.exe: test_odoo_request.c ../src/odoo/odoo_request.c
	$(CC) $(CFLAGS) -o $@ $^

test_odoo_request_todos.exe: test_odoo_request_todos.c \
	../src/odoo/odoo_request.c
	$(CC) $(CFLAGS) -DODOO_INCLUDE_UNDATED_TODOS=1 -o $@ $^

test_time_fmt.exe: test_time_fmt.c ../src/app/time_fmt.c
	$(CC) $(CFLAGS) -o $@ $^

test_http_util.exe: test_http_util.c ../src/net/http_util.c
	$(CC) $(CFLAGS) -o $@ $^

test_refresh.exe: test_refresh.c ../src/app/refresh.c
	$(CC) $(CFLAGS) -o $@ $^

test_gfx.exe: test_gfx.c ../src/gfx/gfx.c
	$(CC) $(CFLAGS) -o $@ $^

test_gfx_text.exe: test_gfx_text.c ../src/gfx/gfx_text.c ../src/gfx/gfx.c
	$(CC) $(CFLAGS) -o $@ $^

test_dashboard.exe: test_dashboard.c ../src/app/dashboard.c \
	../src/gfx/gfx.c ../src/gfx/gfx_text.c ../src/app/time_fmt.c
	$(CC) $(CFLAGS) -o $@ $^

run: all
	for t in $(TESTS); do ./$$t.exe || exit 1; done

clean:
	rm -f *.exe
