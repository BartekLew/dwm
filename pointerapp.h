#ifndef __HEADER_POINTERAPP
#define __HEADER_POINTERAPP

#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include <time.h>
#include <signal.h>


typedef struct {
	int x,y;
} Point;
typedef void (*X11_pointer_app) (Point p);

bool pointer_app (X11_pointer_app touch_action);
bool set_timer(time_t interval, void(*handler)(int));

#endif
