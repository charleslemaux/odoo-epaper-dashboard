/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Mapping from Odoo activity icon classes to embedded icons
*/

#ifndef ACTIVITY_ICON_H_
    #define ACTIVITY_ICON_H_

struct icon_rule {
    char const *needle;
    int icon;
};

int activity_icon_for(char const *icon);

#endif /* !ACTIVITY_ICON_H_ */
