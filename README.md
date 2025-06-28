This is the critbit binary tree part of the <https://github.com/openbook-dex/openbook-v2> codebase which is the most complex part and so to experiment it directly without setting up anchor and stuffs you can use this repo.  

Here's my explanation on how the limits orders are stored <https://pvnotpv.github.io/posts/openbook> and how it works.

You need to set rust to 1.70 for it to work.

![tree](https://github.com/user-attachments/assets/4eb691c8-ca1b-4ac2-a9cd-2ad727c4e2f7)

You need to set rust to 1.70 for it to work.

```bash
rustup override set 1.70
rustup default 1.70
```

```
The parent key is: 5
The here is: 5 4: 63
----
The parent key is: 4
The here is: 4 6: 62
Keep old parent, Inner prefix len: 63
----
The parent key is: 6
The here is: 6 7: 63
Keep old parent, Inner prefix len: 62
The here is: 6 7: 63
----
The parent key is: 6
The here is: 6 10: 60
Keep old parent, Inner prefix len: 62
----
The parent key is: 10
The here is: 10 15: 61
Keep old parent, Inner prefix len: 60
The here is: 10 15: 61
----
The parent key is: 10
The here is: 10 1: 60
Keep old parent, Inner prefix len: 60
The here is: 6 1: 61
Keep old parent, Inner prefix len: 62
----
The parent key is: 10
The here is: 10 18: 59
Keep old parent, Inner prefix len: 60
----
The parent key is: 18
The here is: 18 20: 61
Keep old parent, Inner prefix len: 59
The here is: 18 20: 61
----
The parent key is: 18
The here is: 18 25: 60
Keep old parent, Inner prefix len: 59
The here is: 20 25: 60
Keep old parent, Inner prefix len: 61
----
The parent key is: 18
The here is: 18 31: 60
Keep old parent, Inner prefix len: 59
The here is: 25 31: 61
Keep old parent, Inner prefix len: 60
The here is: 25 31: 61
----
The parent key is: 18
The here is: 18 40: 58
Keep old parent, Inner prefix len: 59
----
The parent key is: 40
The here is: 40 47: 61
Keep old parent, Inner prefix len: 58
The here is: 40 47: 61
----
The parent key is: 40
The here is: 40 42: 62
Keep old parent, Inner prefix len: 58
The here is: 47 42: 61
Keep old parent, Inner prefix len: 61
The here is: 40 42: 62
----
Root node: 0
Inner node: 0, 0: 22, 1: 21 , Key: 40
Leaf node: 1, Price: 4
Leaf node: 2, Price: 5
Inner node: 3, 0: 6, 1: 5 , Key: 7
Inner node: 4, 0: 1, 1: 2 , Key: 4
Leaf node: 5, Price: 7
Leaf node: 6, Price: 6
Inner node: 7, 0: 10, 1: 9 , Key: 15
Inner node: 8, 0: 11, 1: 12 , Key: 1
Leaf node: 9, Price: 15
Leaf node: 10, Price: 10
Leaf node: 11, Price: 1
Inner node: 12, 0: 4, 1: 3 , Key: 6
Inner node: 13, 0: 18, 1: 17 , Key: 25
Inner node: 14, 0: 8, 1: 7 , Key: 10
Leaf node: 15, Price: 20
Leaf node: 16, Price: 18
Inner node: 17, 0: 20, 1: 19 , Key: 31
Inner node: 18, 0: 16, 1: 15 , Key: 20
Leaf node: 19, Price: 31
```
