/* 
	shape_test: tring to dump shapes instead of touches
*/

#include "xi.h"
#include <stdio.h>
#include <stdint.h>
#include <math.h>

bool print_point (Point p) {
	return printf ("%4d,%4d", p.x, p.y) > 0;
}

bool nl (void) {
	return printf ("\n") > 0;
}

typedef uint_fast16_t Cycle;
typedef uint_fast16_t uint;

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

typedef bool (*ShapeTest) (Movement total, Movement absolute);
typedef bool (*ShapeAction) (Point p);
typedef struct {
	ShapeTest	cond;
	ShapeAction	act;
} Shape;

#define Dot_threshold 10.
#define Min_circle_len 80.
#define Max_circle_off 50.
#define Max_circle_direction 0.1

static bool point_test (Movement total, Movement abs) {
	return total.distance < Dot_threshold;
}

static bool point_act (Point p) {
	return printf ("Point at: ") > 0 &&
		print_point (p) && nl ();
}

static bool circle_test (Movement total, Movement abs) {
	return fabsf (total.direction) <= Max_circle_direction &&
		 total.distance >= Min_circle_len &&
		 abs.distance <= Max_circle_off;
}

static bool circle_action (Point p) {
	return printf ("Circle at: ") &&
		print_point (p) && nl ();
}

Shape shapes[] = {
	{.cond = point_test, .act = point_act},
	{.cond = circle_test, .act = circle_action}
};
#define Shapes_cnt (sizeof(shapes)/sizeof(Shape))

bool do_for_shape (Movement total, Movement abs) {
	uint i;
	for (i = 0; i < Shapes_cnt; i++) {
		if (shapes[i].cond (total, abs)) {
			shapes[i].act (total.start);
			break;
		}
	}
	return i < Shapes_cnt;
}

Cycle		now;
Movement	total;
bool		touching;

/*
	Except of handling X events we need additional timer
	to handle a moment when a touch is released because
	the most accurate interface - XInput inform us only
	about touched points. We need to adjust the interval
	for this timer. It must be longer than interval between
	XInput events, otherwise it would interpret all points
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
	if (total.until != now++ && touching) {
		touching = false;
		Movement absolute = movement (total.start, total.end);
		total.direction /= total.distance;

		if (!do_for_shape (total, absolute)) {
			printf (": %.2f %.2f %2d.%1ds\n", 
				total.distance,
				total.direction,
				now/10, now%10);
			printf ("%4d,%4d -> %4d,%4d, %.2f %.2f\n\n",
				total.start.x, total.start.y,
				total.end.x, total.end.y,
				absolute.distance, absolute.direction
			);
		}
		
	}
}

static void trace_pointer (Point pos) {
	total.until = now;

	if (touching) { 
		Movement m = movement (total.end,pos);
		total.distance = total.distance + m.distance;
		if (isnan(total.direction))
			total.direction = m.direction * m.distance;
		else if (!isnan(m.direction))
			total.direction = total.direction + m.distance * m.direction;
	}
	else {
		now = 0;
		total = (Movement){.start=pos};
		touching = true;
	}
		
	total.end = pos;
}

int main (int argc, char **argv)
{
	if (!set_timer( Cycle_interval, &each_cycle )) {

		fprintf (stderr, "Unable to register timer.");
		return 1;
	}

	if (!xi2_app (&trace_pointer) ) {
		fprintf (stderr, "Unable to start XInput app.");
		return 1;
	}

	return 0;
}

