#include "console.h"

#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <poll.h>

static const char *inpath = "/tmp/dwm.in";
static const char *cmdpath = "/tmp/dwm.cmd";
static const char *outpath = "/tmp/dwm.out";

Console init_console(Action_msg msgact, Action_cmd cmdact) {
    mkfifo(inpath, 0660);
    mkfifo(outpath, 0660);
    mkfifo(cmdpath, 0660);

    return (Console) {
        .in = open (inpath, O_RDWR, 0),
        .out = open (outpath, O_RDWR, 0),
        .cmd = open (cmdpath, O_RDWR, 0),
        .msgact = msgact,
        .cmdact = cmdact
    };
}

void close_console (Console *data) {
    close (data->in);
    unlink(inpath);
    close (data->out);
    unlink(outpath);
    close (data->cmd);
    unlink(cmdpath);
}

void console_job (Console *data) {
    struct pollfd polldat[] = {
        {.fd = data->in, .events = POLLIN},
        {.fd = data->cmd, .events = POLLIN}
    };

    if (poll(polldat, 2, 0) > 0) {
  	    if (polldat[0].revents & POLLIN) {	
            char line[81];
            int len = read (data->in, line, 81) - 1;

            data->msgact(line, len);
	    }
        if (polldat[1].revents & POLLIN) {
            char cmd;
            read (data->cmd, &cmd, 1);
            
            data->cmdact(cmd, data->cmd, data->out);
        }
    }
}

