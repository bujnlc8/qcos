# 一个简单的有道翻译插件

## 安装

*   将代码clone到`~/.vim/plugin`下面即可。

OR

*   如果你有安装插件管理工具，比如, `vim-plug`, 可以加入以下行到你的`.vimrc`进行安装

<!---->

    Plug 'linghaihui/vim-youdao-translator'

## 命令

*   `:YdInput`, 支持在底部输入框输入翻译，可以在`.vimrc`加入`noremap <leader>yi :<C-u>YdInput<CR>`支持快捷键输入。

*   `:YdCursor`, 支持翻译光标处单词，可以在`.vimrc`加入`noremap <leader>yc :<C-u>YdCursor<CR>`支持快捷键。

*   `:YdVisual`, 支持在visual模式下选中翻译，可以在`.vimrc`加入`vnoremap <leader>yv :<C-u>YdVisual<CR>`支持快捷键。

## 后记

在写插件的过程中借鉴了<https://github.com/ianva/vim-youdao-translater>
这个项目，特此表示感谢♥️。
但是在使用过程中经常有查询出错的情况，遂萌生了重写插件的念头, 本插件根据有道最新[官网](https://fanyi.youdao.com/)接口封装的，大大减小了出错的几率。在使用过程中如果出现问题，欢迎在Issues提出。


**注意⚠️,， 接口调用统一采用`system`的同步方式，虽然有更佳的`job_start`异步模式，但是经过我的测试，当翻译的词句长度较长时(大概达到换行的程度)，回调的结果会在状态栏一闪而过，根本看不清。google了半天也没找到解决办法，如果你有好的办法， 请在Issues中告诉我， 感激不尽。**
