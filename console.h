#include <signal.h>
#include <stdio.h>
#include <stdarg.h>

#define Signal_Handler(Sig, Handler) { \
	struct sigaction __act = (struct sigaction) { \
		.sa_handler = Handler \
	}; \
	sigaction(Sig, &__act, 0); \
}

typedef void (*Action) (char *msg, size_t len);

typedef const char *Cstr;

typedef struct console Console;

Console* init_console(Action action, Action cmdact);
void console_job(Console *console);
void close_console(Console *console);
void console_log(Console *console, Cstr fmt, ...);
