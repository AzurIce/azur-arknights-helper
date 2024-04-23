## 傅立叶变换

设 $f(t)$ 为关于时间 $t$ 的函数，它描述了一个信号。

设 $F(w)$ 为关于频率 $w$ 的函数，它也可以描述同一个信号。

傅立叶变换与逆变换即知道其中一者求另一者的魔法：


$$
F(w) = \mathbb{F}[f(t)] = \int_{-\infty}^\infty f(t) e^{i\omega t} dt\\
f(t) = \mathbb{F}^{-1}[F(\omega)] = \frac{1}{2\pi}\int_{-\infty}^\infty F(\omega) e^{i\omega t} d\omega
$$

## 离散傅立叶变换

即离散形式。

设 $\{x_n\}_{n=0}^{N-1}$ 为某一满足有限性条件的序列，其 DFT 为：
$$
X_k = \sum_{n=0}^{N-1} x_ne^{-i\frac{2\pi k}{N}n}
$$


假如讲 $x_n$ 看作是 $f(x)$ 的 $x^n$ 项系数，那么傅立叶变换得到的 $X_k$ 便恰好是 $f(x)$ 代入单位根 $e^{-i\frac{2\pi k}{N}}$ 的值。

于是朴素算法的复杂度也就是 $O(n^2)$ 的。





复数 $n$ 次单位根：$w_n^1, w_n^2, \cdots, w_n^n$

性质 1：$w_{2n}^{2k} = w_n^k$

性质 2：$w_n^{k + \frac{n}{2}} = -w_n^k$



对于 $A(x) = a_0 + a_1 \cdot x^1 + a_2 \cdot x^2 + \cdots + a_{n-1} \cdot x^{n-1}$ 来说，可以将其每一项按照奇偶分组：
$$
A(x) = (a_0 + a_2 \cdot x^2 + \cdots + a_{n-2} \cdot x^{n-2}) + x(a_1 + a_3 \cdot x^2 + \cdots + a_{n-1} \cdot x^{n-2})
$$



$$
f(\omega_n^k) = G(\omega_{n/2}^k) + w_n^k \times H(w_{n/2}^k)\\
f(\omega_n^{k + n/2}) = G(\omega_{n/2}^k) - w_n^k \times H(w_{n/2}^k)
$$




### Bit Reverse

```
n = 8 (1000)

i = 000 ~ 111

i = 0
j = 0

l 100 -> 10 -> 1
j 100   break

i = 001
j = 100

l 100 -> 10 -> 1
j  0     10 break

i = 010
j = 010

l 100 -> 10 -> 1
j 110 break

i = 011
j = 110
```



