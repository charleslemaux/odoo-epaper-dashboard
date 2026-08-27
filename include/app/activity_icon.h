/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Mapping from Odoo activity type names to embedded icons
*/

#ifndef ACTIVITY_ICON_H_
    #define ACTIVITY_ICON_H_

struct icon_rule {
    char const *needle;
    int icon;
};

int activity_icon_for(char const *kind);

#endif /* !ACTIVITY_ICON_H_ */
