# 一个有用的翻译VIM插件, 提供有道和百度两种选择

## 安装

*   将代码clone到`~/.vim/plugin`下面即可。

OR

*   如果你有安装插件管理工具，比如, `vim-plug`, 可以加入以下行到你的`.vimrc`进行安装

<!---->

    Plug 'linghaihui/vim-translator'

## 命令

*   `:Ti`, 支持在底部输入框输入翻译，可以在`.vimrc`加入`noremap <leader>ti :<C-u>Ti<CR>`支持快捷键输入。

*   `:Tc`, 支持翻译光标处单词，可以在`.vimrc`加入`noremap <leader>tc :<C-u>Tc<CR>`支持快捷键。

*   `:Tv`, 支持在visual模式下选中翻译，可以在`.vimrc`加入`vnoremap <leader>tv :<C-u>Tv<CR>`支持快捷键。

*   `:Te`, 收藏单词或语句，如果提供了一个参数，那么会收藏该参数，否则收藏visual模式下选择的区域，如果不是visual模式则选择光标处的单词。

*   `:Tee`, 编辑收藏的单词或语句，可以像编辑一个文本来进行编辑。

## 选项

*  `let g:translator_cache=1`, 是否启用缓存，默认1。

*  `let g:translator_cache_path='~/.cache'`，缓存路径，默认`expand('<sfile>:p:h').'/.cache'`。 

*  `let g:translator_channel='youdao'`，查询通道，默认`youdao`, 也可切到`baidu`。


## 后记

在写插件的过程中借鉴了[vim-youdao-translater](https://github.com/ianva/vim-youdao-translater) 这个项目，特此表示感谢♥️。

但在使用上述插件的过程中经常出现查询出错的情况，而且使用的接口也已经比较老了，遂萌生了重写插件的念头, 本插件根据有道翻译[官网](https://fanyi.youdao.com/)最新接口封装，大大减小了出错的几率, 相当智能。写完之后觉得百度翻译好像也还可以，于是又加上了百度翻译。可以通过`let g:translator_channel='baidu'`来切换到百度翻译。

在使用过程中如果出现问题，欢迎在Issues提出。

另外，为了推广python3, 本插件只支持python3。

**注意⚠️， 插件在visual模式翻译的时候会默认输出原句，因为在`job_start`异步模式经过我的测试，如果未在翻译前输出一些字符，当翻译的词句长度较长时(大概达到换行的程度)，回调的结果会在状态栏一闪而过，根本看不清, 后来偶然试出这个略带trick的方法。如果你有好的办法， 请在Issues中告诉我， 感激不尽。**

如果百度翻译提示token失效或者类似的错误，请尝试用chrome访问[百度翻译](https://fanyi.baidu.com/)，打开开发者工具，随意输入一个单词查询，在`https://fanyi.baidu.com/v2transapi?xxx`的form表单提交中应该有一个`token`字段，在Cookies中应该有一个名字为`BAIDUID_BFESS`的cookie, 将它们的值分别替换掉`plugin/baidu.py`文件顶部的`TOKEN`和`BAIDUID_BFESS`变量。 替换之后再次尝试。
