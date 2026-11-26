from air import *
def feel(core):
    if core<33: return 'FREEZ'
    if core<35: return 'cold'
    if core<36.5: return 'chilly'
    if core>40: return 'HEATSTR'
    if core>38.5: return 'hot'
    if core>37.5: return 'warm'
    return 'ok'
def target(air,cold=0,heat=0,fat=0,meal=0,wet=False,wetcold=None):
    c = min(wetcold,cold) if (wet and wetcold is not None) else (0 if wet and wetcold is None and False else cold)
    lo=15-c-4*fat-meal+(6 if wet else 0); hi=28+heat-meal
    if air<lo: return 37-0.35*(lo-air)
    if air>hi: return 37+0.3*(air-hi)
    return 37
FUR=(30,-9,0.3,0.8); WOOL=(14.4,-2,0.6,0.3); HIDE=(9.6,-1,0.3,0.7); LIN=(6,4,0.2,0.1); NONE=(0,0,0,0)
def felt(b,day,hour,windf,cl):
    w=wind(b,0,hour,windf)
    still=air(b,day,hour); ch=air(b,day,hour,w)
    return ch+(still-ch)*cl[3]
if __name__=='__main__':
    import sys
    for cl,name in [(NONE,'naked'),(LIN,'linen'),(HIDE,'hide'),(WOOL,'wool'),(FUR,'fur')]:
        print('==',name)
        for b in B:
            out=[]
            for day in (0,3,6):
                cells=[]
                for hour in (6,15,3):
                    f=felt(b,day if hour!=3 else day+1,hour,1.0,cl)
                    cells.append(feel(target(f,cl[0],cl[1])))
                out.append(f"d{day} 06/15/03: "+"/".join(cells))
            # hours freezing on night d0->d1 (18..30)
            fh=sum(1 for h in range(18,30) if feel(target(felt(b,0 if h<24 else 1,h%24,1.0,cl),cl[0],cl[1]))=='FREEZ')
            print(f"{b:13s}"," | ".join(out), f"| freezing hrs night1 {fh}")
