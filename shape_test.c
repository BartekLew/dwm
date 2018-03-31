/* 
	shape_test: tring to dump shapes instead of touches
*/

#include "pointerapp.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

typedef uint_fast16_t Cycle;
typedef struct {
	Point	start, end;
	float	direction, distance;
	Cycle	until;
} Movement;

Movement movement (Point a, Point b) {
	float	h = b.y - a.y,
		w = b.x - a.x,
		diag = sqrt (w*w + h*h);

	return	(Movement) {
		.distance = diag,
		.direction = h / diag
	};
}

Cycle		now;
Movement	mov;
bool		touching;

/*
	Except of handling X events we need additional timer
	to handle a moment when a touch is released because
	the most accurate interface - XServer inform us only
	about touched points. We need to adjust the interval
	for this timer. It must be longer than interval between
	events, otherwise it would interpret all points
	as separate touches. This interval may be cheched
	using touch_test tool in this repo. On my platform
	(RPi3) I get ~45-50 events per second so it must be
	greater than 0.02s. At the same time if it's too high
	two separate touches could be merged into one. For
	me 0.1s interval works well.
*/
#define Msec 1e6
#define Cycle_interval 100 * Msec

static void each_cycle (int signo) {
	if (mov.until != now++ && touching) {
		touching = false;
		printf (": %.2f %.2f %2d.%1ds\n", 
			mov.distance,
			mov.direction / mov.distance,
			now/10, now%10);

		Movement total = movement (mov.start, mov.end);
		printf ("%4d,%4d -> %4d,%4d, %.2f %.2f\n\n",
			mov.start.x, mov.start.y,
			mov.end.x, mov.end.y,
			total.distance, total.direction
		);
		
		if( now > 50 ) exit(0);
	}
}

static void trace_pointer (Point pos) {
	mov.until = now;

	if (touching) { 
		Movement m = movement (mov.end,pos);
		mov.distance = mov.distance + m.distance;
		if (isnan(mov.direction))
			mov.direction = m.direction * m.distance;
		else if (!isnan(m.direction))
			mov.direction = mov.direction + m.distance * m.direction;
	}
	else {
		now = 0;
		mov = (Movement){.start=pos};
		touching = true;
	}
		
	mov.end = pos;
}

int main (int argc, char **argv)
{
	if (!set_timer( Cycle_interval, &each_cycle )) {

		fprintf (stderr, "Unable to register timer.");
		return 1;
	}

	if (!pointer_app (&trace_pointer) ) {
		fprintf (stderr, "Unable to start XInput app.");
		return 1;
	}

	return 0;
}

