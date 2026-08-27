/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Mapping from Odoo activity type names to embedded icons
*/

#include <ctype.h>
#include <string.h>
#include "icon_data.h"
#include "activity_icon.h"

static const struct icon_rule RULES[] = {
    {"meeting", ICON_MEETING}, {"reunion", ICON_MEETING},
    {"rendez", ICON_MEETING}, {"inbox", ICON_INBOX},
    {"certif", ICON_CERTIFICATE}, {"signature", ICON_SIGNATURE},
    {"sign", ICON_SIGNATURE}, {"document", ICON_DOCUMENT},
    {"upload", ICON_DOCUMENT}, {"telecharge", ICON_DOCUMENT},
    {"mail", ICON_MAIL}, {"courriel", ICON_MAIL},
    {"call", ICON_CALL}, {"appel", ICON_CALL}, {"phone", ICON_CALL},
    {"to-do", ICON_TODO}, {"to do", ICON_TODO}, {"todo", ICON_TODO},
    {"faire", ICON_TODO}, {0, 0}
};

static int match(char const *hay, char const *needle)
{
    size_t nlen = strlen(needle);
    size_t j = 0;

    for (size_t i = 0; hay[i] != '\0'; i++) {
        j = 0;
        while (j < nlen && hay[i + j] != '\0'
            && tolower((unsigned char)hay[i + j])
            == tolower((unsigned char)needle[j]))
            j++;
        if (j == nlen)
            return 1;
    }
    return 0;
}

int activity_icon_for(char const *kind)
{
    for (size_t i = 0; RULES[i].needle != 0; i++) {
        if (match(kind, RULES[i].needle))
            return RULES[i].icon;
    }
    return ICON_CLOCK;
}
