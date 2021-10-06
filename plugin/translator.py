# coding=utf-8
from __future__ import unicode_literals

import ctypes
import json
import os
import random
import re
import time
from sys import argv, stdout, version_info

try:
    import requests
except ModuleNotFoundError:
    os.system(
        'pip3 install requests -i https://mirrors.tuna.tsinghua.edu.cn/pypi/web/simple/')
    import requests


def unsigned_right_shitf(n, i):
    """无符号右移动"""
    if n < 0:
        n = ctypes.c_uint32(n).value
    if i < 0:
        return -int_overflow(n << abs(i))
    return int_overflow(n >> i)


def int_overflow(val):
    maxint = 2147483647
    if not -maxint-1 <= val <= maxint:
        val = (val + (maxint + 1)) % (2 * (maxint + 1)) - maxint - 1
    return val


def n(e, t):
    return ctypes.c_int(e << t ^ 0).value | unsigned_right_shitf(e, 32 - t)


def r(e, t):
    i = ctypes.c_int(2147483648 & e ^ 0).value
    o = ctypes.c_int(2147483648 & t ^ 0).value
    n = ctypes.c_int(1073741824 & e ^ 0).value
    r = ctypes.c_int(1073741824 & t ^ 0).value
    a = ctypes.c_int(((1073741823 & e) + (1073741823 & t)) ^ 0).value
    if n & r:
        return ctypes.c_int((2147483648 ^ a ^ i ^ o) ^ 0).value
    elif n | r:
        if 1073741824 & a:
            return ctypes.c_int((3221225472 ^ a ^ i ^ o) ^ 0).value
        return ctypes.c_int((1073741824 ^ a ^ i ^ o) ^ 0).value
    return ctypes.c_int((a ^ i ^ o) ^ 0).value


def i(e, t, n):
    return ctypes.c_int((e & t | ~e & n) ^ 0).value


def o(e, t, n):
    return ctypes.c_int((e & n | t & ~n) ^ 0).value


def a(e, t, n):
    return ctypes.c_int((e ^ t ^ n) ^ 0).value


def s(e, t, n):
    return ctypes.c_int((t ^ (e | ~n)) ^ 0).value


def l(e, t, o, a, s, l, c):
    e = r(e, r(r(i(t, o, a), s), c))
    return r(n(e, l), t)


def c(e, t, i, a, s, l, c):
    e = r(e, r(r(o(t, i, a), s), c))
    return r(n(e, l), t)


def u(e, t, i, o, s, l, c):
    e = r(e, r(r(a(t, i, o), s), c))
    return r(n(e, l), t)


def d(e, t, i, o, a, l, c):
    e = r(e, r(r(s(t, i, o), a), c))
    return r(n(e, l), t)


def f(e):
    t = len(e)
    n = len(e)
    r = n + 8
    i = 16 * ((r - r % 64) // 64 + 1)
    o = [0] * i
    a = 0
    s = 0
    while s < n:
        a = s % 4 * 8
        t = (s - s % 4) // 4
        o[t] = o[t] | ctypes.c_int(ord(e[s]) << a ^ 0).value
        s += 1
    t = (s - s % 4) // 4
    a = s % 4 * 8
    o[t] = ctypes.c_int((o[t] | 128 << a) ^ 0).value
    o[i-2] = ctypes.c_int(n << 3 ^ 0).value
    o[i-1] = unsigned_right_shitf(n, 29)
    return o


def p(e):
    r, n = '', ''
    for t in range(4):
        r = '0' + str(hex(unsigned_right_shitf(e, 8 * t) & 255))[2:]
        i = len(r) - 2
        n += r[i:i+2]
    return n


def h(e):
    e = re.sub(r'/\x0d\x0a/g', '\n', e)
    t = ''
    n = 0
    for n in range(len(e)):
        r = ord(e[n])
        if r < 128:
            t += chr(r)
        elif r > 127 and r < 2048:
            t += chr(r >> 6 | 192)
            t += chr(63 & r | 128)
        elif r >= 55296 and r <= 56319:
            if n + 1 < len(e):
                i = ord(e[n+1])
                if i >= 56320 and i <= 57343:
                    o = 1024 * (r - 55296) + (i - 56320) + 65536
                    t += chr(240 | o >> 18 & 7)
                    t += chr(128 | o >> 12 & 63)
                    t += chr(128 | o >> 6 & 63)
                    t += chr(128 | 63 & o)
        else:
            t += chr(r >> 12 | 224)
            t += chr(r >> 6 & 63 | 128)
            t += chr(63 & r | 128)
    return t


def md5(e):
    e = h(e)
    y = f(e)
    s = 1732584193
    m = 4023233417
    g = 2562383102
    v = 271733878
    t = 0
    while t < len(y):
        n = s
        i = m
        o = g
        a = v
        s = l(s, m, g, v, y[t + 0], 7, 3614090360)
        v = l(v, s, m, g, y[t + 1], 12, 3905402710)
        g = l(g, v, s, m, y[t + 2], 17, 606105819)
        m = l(m, g, v, s, y[t + 3], 22, 3250441966)
        s = l(s, m, g, v, y[t + 4], 7, 4118548399)
        v = l(v, s, m, g, y[t + 5], 12, 1200080426)
        g = l(g, v, s, m, y[t + 6], 17, 2821735955)
        m = l(m, g, v, s, y[t + 7], 22, 4249261313)
        s = l(s, m, g, v, y[t + 8], 7, 1770035416)
        v = l(v, s, m, g, y[t + 9], 12, 2336552879)
        g = l(g, v, s, m, y[t + 10], 17, 4294925233)
        m = l(m, g, v, s, y[t + 11], 22, 2304563134)
        s = l(s, m, g, v, y[t + 12], 7, 1804603682)
        v = l(v, s, m, g, y[t + 13], 12, 4254626195)
        g = l(g, v, s, m, y[t + 14], 17, 2792965006)
        m = l(m, g, v, s, y[t + 15], 22, 1236535329)
        s = c(s, m, g, v, y[t + 1], 5, 4129170786)
        v = c(v, s, m, g, y[t + 6], 9, 3225465664)
        g = c(g, v, s, m, y[t + 11], 14, 643717713)
        m = c(m, g, v, s, y[t + 0], 20, 3921069994)
        s = c(s, m, g, v, y[t + 5], 5, 3593408605)
        v = c(v, s, m, g, y[t + 10], 9, 38016083)
        g = c(g, v, s, m, y[t + 15], 14, 3634488961)
        m = c(m, g, v, s, y[t + 4], 20, 3889429448)
        s = c(s, m, g, v, y[t + 9], 5, 568446438)
        v = c(v, s, m, g, y[t + 14], 9, 3275163606)
        g = c(g, v, s, m, y[t + 3], 14, 4107603335)
        m = c(m, g, v, s, y[t + 8], 20, 1163531501)
        s = c(s, m, g, v, y[t + 13], 5, 2850285829)
        v = c(v, s, m, g, y[t + 2], 9, 4243563512)
        g = c(g, v, s, m, y[t + 7], 14, 1735328473)
        m = c(m, g, v, s, y[t + 12], 20, 2368359562)
        s = u(s, m, g, v, y[t + 5], 4, 4294588738)
        v = u(v, s, m, g, y[t + 8], 11, 2272392833)
        g = u(g, v, s, m, y[t + 11], 16, 1839030562)
        m = u(m, g, v, s, y[t + 14], 23, 4259657740)
        s = u(s, m, g, v, y[t + 1], 4, 2763975236)
        v = u(v, s, m, g, y[t + 4], 11, 1272893353)
        g = u(g, v, s, m, y[t + 7], 16, 4139469664)
        m = u(m, g, v, s, y[t + 10], 23, 3200236656)
        s = u(s, m, g, v, y[t + 13], 4, 681279174)
        v = u(v, s, m, g, y[t + 0], 11, 3936430074)
        g = u(g, v, s, m, y[t + 3], 16, 3572445317)
        m = u(m, g, v, s, y[t + 6], 23, 76029189)
        s = u(s, m, g, v, y[t + 9], 4, 3654602809)
        v = u(v, s, m, g, y[t + 12], 11, 3873151461)
        g = u(g, v, s, m, y[t + 15], 16, 530742520)
        m = u(m, g, v, s, y[t + 2], 23, 3299628645)
        s = d(s, m, g, v, y[t + 0], 6, 4096336452)
        v = d(v, s, m, g, y[t + 7], 10, 1126891415)
        g = d(g, v, s, m, y[t + 14], 15, 2878612391)
        m = d(m, g, v, s, y[t + 5], 21, 4237533241)
        s = d(s, m, g, v, y[t + 12], 6, 1700485571)
        v = d(v, s, m, g, y[t + 3], 10, 2399980690)
        g = d(g, v, s, m, y[t + 10], 15, 4293915773)
        m = d(m, g, v, s, y[t + 1], 21, 2240044497)
        s = d(s, m, g, v, y[t + 8], 6, 1873313359)
        v = d(v, s, m, g, y[t + 15], 10, 4264355552)
        g = d(g, v, s, m, y[t + 6], 15, 2734768916)
        m = d(m, g, v, s, y[t + 13], 21, 1309151649)
        s = d(s, m, g, v, y[t + 4], 6, 4149444226)
        v = d(v, s, m, g, y[t + 11], 10, 3174756917)
        g = d(g, v, s, m, y[t + 2], 15, 718787259)
        m = d(m, g, v, s, y[t + 9], 21, 3951481745)
        s = r(s, n)
        m = r(m, i)
        g = r(g, o)
        v = r(v, a)
        t += 16
    return (p(s) + p(m) + p(g) + p(v)).lower()


cookies = {
    'JSESSIONID': 'abcRb-rKlRPSh9oLFEuXx',
    'OUTFOX_SEARCH_USER_ID': '-227182269@10.108.160.100',
    'OUTFOX_SEARCH_USER_ID_NCOO': '1812441542.1910617',
    'DICT_UGC': 'be3af0da19b5c5e6aa4e17bd8d90b28a|',
    '_ntes_nnid': '80d69e18fb00c654febe4f151e40e0f0,1633526192627',
}

headers = {
    'Connection': 'keep-alive',
    'sec-ch-ua': '"Google Chrome";v="93", " Not;A Brand";v="99", "Chromium";v="93"',
    'DNT': '1',
    'sec-ch-ua-mobile': '?0',
    'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36',
    'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
    'Accept': 'application/json, text/javascript, */*; q=0.01',
    'X-Requested-With': 'XMLHttpRequest',
    'sec-ch-ua-platform': '"macOS"',
    'Origin': 'https://fanyi.youdao.com',
    'Sec-Fetch-Site': 'same-origin',
    'Sec-Fetch-Mode': 'cors',
    'Sec-Fetch-Dest': 'empty',
    'Referer': 'https://fanyi.youdao.com/',
    'Accept-Language': 'zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7',
}

params = (
    ('smartresult', ['dict', 'rule']),
)


def get_result(query, echo=0):
    if echo:
        return '{}: {}'.format(query, _get_result(query))
    return _get_result(query)


def _get_result(query):
    t = int(time.time() * 1000)
    salt = t+random.randint(1, 10)
    sign = md5('fanyideskweb{}{}Y2FYu%TNSbMCxc3t2u^XT'.format(query, salt))
    data = {
        'i': query,
        'from': 'AUTO',
        'to': 'AUTO',
        'smartresult': 'dict',
        'client': 'fanyideskweb',
        'salt': '{}'.format(salt),
        'sign': '{}'.format(sign),
        'lts': '{}'.format(t),
        'bv': '2269d5603709e65f667af23032808f1a',
        'doctype': 'json',
        'version': '2.1',
        'keyfrom': 'fanyi.web',
        'action': 'FY_BY_REALTlME'
    }
    cookies['___rl__test__cookies'] = str(t)
    try:
        res = requests.post(
            'https://fanyi.youdao.com/translate_o',
            headers=headers,
            params=params,
            cookies=cookies,
            data=data,
            timeout=15,
        )
        if res.status_code != 200:
            return '[{}] 返回异常'.format(res.status_code)
        res = res.json()
        if res['errorCode'] != 0:
            if res['errorCode'] == 40:
                return '无结果'
            if res['errorCode'] == 50:
                return '签名错误'
            return '[{}] 返回异常'.format(res['errorCode'])
        result = []
        if 'smartResult' in res:
            entries = res['smartResult']['entries']
            for x in entries:
                if not x:
                    continue
                result.append(x)
        elif 'translateResult' in res:
            if len(res['translateResult']) > 0:
                for x in res['translateResult']:
                    for y in x:
                        result.append(y['tgt'])
        if result:
            return ''.join(result).replace('\r\n', '')
        return '结果错误'
    except requests.RequestException:
        return '请求异常'


if __name__ == '__main__':
    if len(argv) == 2:
        stdout.write(str(get_result(argv[1])))
    elif len(argv) > 2:
        stdout.write(str(get_result(argv[1], int(argv[2]))))
    stdout.flush()
