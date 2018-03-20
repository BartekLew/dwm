" This is project specific vimrc
"v() {
" 	if [[ -f ".vimrc" ]]; then
" 		vim -S .vimrc $@
" 	else
" 		vim
" 	fi
" }

"some C specific stuff
inoremap <c-i>i #include <><left>
inoremap <c-i>I #include ""<left>
inoremap <c-i>n NULL

"files to open
e config.h
tabe Makefile
tabe xi.c
tabe xi.h
tabe shape_test.c
