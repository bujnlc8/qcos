let s:translator_file= expand('<sfile>:p:h') . '/translator.py'

function! TranslateCallback(chan, msg)
      echo a:msg
endfunction

function! s:translate(words, is_echo=0)
      if !a:is_echo
            let cmd = printf('python3 %s "%s"', s:translator_file, a:words)
      else
            let cmd = printf('python3 %s "%s" 1', s:translator_file, a:words)
      endif
      if exists('*jobstart')
            return jobstart(cmd, self)
      elseif exists('*job_start') && ! has('gui_macvim')
            return job_start(cmd, {'out_cb': 'TranslateCallback'})
      else
            echo system(cmd)
      endif
endfunction

function! s:input_translate()
      let l:word = input('Enter the word: ')
      redraw!
      call s:translate(substitute(l:word, '"', "'", 'g'), 1)
endfunction

function! s:cursor_translate()
      call s:translate(expand('<cword>'), 1)
endfunction

function! s:visual_translate()
      let l:visual_selected = substitute(s:get_visual_select(), '"', "'", 'g')
      call s:translate(l:visual_selected)
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
