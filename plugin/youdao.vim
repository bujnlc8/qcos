let s:translator_file= expand('<sfile>:p:h') . '/translator.py'

if !exists('g:youdao_translator_cache')
  let g:youdao_translator_cache = 1
endif

if !exists('g:youdao_translator_cache_path')
  let g:youdao_translator_cache_path = expand('<sfile>:p:h').'/.cache'
endif

if g:youdao_translator_cache
  if !isdirectory(g:youdao_translator_cache_path)
    call mkdir(g:youdao_translator_cache_path)
  endif
endif

function! s:do_cache(md5, s)
  let l:ppdir = g:youdao_translator_cache_path.'/'.a:md5[:1]
  if !isdirectory(l:ppdir)
    call mkdir(l:ppdir)
  endif
  let l:pdir = l:ppdir.'/'.a:md5[2:3]
  if !isdirectory(l:pdir)
    call mkdir(l:pdir)
  endif
  call writefile([a:s], l:pdir.'/'.a:md5)
endfunction

function! TranslateCallback(chan, msg)
  echom a:msg
  if g:youdao_translator_cache
    let l:channel_id = matchstr(string(a:chan), '[0-9]\+')
    if has_key(s:channel_map, l:channel_id)
      call s:do_cache(s:channel_map[l:channel_id], a:msg)
      unlet s:channel_map[l:channel_id]
    endif
  endif
endfunction


let s:channel_map = {}

function! s:translate(words, is_echo)
  if len(substitute(a:words, '\s', '', 'g')) == 0
    return
  endif
  let l:base64 = util#base64(substitute(a:words, '\r', '', 'g'))
  let l:md5 = ''
  if g:youdao_translator_cache
    let l:md5 = util#md5(l:base64)
    let l:ppdir = g:youdao_translator_cache_path.'/'.l:md5[:1]
    if isdirectory(l:ppdir)
      let l:pdir = l:ppdir.'/'.l:md5[2:3]
      if isdirectory(l:pdir)
        let l:path = l:pdir.'/'.l:md5
        if filereadable(l:path)
          echo readfile(l:path)[0]
          return
        endif
      endif
    endif
  endif
  let l:cmd = 'python3 '.s:translator_file.' '.l:base64.' '.a:is_echo
  if exists('*job_start') && ! has('gui_macvim')
    let l:job = job_start(l:cmd, {'out_cb': 'TranslateCallback', 'err_cb': 'TranslateCallback', 'mode': 'raw'})
    if g:youdao_translator_cache
      let l:channel_id = matchstr(string(job_getchannel(l:job)), '[0-9]\+')
      let s:channel_map[l:channel_id] = l:md5
    endif
  else
    let l:res = system(l:cmd)
    echo l:res
    if g:youdao_translator_cache
      call s:do_cache(l:md5, l:res)
    endif
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
    if len(@a) > 0
      echo @a."\n"
    endif
    return @a
  finally
    let @a = l:a_save
  endtry
endfunction


command! YdI call <SID>input_translate()
command! YdC call <SID>cursor_translate()
command! YdV call <SID>visual_translate()
