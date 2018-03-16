/* 
	shape_test: tring to dump shapes instead of touches
*/

#include "xi.h"
#include <stdio.h>
#include <math.h>


float direction (Point a, Point b) {
	float	h = b.y - a.y,
		w = b.x - a.x,
		diag = sqrt (w*w + h*h);

	return	h / diag; /* NaN = no direction*/
}

uintmax_t	now, last_time;
Point		last;
bool		touching;

/*
	cycle is XInput refresh time. You can check it using
	touch_test from this package. On my machine I get 50
	events each second so 20ms is cycle time.
	BUT: in practice it it turned out that if some other
	events occur, Xserver would need more time and i get
	more timer cycles in between. The effect is that i get
	"hit" when I move cursor too. 0.1 sec seems to be safe
	value here.
*/
#define Msec 1e6
#define Cycle 100 * Msec

static void each_cycle (int signo) {
	if (last_time != now++ && touching) {
		touching = false;
		printf (": %4d, %4d %2lld.%1llds\n\n", last.x, last.y, now/10, now%10);
	}
}

static void trace_pointer (Point pos) {
	last_time = now;

	if (touching) { 
		printf ("%.2f\t",
			direction (pos, last)
		);
	}
	else {
		now = last_time = 0;
		touching = true;
	}
	last = pos;
}

int main (int argc, char **argv)
{
	if (!set_timer( Cycle, &each_cycle )) {

		fprintf (stderr, "Unable to register timer.");
		return 1;
	}

	if (!xi2_app (&trace_pointer) ) {
		fprintf (stderr, "Unable to start XInput app.");
		return 1;
	}

	return 0;
}

