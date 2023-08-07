
#include <stdio.h>

typedef struct stream *Stream;
typedef struct win *Window;
typedef unsigned char u8;

Stream init_stream(const char* win_name);
int win2stream(Stream s, Window win, const char *name);
void key2stream(Stream s, Window win, u8 key);
void free_streams(Stream s);

int main() {
    Stream s = init_stream("foowin");
    Window w = (Window) 0xafa;

    printf("%d\n", win2stream(s, w, "caca"));
    key2stream(s, w, 's');

    printf("%d\n", win2stream(s, w, "foowind"));
    key2stream(s, w, 'd');

    free_streams(s);
}

