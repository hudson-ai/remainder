Let $R(d, r, c)$ be the regex matching any string $s$ such that
$$
\text{val}(s) + 10^{|s|}c \equiv r \pmod{d}\,.
$$

By the definition of the regex derivative,
$$
\text{deriv}(R(d,r,c), x) = \lbrace s \mid x\cdot s \in R(d,r,c)\rbrace
$$

We have that $x\cdot s \in R(d,r,c) \iff \text{val}(x\cdot s) + 10^{|s|+1}c$. Furthermore, we have that $\text{val}(x\cdot s) = s + 10^{|s|}\text{val}(x)$.

Therefore, $\text{val}(s) + 10^{|s|}\text{val}(x) + 10^{|s+1|}c \equiv r \pmod{d}$.

Rearranging,
$$
\text{val}(s) + 10^{|s|}(10c + \text{val}(x)) \equiv r \pmod{d}\,,
$$

so
$$
\text{deriv}(R(d,r,c), x) = R(d, r, 10c + \text{val}(x))\,.
$$

Finally, observe that $R(d,r,0)$ matches all strings $s$ such that
$$
\text{val}(s) \equiv r \pmod{d}\,,
$$
as needed.