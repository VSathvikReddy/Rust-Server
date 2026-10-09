#!/usr/bin/env python3
"""Stress tester for the RCX server. Standard library only.

Modes:
  sustained   each worker has its own token and hits /load and /ask
  shared      all workers share one token (one rate-limit bucket)
  reauth      each worker calls /auth before every request (limiter bypass check)
  auth-flood  only hits /auth
  unauth      sends requests without a token, expects only 401

Example:
  python3 stress_test.py --host 127.0.0.1 --port 3000 --mode sustained --workers 20 --duration 15
"""
import argparse,http.client,json,random,sys,threading,time
from collections import Counter

# Safe inputs per question. Nothing here can hang the server (fib stays <=90).
PAYLOADS={
    0:["hello_world","","abc"*500,"ünïcode"],
    1:["10 20 30 40","1 2 x 3","","-5 5 100"],
    2:["4 17 2 31 9","","nan nan","-1 -2 -3"],
    3:["29","0","18446744073709551615","abc"],
    4:["10","0","1","90","abc"],
}
N_QUESTIONS=len(PAYLOADS)


class Stats:
    def __init__(self):
        self.lock=threading.Lock()
        self.lat=[]
        self.codes=Counter()
        self.errors=Counter()

    def add(self,code,dt,err=None):
        with self.lock:
            self.lat.append(dt)
            self.codes[code]+=1
            if(err):self.errors[err]+=1


def call(conn,method,path,token=None,body=None):
    """One request. Returns (status, text, seconds, error_name_or_None). Status 0 means transport error."""
    headers={}
    data=None
    if(token):headers["Authorization"]="Bearer "+token
    if(body is not None):
        data=json.dumps(body)
        headers["Content-Type"]="application/json"
    t=time.perf_counter()
    try:
        conn.request(method,path,body=data,headers=headers)
        r=conn.getresponse()
        text=r.read().decode(errors="replace")
        return r.status,text,time.perf_counter()-t,None
    except Exception as e:
        conn.close()  # HTTPConnection reopens itself on the next request
        return 0,"",time.perf_counter()-t,type(e).__name__


def fetch_token(conn,stats):
    code,text,dt,err=call(conn,"POST","/auth")
    stats.add(code,dt,err)
    if(code!=200):return None
    text=text.strip()
    try:
        parsed=json.loads(text)
        if(isinstance(parsed,str)):return parsed
        if(isinstance(parsed,dict)):return parsed.get("token")
    except ValueError:
        pass
    return text


def pick_request(bad_rate):
    """Random /load or /ask call, with some out-of-range indices mixed in."""
    r=random.random()
    if(r<0.15):return "GET","/load",None
    if(r<0.15+ bad_rate):return "POST","/ask/%d"%(N_QUESTIONS+ random.randint(0,50)),"x"
    q=random.randrange(N_QUESTIONS)
    return "POST","/ask/%d"%q,random.choice(PAYLOADS[q])


def worker(args,stats,stop_at,shared):
    conn=http.client.HTTPConnection(args.host,args.port,timeout=args.timeout)
    token=shared.get("token") if args.mode=="shared" else None
    while(time.time()<stop_at):
        if(args.mode=="auth-flood"):
            fetch_token(conn,stats)
        else:
            if(args.mode=="reauth"):
                token=fetch_token(conn,stats)
            elif(args.mode=="sustained" and token is None):
                token=fetch_token(conn,stats)
                if(token is None):
                    time.sleep(0.5)
                    continue
            use=None if args.mode=="unauth" else token
            method,path,body=pick_request(args.bad_rate)
            code,_,dt,err=call(conn,method,path,use,body)
            stats.add(code,dt,err)
            if(code==401 and args.mode=="sustained"):token=None  # expired, re-auth next loop
        if(args.delay>0):time.sleep(args.delay)
    conn.close()


def pct(sorted_vals,p):
    if(not sorted_vals):return 0.0
    i=min(len(sorted_vals)-1,int(round(p/100*(len(sorted_vals)-1))))
    return sorted_vals[i]


def report(args,stats,elapsed):
    total=len(stats.lat)
    lat=sorted(stats.lat)
    print("\n=== results ===")
    print("mode:      %s"%args.mode)
    print("workers:   %d"%args.workers)
    print("elapsed:   %.2fs"%elapsed)
    print("requests:  %d  (%.1f req/s)"%(total,total/elapsed if elapsed>0 else 0))
    print("\nstatus codes:")
    for code,n in sorted(stats.codes.items()):
        label="transport error" if code==0 else str(code)
        print("  %-16s %6d  (%.1f%%)"%(label,n,100*n/total if total else 0))
    if(stats.errors):
        print("\ntransport errors:")
        for name,n in stats.errors.most_common():print("  %-24s %d"%(name,n))
    if(lat):
        print("\nlatency (ms):")
        print("  min %.1f   mean %.1f   p50 %.1f   p95 %.1f   p99 %.1f   max %.1f"%(
            lat[0]*1000,sum(lat)/len(lat)*1000,pct(lat,50)*1000,
            pct(lat,95)*1000,pct(lat,99)*1000,lat[-1]*1000))
    verdict(args,stats,total)


def verdict(args,stats,total):
    c=stats.codes
    ok=c.get(200,0)
    limited=c.get(429,0)
    print("\nverdict:")
    if(total==0 or c.get(0,0)==total):
        print("  no requests completed, is the server running?")
    elif(args.mode=="unauth"):
        bad=total-c.get(401,0)
        print("  PASS: every request was rejected with 401"
              if bad==0 else "  FAIL: %d requests were not 401"%bad)
    elif(args.mode=="reauth"):
        print("  rate limiter is bypassable: %d/%d requests succeeded despite the limit"%(ok,total)
              if ok>total*0.5 else "  limiter held up: %d requests were limited (429)"%limited)
    elif(args.mode in("sustained","shared")):
        print("  rate limiter engaged: %d requests got 429"%limited
              if limited else "  no 429s seen, try more workers or a lower --delay")
    else:
        print("  %d tokens issued, %d limited"%(ok,limited))
    if(c.get(500,0)):print("  WARNING: %d responses were 500"%c[500])
    if(c.get(0,0)):print("  WARNING: %d transport errors (timeouts or resets)"%c[0])


def main():
    ap=argparse.ArgumentParser(description="RCX server stress tester")
    ap.add_argument("--host",default="127.0.0.1")
    ap.add_argument("--port",type=int,default=3000)
    ap.add_argument("--mode",default="sustained",
                    choices=["sustained","shared","reauth","auth-flood","unauth"])
    ap.add_argument("--workers",type=int,default=10)
    ap.add_argument("--duration",type=float,default=10,help="seconds")
    ap.add_argument("--delay",type=float,default=0,help="seconds between requests per worker")
    ap.add_argument("--bad-rate",type=float,default=0.05,help="fraction of out-of-range /ask indices")
    ap.add_argument("--timeout",type=float,default=10,help="per-request timeout in seconds")
    ap.add_argument("--seed",type=int,default=None)
    args=ap.parse_args()

    if(args.seed is not None):random.seed(args.seed)

    stats=Stats()
    shared={}
    if(args.mode=="shared"):
        conn=http.client.HTTPConnection(args.host,args.port,timeout=args.timeout)
        shared["token"]=fetch_token(conn,Stats())
        conn.close()
        if(not shared["token"]):
            sys.exit("could not get a token from /auth, is the server up with distribute_keys on?")

    print("hitting %s:%d  mode=%s  workers=%d  duration=%gs"%(
        args.host,args.port,args.mode,args.workers,args.duration))

    start=time.time()
    stop_at=start+ args.duration
    threads=[threading.Thread(target=worker,args=(args,stats,stop_at,shared),daemon=True)
             for _ in range(args.workers)]
    for t in threads:t.start()
    try:
        for t in threads:t.join()
    except KeyboardInterrupt:
        print("\ninterrupted, reporting what we have")
        stop_at=0
    report(args,stats,time.time()-start)


if(__name__=="__main__"):
    main()