let s:current_path = expand('<sfile>:p:h')

if !exists('g:translator_channel')
    let g:translator_channel = 'youdao'
endif

if g:translator_channel != 'youdao' &&  g:translator_channel != 'baidu'
    echo 'g:translator_channel 配置错误'
endif

let s:translator_file= s:current_path . '/'.g:translator_channel.'.py'

if !exists('g:translator_cache')
    let g:translator_cache = 1
endif

if !exists('g:translator_cache_path')
    let g:translator_cache_path = s:current_path.'/.cache'
endif

if g:translator_cache
    if !isdirectory(g:translator_cache_path)
        call mkdir(g:translator_cache_path)
    endif
endif

let s:translator_enshrine_path = s:current_path.'/.enshrine.tdata'

let s:translator_enshrine_comp_algo = ''

if executable('bzip2')
    let s:translator_enshrine_comp_algo = 'bzip2'
elseif executable('gzip')
    let s:translator_enshrine_comp_algo = 'gzip'
endif

function! s:do_cache(md5, s)
    let l:ppdir = g:translator_cache_path.'/'.a:md5[:1]
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
    if g:translator_cache
        let l:channel_id = matchstr(string(a:chan), '[0-9]\+')
        if has_key(s:channel_map, l:channel_id)
            call s:do_cache(s:channel_map[l:channel_id], a:msg)
            unlet s:channel_map[l:channel_id]
        endif
    endif
endfunction

function! s:do_enshrine(words, translation)
    if len(s:translator_enshrine_comp_algo) > 0
        if filereadable(s:translator_enshrine_path)
            call system(s:translator_enshrine_comp_algo.' '.s:translator_enshrine_path.' -c -d  > '. s:current_path.'/.tmp.tmp')
            let l:need_write = 1
            for x in readfile(s:current_path.'/.tmp.tmp')
                if match(x, a:words."\u0001") != -1
                    let l:need_write = 0
                    break
                endif
            endfor
            if l:need_write
                call system('echo "'.a:words.'\u0001 '.a:translation.'  ['.g:translator_channel.']\n" >> '.s:current_path.'/.tmp.tmp && '.s:translator_enshrine_comp_algo.' -c  --best '.s:current_path.'/.tmp.tmp  > '.s:translator_enshrine_path.' && rm -rf '.s:current_path.'/.tmp.tmp')
            else
                call system('rm -rf '.s:current_path.'/.tmp.tmp')
            endif
        else
            call system('echo "'.a:words.'\u0001 '.a:translation.'  ['.g:translator_channel.']\n" | '.s:translator_enshrine_comp_algo.' -c  --best > '.s:translator_enshrine_path)
        endif
    else
        let l:need_write = 1
        if filereadable(s:translator_enshrine_path)
            for x in readfile(s:translator_enshrine_path)
                if match(x, a:words."\u0001") != -1
                    let l:need_write = 0
                    break
                endif
            endfor
        endif
        if l:need_write
            call system('echo "'.a:words.'\u0001 '.a:translation.'  ['.g:translator_channel.']\n" >> '.s:translator_enshrine_path)
        endif
    endif
endfunction

let s:channel_map = {}

function! s:translate(words, is_echo, do_enshrine, is_replace)
    if len(substitute(a:words, '\s', '', 'g')) == 0
        echo '输入为空'
        return
    endif
    let l:base64 = util#base64(a:words)
    let l:md5 = ''
    if g:translator_cache
        let l:md5 = util#md5(l:base64)
        let l:ppdir = g:translator_cache_path.'/'.l:md5[:1]
        if isdirectory(l:ppdir)
            let l:pdir = l:ppdir.'/'.l:md5[2:3]
            if isdirectory(l:pdir)
                let l:path = l:pdir.'/'.l:md5
                if filereadable(l:path)
                    let l:res = readfile(l:path)[0]
                    if !a:is_replace
                        echo l:res
                    endif
                    if a:do_enshrine && len(l:res) > 0 && match(l:res, 'Err:') == -1
                        call s:do_enshrine(a:words, l:res)
                        echo a:words.' 收藏成功'
                    endif
                    return l:res
                endif
            endif
        endif
    endif
    let l:cmd = 'python3 '.s:translator_file.' '.l:base64.' '.a:is_echo
    if !a:is_replace && !a:do_enshrine && exists('*job_start') && ! has('gui_macvim')
        let l:job = job_start(l:cmd, {'out_cb': 'TranslateCallback', 'err_cb': 'TranslateCallback', 'mode': 'raw'})
        if g:translator_cache
            let l:channel_id = matchstr(string(job_getchannel(l:job)), '[0-9]\+')
            let s:channel_map[l:channel_id] = l:md5
        endif
    else
        let l:res = system(l:cmd)
        if !a:is_replace
            echo l:res
        endif
        if g:translator_cache
            call s:do_cache(l:md5, l:res)
        endif
        if a:do_enshrine && len(l:res) > 0 && match(l:res, 'Err:') ==-1
            call s:do_enshrine(a:words, l:res)
            echo a:words.'收藏成功'
        endif
        return l:res
    endif
endfunction

function! s:input_translate()
    let l:word = input('Enter the word: ')
    redraw!
    call s:translate(l:word, 1, 0, 0)
endfunction

function! s:cursor_translate()
    call s:translate(expand('<cword>'), 1, 0, 0)
endfunction

function! s:visual_translate()
    call s:translate(s:get_visual_select(1), 0, 0, 0)
endfunction

function! s:get_visual_select(is_echo)
    try
        let l:a_save = @a
        silent! normal! gv"ay
        if len(@a) > 0 && a:is_echo
            echo @a."\n"
        endif
        return @a
    finally
        let @a = l:a_save
    endtry
endfunction

function! s:_enshrine_words(arg,...)
    if len(a:arg) == 0
        let l:word = expand('<cword>')
    else
        let l:word = a:arg
    endif
    if len(a:000) > 0
        let l:word = s:get_visual_select(1)
    endif
    if len(l:word) > 500
        echo '待收藏词太长'
        return
    endif
    call s:translate(l:word, 0, 1, 0)
endfunction

function! s:enshrine_words(arg)
    call s:_enshrine_words(a:arg)
endfunction

function! s:enshrine_wordsv()
    call s:_enshrine_words('', 1)
endfunction

function! s:enshrine_edit()
    if !filereadable(s:translator_enshrine_path)
        echo '收藏文件找不到:('
        return
    endif
    execute 'silent! tabnew! '.s:translator_enshrine_path
    if len(s:translator_enshrine_comp_algo) > 0
        execute '0,$ !'.s:translator_enshrine_comp_algo.' -d -c -q'
    endif
endfunction

function! s:after_write_enshrine_file()
    if len(s:translator_enshrine_comp_algo) > 0
        call system('cat '.s:translator_enshrine_path.' | '.s:translator_enshrine_comp_algo.' --best -c > '.s:translator_enshrine_path.'.1')
        call system('mv '.s:translator_enshrine_path.'.1 '.s:translator_enshrine_path)
    endif
    execute 'bd!'
endfunction

function! s:replace_translate()
    let l:text = s:get_visual_select(0)
    let reg_tmp = @a
    let @a = s:translate(l:text, 0, 0, 1)
    silent! normal! gv"ap
    let @a = reg_tmp
endfunction

command! Ti call <SID>input_translate()
command! Tc call <SID>cursor_translate()
command! -range Tv call <SID>visual_translate()
command! -range Tr call <SID>replace_translate()
command! -nargs=? Te call <SID>enshrine_words(<q-args>)
command! Tee call <SID>enshrine_edit()
command! Tev call <SID>enshrine_wordsv()
autocmd! BufWritePre *.tdata set fileencoding=utf-8
autocmd! BufWritePost *.tdata :call <SID>after_write_enshrine_file()
autocmd! BufWinEnter *.tdata match  Conceal /[\u0001]/
