# dwm - dynamic window manager
# See LICENSE file for copyright and license details.

include config.mk

all: options bin/dwm bin/touch_test bin/shape_test


options:
	@echo dwm build options:
	@echo "CFLAGS   = ${CFLAGS}"
	@echo "LDFLAGS  = ${LDFLAGS}"
	@echo "CC       = ${CC}"

bin/shape_test: o/xi.o o/shape_test.o
	${CC} ${CFLAGS} $^ -o $@ -lX11 -lXi -lrt

bin/touch_test: o/xi.o o/touch_test.o
	${CC} ${CFLAGS} $^ -o $@ -lX11 -lXi -lrt

bin/dwm: o/dwm.o o/drw.o o/util.o ${OBJ}
	@echo CC -o $@
	@${CC} -o $@ $^ ${LDFLAGS}

bin/:
	mkdir -p bin

o/%.o: %.c 
	${CC} ${CFLAGS} -c -o $@ $<

o/:
	mkdir -p o

${OBJ}: config.h config.mk

config.h:
	@echo creating $@ from config.def.h
	@cp config.def.h $@

clean:
	@echo cleaning
	@rm -fr bin o *.tar.gz

dist: clean
	@echo creating dist tarball
	@mkdir -p dwm-${VERSION}
	@cp -R LICENSE TODO BUGS Makefile README config.def.h config.mk \
		dwm.1 drw.h util.h ${SRC} dwm.png transient.c dwm-${VERSION}
	@tar -cf dwm-${VERSION}.tar dwm-${VERSION}
	@gzip dwm-${VERSION}.tar
	@rm -rf dwm-${VERSION}

install: all
	@echo installing executable file to ${DESTDIR}${PREFIX}/bin
	@mkdir -p ${DESTDIR}${PREFIX}/bin
	@cp -f dwm ${DESTDIR}${PREFIX}/bin
	@chmod 755 ${DESTDIR}${PREFIX}/bin/dwm
	@echo installing manual page to ${DESTDIR}${MANPREFIX}/man1
	@mkdir -p ${DESTDIR}${MANPREFIX}/man1
	@sed "s/VERSION/${VERSION}/g" < dwm.1 > ${DESTDIR}${MANPREFIX}/man1/dwm.1
	@chmod 644 ${DESTDIR}${MANPREFIX}/man1/dwm.1

uninstall:
	@echo removing executable file from ${DESTDIR}${PREFIX}/bin
	@rm -f ${DESTDIR}${PREFIX}/bin/dwm
	@echo removing manual page from ${DESTDIR}${MANPREFIX}/man1
	@rm -f ${DESTDIR}${MANPREFIX}/man1/dwm.1

.PHONY: all options clean dist install uninstall
