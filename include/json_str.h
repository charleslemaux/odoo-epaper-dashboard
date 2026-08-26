/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** JSON string token extraction with UTF-8 and escape folding to ASCII
*/

#ifndef JSON_STR_H_
    #define JSON_STR_H_

    #include <stddef.h>
    #include <stdint.h>

struct json_out {
    char *dst;
    size_t size;
    size_t pos;
};

struct fold_entry {
    uint32_t cp;
    char const *out;
};

void json_str_fold(char *dst, size_t size, char const *src, size_t len);

#endif /* !JSON_STR_H_ */
