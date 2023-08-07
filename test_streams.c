
#include <stdio.h>

typedef struct streams *Streams;
typedef struct win *Window;
typedef unsigned char u8;

Streams init_streams();
void new_stream(Streams s, const char* name);
int win2stream(Streams s, Window win, const char *name);
void key2stream(Streams s, Window win, u8 key);
void free_streams(Streams s);

int main() {
    Streams s = init_streams();
    new_stream(s, "foowin");

    Window w = (Window) 0xafa;
    Window w2 = (Window) 0xbfa;

    printf("%d\n", win2stream(s, w, "caca"));
    key2stream(s, w, 's');

    printf("%d\n", win2stream(s, w, "foowind"));
    key2stream(s, w, 'd');

    new_stream(s, "looloo");
    printf("%d\n", win2stream(s, w2, "loo"));
    printf("%d\n", win2stream(s, w2, "looloo"));

    key2stream(s, w, 'q');

    free_streams(s);
}

