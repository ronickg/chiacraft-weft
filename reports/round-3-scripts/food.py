F={ # name: grams, water,prot,carb,fat,fiber,vitc
'apple':(180,85.6,0.3,13.8,0.2,2.4,4.6),'bread':(250,36,9,49,3.3,2.7,0),
'cooked_beef':(150,59,28,0,11,0,0),'cooked_porkchop':(110,61,27,0,8.9,0,0),'cooked_chicken':(220,59.5,27.3,0,13.6,0,0),
'cooked_mutton':(110,57,25,0,16.5,0,0),'beef_raw':(200,70,20.9,0,8,0,0),'cooked_egg':(100,69.5,13.6,0.8,14.8,0,0),
'berries':(100,84.2,0.7,14.5,0.3,2.4,9.7),'lingonberries':(100,87.3,0.5,12,0.1,3.6,14),'crowberries':(100,85,0.6,7,1,4.5,6),
'cloudberries':(100,85,2.4,8.6,0.8,6.3,100),'raspberries':(100,85.8,1.2,11.9,0.7,6.5,26.2),'prickly_pear':(100,87.6,0.7,9.6,0.5,3.6,14),
'hazelnut':(30,5.3,15,16.7,60.8,9.7,6.3),'sunflower':(30,4.7,20.8,20,51.5,8.6,1.4),'mushroom':(50,92.4,3.1,3.3,0.3,1,2.1),
'dried_meat':(75,20,55,2,15,0,0),'smoked_meat':(120,50,30,0,14,0,0),'salted_meat':(160,55,22,0,14,0,0),
}
FULL=[100,250,50,25,500]
def info(n):
    g,w,p,c,f,fi,vc=F[n]; u=g/10
    nut=[p*g/100,c*g/100,f*g/100,(fi+vc/10)*g/100,w*g/100]
    slow=1+4*p/100+6*f/100; wat=1+3*max(w/100-0.75,0); rate=1.5*wat/slow
    mins=u/rate
    return u,mins,nut
print(f"{'food':16s} units  min  prot  carb  fat  green water | fullness% at eating (prot,energy,fat,greens,water)")
for n in F:
    u,m,nut=info(n)
    print(f"{n:16s} {u:5.1f} {m:5.1f} "+" ".join(f"{x:5.1f}" for x in nut)+" | "+" ".join(f"{100*x/FULL[i]:4.0f}" for i,x in enumerate(nut)))
# portions per 20-min day to hold fullness 0.5 of a nutrient, from that food alone (steady state: held = n*rate_eat * mins/2)
print()
for n,i in [('cooked_beef',0),('bread',1),('berries',1),('lingonberries',1),('prickly_pear',1),('hazelnut',2),('cooked_chicken',0)]:
    u,m,nut=info(n)
    per_min=0.5*FULL[i]/(nut[i]*m/2)
    print(f"{n}: {per_min*20:.1f} portions/day = {per_min*20*F[n][0]/1000:.2f} kg/day; stomach units held {per_min*u*m/2:.0f}")
# arrival
u1,m1,n1=info('bread');u2,m2,n2=info('apple')
tot=[a+b for a,b in zip(n1,n2)]
print('arrival fullness', [round(t/FULL[i],2) for i,t in enumerate(tot)], 'maxH',100*(0.4+0.6*tot[0]/100),'maxS',100*(0.4+0.6*min(1,tot[1]/250)))
