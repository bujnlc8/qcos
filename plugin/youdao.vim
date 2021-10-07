let s:translator_file= expand('<sfile>:p:h') . '/translator.py'

function! TranslateCallback(chan, msg)
      echo a:msg
endfunction

function! s:base64(s)
    let @b = a:s
    if has('python')
python << EOF
import base64
import vim
res = base64.b64encode(vim.bindeval('@b'))
EOF
    elseif has('python3')
python3 << EOF
import base64
import vim
res = base64.b64encode(vim.bindeval('@b'))
EOF
    endif
    if has('python')
        return pyeval('res')
    elseif has('python3')
        return py3eval('res')
    endif
endfunction

function! s:translate(words, is_echo)
    if len(substitute(a:words, '\s', '', 'g')) == 0
        return
    endif
    "echom a:words
    let l:cmd = 'python3 '.s:translator_file.' '.s:base64(a:words).' '.a:is_echo
    if exists('*job_start') && ! has('gui_macvim')
        let l:job = job_start(l:cmd, {'out_cb': 'TranslateCallback', 'err_cb': 'TranslateCallback'})
        if job_status(l:job) != 'on'
          echo system(l:cmd)
        endif
    else
        echo system(l:cmd)
    endif
endfunction

function! s:input_translate()
      let l:word = input('Enter the word: ')
      redraw!
      call s:translate(l:word, 1)
endfunction

function! s:cursor_translate()
      call s:translate(expand('<cword>'), 1)
endfunction

function! s:visual_translate()
      call s:translate(s:get_visual_select(), 0)
endfunction

function! s:get_visual_select()
      try
            let l:a_save = @a
            normal! gv"ay
            return @a
      finally
            let @a = l:a_save
      endtry
endfunction


command! YdInput call <SID>input_translate()
command! YdCursor call <SID>cursor_translate()
command! YdVisual call <SID>visual_translate()
