/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Watchdog-fed idle wait that keeps the wifi driver polled
*/

#ifndef SYS_IDLE_H_
    #define SYS_IDLE_H_

    #include <stdint.h>

void sys_idle_ms(uint32_t ms);

#endif /* !SYS_IDLE_H_ */
