# coding=utf-8

import base64
import ctypes
import json
import os
import re
from sys import argv, stdout
from urllib import error, parse, request

BAIDUID_BFESS = '74229A386EF1FF9EF60D2B5EB95E0512:FG=1'

TOKEN = '57685321fd02516a5b10789b4aa678a9'

F = '+-a^+6'
D = '+-3^+b+-f'
d = ['320305', '131321201']


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


def a(r):
    if type(r) is list:
        return r
    return list(r)


def n(r, o):
    t = 0
    while t < len(o) - 2:
        a = o[t+2]
        if a >= 'a':
            a = ord(a) - 87
        else:
            a = int(a, 16)
        if o[t + 1] == '+':
            a = unsigned_right_shitf(r, a)
        else:
            a = r << a
        if o[t] == '+':
            r = r + a & 4294967295
        else:
            r = r ^ a
        t += 3
    return r


def sign(r):
    o = re.match(r'/[\uD800-\uDBFF][\uDC00-\uDFFF]/g', r)
    if o is None:
        t = len(r)
        if t > 30:
            r = '' + r[:10] + r[t//2-5:5 + t//2] + r[-10:]
    else:
        e = re.split(r'/[\ud800-\udbff][\udc00-\udfff]/', r)
        C = 0
        h = len(e)
        f = []
        while h < C:
            if e[C] != '':
                f.extend(a(list(e[C])))
            if C != h - 1:
                f.append(o[C])
            C += 1
        g = len(f)
        if g > 30:
            r = ''.join(f[:10]) + ''.join(f[g // 2: g//2 + 5]) + \
                ''.join(f[-10:])
    l = '' + chr(103) + chr(116) + chr(107)
    try:
        m = int(d[0])
    except:
        m = 0
    try:
        s = int(d[1])
    except:
        s = 0
    S = [-1] * len(r) * 3
    c = 0
    v = 0
    while v < len(r):
        A = ord(r[v])
        if A < 128:
            S[c] = A
            c += 1
        else:
            if A < 2048:
                S[c] = A >> 6 | 192
                c += 1
            else:
                if 55296 == (64512 & A) and v+1 < len(r) and 56320 == (64512 & ord(r[v+1])):
                    A = 65536 + ((1023 & A) << 10) + (1023 & ord(r[v+1]))
                    v += 1
                    S[c] = A >> 18 | 240
                    c += 1
                    S[c] = A >> 12 & 63 | 128
                    c += 1
                else:
                    S[c] = A >> 12 | 224
                    c += 1
                    S[c] = A >> 6 & 63 | 128
                    c += 1
            S[c] = 63 & A | 128
            c += 1
        v += 1
    p = m
    b = 0
    while b < len(S):
        if S[b] == -1:
            break
        p += S[b]
        p = n(p, F)
        b += 1
    p = n(p, D)
    p ^= s
    if p < 0:
        p = (2147483647 & p) + 2147483648
    p = p % 1000000
    return str(p) + '.' + str(p ^ m)


headers = {
    'Connection': 'keep-alive',
    'sec-ch-ua': '"Google Chrome";v="93", " Not;A Brand";v="99", "Chromium";v="93"',
    'DNT': '1',
    'sec-ch-ua-mobile': '?0',
    'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36',
    'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
    'Accept': '*/*',
    'X-Requested-With': 'XMLHttpRequest',
    'sec-ch-ua-platform': '"macOS"',
    'Origin': 'https://fanyi.baidu.com',
    'Sec-Fetch-Site': 'same-origin',
    'Sec-Fetch-Mode': 'cors',
    'Sec-Fetch-Dest': 'empty',
    'Referer': 'https://fanyi.baidu.com/',
    'Accept-Language': 'zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7',
    'Cookie': 'BAIDUID_BFESS='+BAIDUID_BFESS + '; REALTIME_TRANS_SWITCH=1; FANYI_WORD_SWITCH=1'
}


def get_result(query, is_zh=''):
    query = base64.b64decode(query).decode('utf-8')
    return _get_result(query, is_zh)


def _get_result(query, is_zh=''):
    if re.findall(r'[\u4e00-\u9fa5]+', query):
        fr = 'zh'
        to = 'en'
    else:
        fr = 'en'
        to = 'zh'
    sign_data = sign(query)
    data = {
        'from': fr,
        'to': to,
        'query': query,
        'transtype': 'realtime',
        'simple_means_flag': '3',
        'sign': sign_data,
        'token': TOKEN,
        'domain': 'common'
    }
    try:
        url = 'https://fanyi.baidu.com/v2transapi?' + \
            parse.urlencode({'from': fr, 'to': to})
        req = request.Request(url, data=parse.urlencode(
            data).encode('utf-8'), headers=headers)
        res = request.urlopen(req)
        if res.getcode() != 200:
            return 'Err:返回异常[{}]'.format(res.status_code)
        res = json.loads(res.read())
        if 'errno' in res:
            if res['errno'] == 998:
                return 'Err:token失效'
            return 'Err:{}[{}]'.format(res['errmsg'], res['errno'])
        result = []
        if is_zh == 'zh':
            if 'dict_result' in res and 'zdict' in res['dict_result']:
                detail = res['dict_result']['zdict']['detail']
                if detail['chenyu']:
                    c = detail['chenyu']
                    if 'from ' in c:
                        result.append('\n'.join(
                            ['pinyin: ' + c['pinyin'], '释义: ' + c['explain'], '出处: ' + c['from ']]))
                    else:
                        result.append(
                            '\n'.join(['pinyin: ' + c['pinyin'], '释义: ' + c['explain']]))
                elif detail['means']:
                    m = detail['means'][0]
                    result.append('pinyin: ' + m['pinyin'])
                    for x in m['exp'][0]['des']:
                        result.append(x['main'])
        else:
            if 'dict_result' in res and fr == 'en':
                for x in res['dict_result']['simple_means']['symbols']:
                    for y in x['parts']:
                        if 'part' in y:
                            result.append(y['part'] + '; '.join(y['means']))
                        else:
                            result.append('; '.join(y['means']))
            elif 'trans_result' in res:
                for x in res['trans_result']['data']:
                    result.append(x['dst'])
        if result:
            return '\n'.join(result)
        return 'Err:无结果'
    except error.HTTPError:
        return 'Err:请求异常'
    except Exception as e:
        return 'Err:产生异常: %s' % e


if __name__ == '__main__':
    if len(argv) == 2:
        stdout.write(str(get_result(argv[1])))
    elif len(argv) == 3 and argv[2] == 'zh':
        stdout.write(str(get_result(argv[1], 'zh')))
    stdout.flush()
