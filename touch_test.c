/* 
	touch_test: dump positions + times of touches.

	this may be usefull if your screen needs adjustments
*/

#include "xi.h"
#include <stdio.h>
#include <time.h>

static void trace_pointer (Point pos) {
	struct {
		time_t 		time;
		uintmax_t	cnt;
	} static ctx = {0};

	time_t t = time(NULL);
	if (t > ctx.time) {
		printf ("rate : %llu ev/s\n", ctx.cnt);
		ctx.time = t;
		ctx.cnt = 1;
	} else
		ctx.cnt++;

	printf ("%d,%d, %lu\n", pos.x, pos.y, time(NULL));
}

int main (int argc, char **argv)
{
	if (!xi2_app (&trace_pointer) ) {
		fprintf (stderr, "Unable to start XInput app.");
		return 1;
	}

	return 0;
}

