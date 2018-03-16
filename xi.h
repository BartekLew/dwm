#ifndef __HEADER_XI
#define __HEADER_XI

#include <stdbool.h>
#include <stdint.h>

typedef struct {
	int x,y;
} Point;
typedef void (*X11_pointer_app) (Point p);

bool xi2_app (X11_pointer_app touch_action);

#endif
