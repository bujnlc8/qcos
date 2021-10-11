# coding=utf-8
from __future__ import unicode_literals

import base64
import ctypes
import hashlib
import json
import os
import random
import re
import time
from sys import argv, stdout
from urllib import error, parse, request

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
    'Cookie': '',
}

Cookie = 'OUTFOX_SEARCH_USER_ID=1485736359@10.108.160.102; JSESSIONID=aaaJWUsucwEMl70lf-QXx; OUTFOX_SEARCH_USER_ID_NCOO=1086609560.8868542; '

Url = 'https://fanyi.youdao.com/translate_o?smartresult=dict&smartresult=rule'


def get_result(query):
    query = base64.b64decode(query).decode('utf-8')
    return _get_result(query)


def _get_result(query):
    t = int(time.time() * 1000)
    salt = t+random.randint(1, 10)
    sign = hashlib.md5(str('fanyideskweb{}{}Y2FYu%TNSbMCxc3t2u^XT'.format(
        query, salt)).encode('utf-8')).hexdigest()
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
    headers['Cookie'] = Cookie + '___rl__test__cookies='+str(t)
    try:
        req = request.Request(Url, data=parse.urlencode(
            data).encode('utf-8'), headers=headers)
        res = request.urlopen(req)
        if res.getcode() != 200:
            return 'Err:返回异常[{}]'.format(res.status_code)
        res = json.loads(res.read())
        if res['errorCode'] != 0:
            if res['errorCode'] == 40:
                return 'Err:无结果'
            if res['errorCode'] == 50:
                return 'Err:签名错误'
            return 'Err:返回异常[{}]'.format(res['errorCode'])
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
            return '\n'.join(result)
        return 'Err:无结果'
    except error.HTTPError:
        return 'Err:请求异常'
    except Exception as e:
        return 'Err:产生异常: %s' % e


if __name__ == '__main__':
    if len(argv) >= 2:
        stdout.write(str(get_result(argv[1])))
    stdout.flush()
