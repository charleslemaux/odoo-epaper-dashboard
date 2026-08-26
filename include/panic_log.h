/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Panic message capture surviving reboot for headless diagnostics
*/

#ifndef PANIC_LOG_H_
    #define PANIC_LOG_H_

#define PANIC_LOG_MAGIC 0x50414e43u
#define PANIC_LOG_CAP 160

struct panic_log {
    unsigned int magic;
    char message[PANIC_LOG_CAP];
};

void panic_log_store(char const *fmt, ...) __attribute__((__noreturn__));
void panic_log_report(void);

#endif /* !PANIC_LOG_H_ */
