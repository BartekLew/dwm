#ifndef __HAVE_STREAMS_H
#define __HAVE_STREAMS_H

// This is C header for stream.rs

#include "dwm-rs.h"
#include <X11/Xlib.h>

typedef struct streams *Streams;

Streams init_streams(Display *dpy);
void new_stream(Streams s, CStr name);
void end_stream(Streams s, Window win);
CLenStr win2stream(Streams s, Window win, CStr name);
void key2stream(Streams s, XKeyEvent *ev, u16 key);
void free_streams(Streams s);

#endif
