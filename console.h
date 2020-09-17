#include <signal.h>

#define Signal_Handler(Sig, Handler) { \
	struct sigaction __act = (struct sigaction) { \
		.sa_handler = Handler \
	}; \
	sigaction(Sig, &__act, 0); \
}

typedef void (*Action_msg) (char *msg, size_t len);

typedef struct {
    int             in;
    Action_msg      action;
} Console;

Console init_console(Action_msg action);
void console_job(Console *console);
void close_console(Console *console);
