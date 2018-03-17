/* 
	shape_test: tring to dump shapes instead of touches
*/

#include "xi.h"
#include <stdio.h>
#include <math.h>

typedef struct {
	float direction, distance;
} Movement;

Movement movement (Point a, Point b) {
	float	h = b.y - a.y,
		w = b.x - a.x,
		diag = sqrt (w*w + h*h);

	return	(Movement) {
		.distance = diag,
		.direction = (w < 0)	? -h / diag
					: h /diag
	};
}

uintmax_t	now, last_time;
Point		last;
Movement	total_mov;
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
		printf (": %.2f %.2f %2lld.%1llds\n\n", 
			total_mov.distance,
			total_mov.direction / total_mov.distance,
			 now/10, now%10);
	}
}

static void trace_pointer (Point pos) {
	last_time = now;

	if (touching) { 
		Movement m = movement (pos, last);
		total_mov = (Movement) {
			.distance = total_mov.distance + m.distance,
			.direction = (isnan(m.direction))
				? total_mov.direction
				: total_mov.direction +
					m.distance * m.direction
		};
	}
	else {
		now = last_time = 0;
		total_mov = (Movement){.distance=0};
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

