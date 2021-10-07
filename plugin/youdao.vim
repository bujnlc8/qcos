let s:current_path = expand('<sfile>:p:h')

let s:translator_file= s:current_path . '/translator.py'

if !exists('g:youdao_translator_cache')
  let g:youdao_translator_cache = 1
endif

if !exists('g:youdao_translator_cache_path')
  let g:youdao_translator_cache_path = s:current_path.'/.cache'
endif

if g:youdao_translator_cache
  if !isdirectory(g:youdao_translator_cache_path)
    call mkdir(g:youdao_translator_cache_path)
  endif
endif

let s:youdao_translator_enshrine_path = s:current_path.'/.enshrine.yde'

let s:youdao_translator_enshrine_comp_algo = ''

if executable('bzip2')
  let s:youdao_translator_enshrine_comp_algo = 'bzip2'
elseif executable('gzip')
  let s:youdao_translator_enshrine_comp_algo = 'gzip'
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

function! s:do_enshrine(words, translation)
  if len(s:youdao_translator_enshrine_comp_algo) > 0
    if filereadable(s:youdao_translator_enshrine_path)
      call system(s:youdao_translator_enshrine_comp_algo.' '.s:youdao_translator_enshrine_path.' -c -d  > '. s:current_path.'/.tmp.tmp')
      let l:need_write = 1
      for x in readfile(s:current_path.'/.tmp.tmp')
        if match(x, a:words.' ## ') != -1
          let l:need_write = 0
          break
        endif
      endfor
      if l:need_write
        call system('echo "'.a:words.' ## '.a:translation.'\n" >> '.s:current_path.'/.tmp.tmp && '.s:youdao_translator_enshrine_comp_algo.' -c  --best '.s:current_path.'/.tmp.tmp  > '.s:youdao_translator_enshrine_path.' && rm -rf '.s:current_path.'/.tmp.tmp')
      else
        call system('rm -rf '.s:current_path.'/.tmp.tmp')
      endif
    else
      call system('echo "'.a:words.' ## '.a:translation.'\n" | '.s:youdao_translator_enshrine_comp_algo.' -c  --best > '.s:youdao_translator_enshrine_path)
    endif
  else
    let l:need_write = 1
    if filereadable(s:youdao_translator_enshrine_path)
      for x in readfile(s:youdao_translator_enshrine_path)
        if match(x, a:words.' ## ') != -1
          let l:need_write = 0
          break
        endif
      endfor
    endif
    if l:need_write
      call system('echo "'.a:words.' ## '.a:translation.'\n" >> '.s:youdao_translator_enshrine_path)
    endif
  endif
endfunction

let s:channel_map = {}

function! s:translate(words, is_echo, do_enshrine)
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
          let l:res = readfile(l:path)[0]
          echo l:res
          if a:do_enshrine && len(l:res) > 0 && l:res != '无结果'
            call s:do_enshrine(a:words, l:res)
            echo a:words.' 收藏成功'
          endif
          return
        endif
      endif
    endif
  endif
  let l:cmd = 'python3 '.s:translator_file.' '.l:base64.' '.a:is_echo
  if !a:do_enshrine && exists('*job_start') && ! has('gui_macvim')
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
    if a:do_enshrine && len(l:res) > 0 && l:res != '无结果'
      call s:do_enshrine(a:words, l:res)
      echo a:words' 收藏成功'
    endif
  endif
endfunction

function! s:input_translate()
  let l:word = input('Enter the word: ')
  redraw!
  call s:translate(l:word, 1, 0)
endfunction

function! s:cursor_translate()
  call s:translate(expand('<cword>'), 1, 0)
endfunction

function! s:visual_translate()
  call s:translate(s:get_visual_select(), 0, 0)
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

function! s:enshrine_words(arg)
  if len(a:arg) == 0
    let l:word = expand('<cword>')
  else
    let l:word = a:arg
  endif
  if len(l:word) > 100
    echo '待收藏词太长'
    return
  endif
  call s:translate(l:word, 0, 1)
endfunction

function! s:enshrine_edit()
  execute 'tabnew! '.s:youdao_translator_enshrine_path
  if len(s:youdao_translator_enshrine_comp_algo) > 0
    execute '0,$ !'.s:youdao_translator_enshrine_comp_algo.' -d -c -q'
  endif
endfunction

function! s:after_write_enshrine_file()
  if len(s:youdao_translator_enshrine_comp_algo) > 0
    call system('cat '.s:youdao_translator_enshrine_path.' | '.s:youdao_translator_enshrine_comp_algo.' --best -c > '.s:youdao_translator_enshrine_path.'.1')
    call system('mv '.s:youdao_translator_enshrine_path.'.1 '.s:youdao_translator_enshrine_path)
  endif
  execute 'bd!'
endfunction

command! YdI call <SID>input_translate()
command! YdC call <SID>cursor_translate()
command! YdV call <SID>visual_translate()
command! -nargs=?   YdE call <SID>enshrine_words(<q-args>)
command! YdEE call <SID>enshrine_edit()
autocmd! BufWritePre *.yde set fileencoding=utf-8
autocmd! BufWritePost *.yde :call <SID>after_write_enshrine_file()
