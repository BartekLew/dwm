#include "console.h"

#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <poll.h>

static const char *inpath = "/tmp/dwm.in";

Console init_console(Action_msg act) {
    mkfifo(inpath, 0660);

    return (Console) {
        .in = open (inpath, O_RDWR, 0),
        .action = act
    };
}

void close_console (Console *data) {
    close (data->in);
    unlink(inpath);
}

void console_job (Console *data) {
    struct pollfd polldat = (struct pollfd){
        .fd = data->in, .events = POLLIN
    };

    if (poll(&polldat, 1, 0) > 0) {
  	    if (polldat.revents & POLLIN) {	
            char line[81];
            int len = read (data->in, line, 81) - 1;

            data->action(line, len);
	    }
    }
}

