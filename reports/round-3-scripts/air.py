import math
B = {
 'plains':(12.8,0.42,14.2,[0.27,0.27,0.2,0.14],[3.8,3.0,3.0,3.1]),
 'forest':(9.3,0.77,8.7,[0.34,0.34,0.37,0.41],[2.5,2.0,2.4,3.0]),
 'birch_forest':(6.3,0.8,12.9,[0.29,0.3,0.32,0.47],[1.5,1.1,1.4,1.6]),
 'taiga':(-2.1,0.51,19.8,[0.14,0.32,0.26,0.22],[2.3,2.0,1.5,1.1]),
 'snowy_plains':(-10.2,0.92,14.9,[0.14,0.24,0.32,0.15],[5.3,5.3,6.1,6.0]),
 'desert':(25.7,0.18,8.1,[0.01,0.13,0.06,0.05],[2.3,2.6,1.6,1.4]),
 'savanna':(27.0,0.65,1.6,[0.4,0.7,0.45,0.11],[1.5,1.2,1.2,1.6]),
 'swamp':(25.1,0.42,3.4,[0.18,0.33,0.2,0.07],[3.5,2.9,2.9,3.6]),
 'mountains':(6.0,0.63,8.8,[0.32,0.45,0.31,0.29],[2.4,2.7,2.2,1.7]),
 'ocean':(18.2,0.97,4.1,[0.33,0.19,0.38,0.43],[3.1,2.5,2.7,3.4]),
 'beach':(18.2,0.74,6.2,[0.16,0.01,0.16,0.2],[4.1,3.7,3.4,3.8]),
}
YEAR=56; MIDS=YEAR*3/8
def fmw(day): return ((day-(MIDS+YEAR/2))/YEAR)%1.0
def season(sw,y): return -sw*math.cos(2*math.pi*y)
def feels(c,w):
    k=w*3.6
    if c>10 or k<=4.8: return c
    v=k**0.16
    return min(13.12+0.6215*c-11.37*v+0.3965*c*v,c)
def air(b,day,hour,wind=0.0,rain=False,alt=0.0,open_=True):
    c,h,sw,_,_=B[b]
    swing=2.5+6.5*(1-h)
    d=math.cos(2*math.pi*(hour-15)/24)
    t=c+season(sw,fmw(day+hour/24))+swing*d-0.0065*alt
    snow=False
    if rain:
        tr=t-3 if open_ else t
        if tr<1 and open_: snow=True
    if open_ and 7<=hour<19 and not rain: t+=3*max(d,0)
    if open_ and rain and not snow: t-=3
    if open_ and snow: t-=5
    if open_: t=feels(t,wind)
    return t
def wind(b,season_i,hour,daily=1.0):
    return B[b][4][season_i]*daily*(1+0.25*math.cos((hour-15)/24*2*math.pi))
if __name__=='__main__':
    print("biome   day: night-low still/windy(1.55)  | 15h still | mean")
    for b in B:
        row=[]
        for day in (0,3,6):
            lo=air(b,day,3); lo_w=air(b,day,3,wind(b,0,3,1.55)); hi=air(b,day,15)
            row.append(f"d{day}: {lo:5.1f}/{lo_w:5.1f} {hi:5.1f}")
        c,h,sw,_,_=B[b]
        print(f"{b:13s}", " | ".join(row), f" seasonoff d0 {season(sw,fmw(0)):5.1f} d6 {season(sw,fmw(6.5)):5.1f}")
