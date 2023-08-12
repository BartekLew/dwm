#ifndef __HAVE_CONSOLE_H
#define __HAVE_CONSOLE_H 1

#include <signal.h>
#include <stdio.h>
#include <stdarg.h>
#include "stream.h"

#define Signal_Handler(Sig, Handler) { \
	struct sigaction __act = (struct sigaction) { \
		.sa_handler = Handler \
	}; \
	sigaction(Sig, &__act, 0); \
}

typedef const char *Cstr;

typedef struct console Console;

Console* init_console(Streams streams);
void console_job(Console *console);
void close_console(Console *console);
void console_log_del(Console *console, const char* name, Window id);
void console_log_upd(Console *console, const char* name, Window id);
#endif
