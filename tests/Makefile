##
## EPITECH PROJECT, 2026
## epaper_dashboard
## File description:
## Host-side unit test build and runner (gcc + make from w64devkit)
##

CC ?= gcc
CFLAGS = -Wall -Wextra -Werror -std=gnu11 -Itest_config -I../include \
	-I../lib/jsmn -I../lib/font8x8

TESTS = test_harness test_json_str test_odoo_parse test_odoo_request \
	test_time_fmt test_http_util

all: $(addsuffix .exe,$(TESTS))

test_harness.exe: test_harness.c
	$(CC) $(CFLAGS) -o $@ $^

test_json_str.exe: test_json_str.c ../src/json_str.c
	$(CC) $(CFLAGS) -o $@ $^

test_odoo_parse.exe: test_odoo_parse.c ../src/odoo_parse.c \
	../src/jsmn_util.c ../src/json_str.c
	$(CC) $(CFLAGS) -o $@ $^

test_odoo_request.exe: test_odoo_request.c ../src/odoo_request.c
	$(CC) $(CFLAGS) -o $@ $^

test_time_fmt.exe: test_time_fmt.c ../src/time_fmt.c
	$(CC) $(CFLAGS) -o $@ $^

test_http_util.exe: test_http_util.c ../src/http_util.c
	$(CC) $(CFLAGS) -o $@ $^

run: all
	for t in $(TESTS); do ./$$t.exe || exit 1; done

clean:
	rm -f *.exe
