/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Mapping from Odoo activity icon classes to embedded icons
*/

#include <string.h>
#include "icon_data.h"
#include "activity_icon.h"

static const struct icon_rule RULES[] = {
    {"fa-check", ICON_CHECK},
    {"fa-envelope", ICON_ENVELOPE},
    {"fa-phone", ICON_PHONE},
    {"fa-users", ICON_USERS},
    {"fa-inbox", ICON_INBOX},
    {"fa-code", ICON_CODE},
    {"fa-cube", ICON_CUBE},
    {"fa-microchip", ICON_MICROCHIP},
    {"fa-lightbulb-o", ICON_LIGHTBULB},
    {"fa-calendar-check-o", ICON_CALENDAR_CHECK},
    {"fa-calendar", ICON_CALENDAR_CHECK},
    {"fa-upload", ICON_UPLOAD},
    {"fa-pencil-square-o", ICON_PEN_SQUARE},
    {"fa-pencil", ICON_PEN_SQUARE},
    {"fa-tasks", ICON_CHECK},
    {"fa-clock-o", ICON_CLOCK},
    {0, 0}
};

int activity_icon_for(char const *icon)
{
    for (size_t i = 0; RULES[i].needle != 0; i++) {
        if (strcmp(icon, RULES[i].needle) == 0)
            return RULES[i].icon;
    }
    return ICON_CLOCK;
}
