#ifndef __HAVE_DWM_RS_H
#define __HAVE_DWM_RS_H 1

typedef struct Client Client;

// Header for src/dwm.rs
typedef unsigned char u8;
typedef unsigned short u16;
typedef char i8;
typedef unsigned long int u64;
typedef int i32;
typedef const char *CStr;

typedef struct {
    CStr buff;
    size_t len;
} CLenStr;

void set_term_title(CLenStr title);

extern void showhide(Client *c);

#endif
