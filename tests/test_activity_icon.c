/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Unit tests for the icon-class to embedded-icon mapping
*/

#include <assert.h>
#include <stdio.h>
#include "icon_data.h"
#include "activity_icon.h"

static void test_known_classes(void)
{
    assert(activity_icon_for("fa-check") == ICON_CHECK);
    assert(activity_icon_for("fa-envelope") == ICON_ENVELOPE);
    assert(activity_icon_for("fa-phone") == ICON_PHONE);
    assert(activity_icon_for("fa-users") == ICON_USERS);
    assert(activity_icon_for("fa-code") == ICON_CODE);
    assert(activity_icon_for("fa-cube") == ICON_CUBE);
    assert(activity_icon_for("fa-microchip") == ICON_MICROCHIP);
    assert(activity_icon_for("fa-lightbulb-o") == ICON_LIGHTBULB);
    assert(activity_icon_for("fa-calendar-check-o")
        == ICON_CALENDAR_CHECK);
    assert(activity_icon_for("fa-upload") == ICON_UPLOAD);
    assert(activity_icon_for("fa-pencil-square-o") == ICON_PEN_SQUARE);
}

static void test_unknown_falls_back(void)
{
    assert(activity_icon_for("fa-rocket") == ICON_CLOCK);
    assert(activity_icon_for("") == ICON_CLOCK);
    assert(activity_icon_for("fa-che") == ICON_CLOCK);
}

int main(void)
{
    test_known_classes();
    test_unknown_falls_back();
    printf("test_activity_icon: OK\n");
    return 0;
}
