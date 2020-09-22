#include <signal.h>
#include <stdio.h>

#define Signal_Handler(Sig, Handler) { \
	struct sigaction __act = (struct sigaction) { \
		.sa_handler = Handler \
	}; \
	sigaction(Sig, &__act, 0); \
}

typedef void (*Action_msg) (char *msg, size_t len);
typedef void (*Action_cmd) (char cmd, int in, int out);

typedef struct {
    int             in, out, cmd;
    Action_msg      msgact;
    Action_cmd      cmdact;
} Console;

Console init_console(Action_msg action, Action_cmd cmdact);
void console_job(Console *console);
void close_console(Console *console);
