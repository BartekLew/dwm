#include "pointerapp.h"

#include <string.h>
#include <X11/Xlib.h>

bool pointer_app (X11_pointer_app touch_action)
{
	Display	*dpy = XOpenDisplay (NULL);
	if (dpy == NULL)
		return false;

	Window root = DefaultRootWindow (dpy);
	XSelectInput( dpy, root, PointerMotionMask );
	XGrabPointer(dpy, root, False, PointerMotionMask, GrabModeAsync, 0, None, None, CurrentTime);

	while(1) {
		XEvent ev;
		XNextEvent(dpy, &ev);
	
		if (ev.type == MotionNotify) {
			touch_action ((Point){
				.x = ev.xmotion.x, .y = ev.xmotion.y
			} );
			printf( "%x\n", ev.xmotion.subwindow );
		}
	}
	return true;
}


bool set_timer(time_t interval, void(*handler)(int)) {
	#define SIGNO SIGRTMIN
	struct sigaction sa = { .sa_handler = handler };
	sigemptyset (&sa.sa_mask);

	timer_t timer;
	struct sigevent te = {
		.sigev_notify = SIGEV_SIGNAL,
		.sigev_signo = SIGNO,
		.sigev_value.sival_ptr = &timer
	};

	/* I don't know why this setting works. */
	struct itimerspec it = {
		.it_interval.tv_nsec = interval,
		.it_value.tv_nsec = interval
	};

	return	sigaction (SIGNO, &sa, NULL) == 0 &&
		timer_create (CLOCK_REALTIME, &te, &timer) == 0 &&
		timer_settime (timer, 0, &it, 0) == 0;
}
