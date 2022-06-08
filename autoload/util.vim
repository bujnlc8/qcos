function! util#base64(s)
    if has('python')
        let @b = a:s
python << EOF
import base64
import vim
res = base64.b64encode(vim.bindeval('@b'))
EOF
    elseif has('python3')
        let @b = a:s
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

function! util#md5(s)
    if executable('md5')
        return substitute(system('echo -n "'.a:s.'" | md5'), '\n', '', 'g')
    elseif has('python')
        let @b = a:s
python << EOF
import hashlib
import vim
res = hashlib.md5(str(vim.bindeval('@b')).encode('utf-8')).hexdigest()
EOF
    elseif has('python3')
        let @b = a:s
python3 << EOF
import hashlib
import vim
res = hashlib.md5(str(vim.bindeval('@b')).encode('utf-8')).hexdigest()
EOF
    endif
    if has('python')
        return pyeval('res')
    elseif has('python3')
        return py3eval('res')
    endif
endfunction
