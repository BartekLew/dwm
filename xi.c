#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>
#include <X11/Xlib.h>
#include <X11/extensions/XInput2.h>

typedef void (*X11_app) (Display *dpy, int xi_opcode);
typedef struct {
	int x,y;
} Point;

static bool xi2_app (X11_app entry)
{
	Display	*dpy = XOpenDisplay (NULL);
	if (dpy == NULL)
		return false;

	int minor = 2, major=2, xi_opcode, event, error;

	/* test if XInput supported at all.
	   on error second call puts supported version in given variables. use it
	   if you have lower version. This might be enough. */
	if (!XQueryExtension(dpy, "XInputExtension", &xi_opcode, &event, &error) ||
		XIQueryVersion(dpy, &major, &minor) == BadRequest)
		return false;

	entry (dpy, xi_opcode);
	return true;
}

static void selelct_motion_events(Display *dpy, Window win)
{
	#if XI_LASTEVENT > 64
	#error XI events count > 64, adjust mask in code below.
	#endif

	uint64_t mot_events = 0;
	XISetMask((uint8_t*)&mot_events, XI_RawMotion);

	XISelectEvents (dpy, win, (XIEventMask[]) { {
		.deviceid = XIAllMasterDevices,
		.mask_len = sizeof(mot_events),
		.mask = (uint8_t*)&mot_events
	} }, 1 );
	
	XFlush(dpy);
}

void trace_pointer (Display *dpy, int xi_opcode) {
	Window root = DefaultRootWindow (dpy);
	selelct_motion_events(dpy, root);

	while(1) {
		XEvent ev;
		XGenericEventCookie *cookie = &ev.xcookie;
	
		XNextEvent(dpy, &ev);
	
		if (cookie->type != GenericEvent ||
		    cookie->extension != xi_opcode ||
		    !XGetEventData(dpy, cookie))
		    continue;
	
		if (cookie->evtype == XI_RawMotion) {
			Window		ptr_root, ptr_win;
			Point		root_rel, win_rel;
			uint32_t	mask;
			XQueryPointer(dpy, root, &ptr_root, &ptr_win,
				&(root_rel.x), &(root_rel.y),
				&(win_rel.x), &(win_rel.y), &mask
			);
		    	printf ("%d,%d\n", root_rel.x, root_rel.y);
		}
		XFreeEventData(dpy, cookie);
	}
}

int main (int argc, char **argv)
{
	if (!xi2_app (&trace_pointer) ) {
		fprintf (stderr, "Unable to start XInput app.");
		return 1;
	}

	return 0;
}

