" This is project specific vimrc
"v() {
" 	if [[ -f ".vimrc]]; then
" 		vim -S .vimrc $@
" 	else
" 		vim
" 	fi
" }

"some C specific stuff
inoremap <c-i>i #include <><left>
inoremap <c-i>I #include ""<left>

"files to open
e config.h
tabe Makefile
tabe dwm.c
tabe xi.c
