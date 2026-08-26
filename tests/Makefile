##
## EPITECH PROJECT, 2026
## epaper_dashboard
## File description:
## Host-side unit test build and runner (gcc + make from w64devkit)
##

CC ?= gcc
CFLAGS = -Wall -Wextra -Werror -std=gnu11 -Itest_config -I../include \
	-I../lib/jsmn -I../lib/font8x8

TESTS = test_harness

all: $(addsuffix .exe,$(TESTS))

test_harness.exe: test_harness.c
	$(CC) $(CFLAGS) -o $@ $^

run: all
	for t in $(TESTS); do ./$$t.exe || exit 1; done

clean:
	rm -f *.exe
