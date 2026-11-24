#!/usr/bin/env python3
# GEORGE-NUM-1: where did Hamblin's sin/cos coefficients (src/functions.rs, SINCOS) come from?
# Compares his five 30-bit words with Taylor, minimax, least squares and Chebyshev economisation.
# Needs mpmath and numpy. Run: python3 scripts/cosine-fit.py
import mpmath as mp
mp.mp.dps=40
S=2**30
H=[mp.mpf(1), mp.mpf(-1324675860)/S, mp.mpf(272375348)/S, mp.mpf(-22401135)/S, mp.mpf(985355)/S, mp.mpf(-25564)/S]
f=lambda x: mp.cos(mp.pi*x/2)
def P(c,x): return sum(ci*x**(2*i) for i,ci in enumerate(c))
def err(c, n=4001):
    xs=[mp.mpf(i)/(n-1) for i in range(n)]
    es=[P(c,x)-f(x) for x in xs]
    return xs,es
def extrema(c):
    xs,es=err(c)
    ex=[(xs[0],es[0])]
    for i in range(1,len(es)-1):
        if (es[i]-es[i-1])*(es[i+1]-es[i])<=0 and abs(es[i])>1e-12: ex.append((xs[i],es[i]))
    ex.append((xs[-1],es[-1]))
    return ex
def show(name,c):
    xs,es=err(c); m=max(abs(e) for e in es)
    print(f"{name:42s} max|err| {mp.nstr(m,4):>10s}  coeffs x2^30:", [int(mp.nint(ci*S)) for ci in c[1:]])
    return m
show("Hamblin",H)
print(" Hamblin extrema:", [(mp.nstr(x,4), mp.nstr(e,3)) for x,e in extrema(H)])
# Taylor
T=[(-1)**k*(mp.pi/2)**(2*k)/mp.factorial(2*k) for k in range(6)]
show("Taylor, 6 terms",T)
# Remez minimax, c0=1 fixed, absolute error, basis x^2..x^10
def remez(weight=lambda x:1, fix0=True, iters=30):
    n=5  # free coefficients
    pts=[mp.mpf(i)/(n) for i in range(1,n+2)]  # 6 points in (0,1]
    pts=[ (mp.cos(mp.pi*(n+1-i)/(2*(n+1))) ) for i in range(n+1)]
    pts=[ (1+p)/2 for p in pts]; pts[0]=mp.mpf('0.05')
    for it in range(iters):
        # solve sum c_k x^(2k) + (-1)^i E / w = f - 1
        A=[[pts[i]**(2*k) for k in range(1,n+1)]+[(-1)**i/weight(pts[i])] for i in range(n+1)]
        b=[f(pts[i])-1 for i in range(n+1)]
        sol=mp.lu_solve(mp.matrix(A),mp.matrix(b))
        c=[mp.mpf(1)]+[sol[k] for k in range(n)]
        # new extrema of weighted error on (0,1]
        g=lambda x:(P(c,x)-f(x))*weight(x)
        xs=[mp.mpf(i)/4000 for i in range(1,4001)]
        gs=[g(x) for x in xs]
        new=[]
        for i in range(len(gs)):
            l=gs[i-1] if i>0 else None; r=gs[i+1] if i<len(gs)-1 else None
            if (l is None or abs(gs[i])>=abs(l)) and (r is None or abs(gs[i])>=abs(r)): new.append(xs[i])
        if len(new)>=n+1:
            # keep n+1 largest alternating
            new=sorted(new,key=lambda x:-abs(g(x)))[:n+1]; pts=sorted(new)
    return c
M=remez()
show("minimax abs, c0=1",M)
print(" minimax extrema:", [(mp.nstr(x,4), mp.nstr(e,3)) for x,e in extrema(M)])
R=remez(weight=lambda x: 1/f(x) if x<0.999 else mp.mpf(1)/f(mp.mpf('0.999')))
show("minimax relative-ish, c0=1",R)
# least squares, c0=1
import itertools
xs=[mp.mpf(i)/2000 for i in range(2001)]
A=mp.matrix([[x**(2*k) for k in range(1,6)] for x in xs]); b=mp.matrix([f(x)-1 for x in xs])
ls=mp.lu_solve(A.T*A, A.T*b); L=[mp.mpf(1)]+[ls[k] for k in range(5)]
show("least squares, c0=1",L)
# Chebyshev economisation: Taylor to x^12 (and x^14), drop top term(s) via T_n on [-1,1]
def econ(deg_from):
    import numpy.polynomial.chebyshev as C
    import numpy.polynomial.polynomial as Pn
    c=[0.0]*(deg_from+1)
    for k in range(0,deg_from//2+1): c[2*k]=float((-1)**k*(mp.pi/2)**(2*k)/mp.factorial(2*k))
    ch=C.poly2cheb(c); ch=ch[:11]; p=C.cheb2poly(ch)
    return [mp.mpf(p[2*k]) for k in range(6)]
for d in (12,14,16):
    show(f"Chebyshev economised from x^{d} (c0 free)",econ(d))
# economise in u=x^2 on [0,1] shifted Chebyshev
def econ_u(nterms):
    import numpy as np
    from numpy.polynomial import Chebyshev, Polynomial
    c=[float((-1)**k*(mp.pi/2)**(2*k)/mp.factorial(2*k)) for k in range(nterms)]
    p=Polynomial(c,domain=[0,1],window=[0,1]); ch=p.convert(kind=Chebyshev,domain=[0,1]); ch=Chebyshev(ch.coef[:6],domain=[0,1])
    q=ch.convert(kind=Polynomial,domain=[0,1],window=[0,1]).coef
    return [mp.mpf(q[k]) for k in range(6)]
for n in (7,8):
    show(f"Chebyshev economised in x^2 from {n} terms",econ_u(n))

# --- the closest family: Chebyshev economisation of the Taylor series, exact
Hu=[-1324675860, 272375348, -22401135, 985355, -25564]
t=[(-1)**k*(mp.pi/2)**(2*k)/mp.factorial(2*k) for k in range(9)]
T12=mp.taylor(lambda x: mp.chebyt(12,x),0,12)
e=[t[k]-t[6]*T12[2*k]/T12[12] for k in range(6)]
print("economised from x^12, minus Hamblin, units of 2^-30:", [int(mp.nint(c*S))-h for c,h in zip(e[1:],Hu)])
for nd in range(6,11):
    u=[int(mp.nint(mp.nint(c*10**nd)/10**nd*S)) for c in e[1:]]
    print("  rounded first to", nd, "decimals:", [a-b for a,b in zip(u,Hu)])
