# -*- coding: utf-8 -*-
text = 'Выбранная позиция'
print(text.encode('utf-8'))
try:
    print(text.encode('utf-8').decode('cp1251'))
except Exception as e:
    print('error', e)
