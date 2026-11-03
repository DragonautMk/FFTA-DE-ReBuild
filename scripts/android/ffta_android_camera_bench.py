"""Reproducible RG Rotate camera test; no ROM or save data is distributed.
Requires an imported/verified ROM and the debug Android APK.
The supplied save is only read; Android uses private benchmark.sav.
"""
import argparse
import csv
import io
import math
from pathlib import Path
import subprocess
import time

PACKAGE = "com.fftarecomp.port"
DIRECTIONS = [(0x3ef,300),(0x37f,240),(0x3df,540),(0x3bf,360),
              (0x3ef,540),(0x37f,360),(0x3df,540),(0x3bf,360)]

def camera_input(text, start=9700):
    lines = text.splitlines()
    if not lines or lines[0].strip() != "gba-input v1":
        raise ValueError("Expected gba-input v1")
    events = []
    for line in lines[1:]:
        if not line.strip():
            continue
        frame, keys = line.split()
        frame, keys = int(frame), int(keys,16)
        if frame < 0 or not 0 <= keys <= 1023:
            raise ValueError("Invalid input event")
        events.append((frame,keys))
    if not events or max(f for f,_ in events) >= start-100:
        raise ValueError("Use the original battle-1 replay, ending before frame 9600")
    events += [(start-100,1023),(start-80,1021),(start-70,1023),
               (start-50,1021),(start-40,1023)]
    frame = start
    for keys, duration in DIRECTIONS:
        events += [(frame,keys),(frame+duration,1023)]
        frame += duration+20
    return "gba-input v1\n" + "".join(f"{f} {k:03x}\n" for f,k in sorted(events,key=lambda item:item[0])),frame

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--adb",default="adb")
    parser.add_argument("--serial",required=True)
    parser.add_argument("--input",type=Path,required=True)
    parser.add_argument("--save",type=Path)
    parser.add_argument("--out",type=Path,default=Path("target/android-camera"))
    parser.add_argument("--zoom",type=int,choices=range(7),default=6)
    parser.add_argument("--generate-only",action="store_true")
    parser.add_argument("--single-core",action="store_true")
    parser.add_argument("--unpaced",action="store_true")
    args=parser.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    text,end=camera_input(args.input.read_text(encoding="utf-8-sig"))
    replay=args.out/"camera.input"
    replay.write_text(text,encoding="utf-8")
    if args.generate_only:
        print(f"Generated {replay}; end frame {end}")
        return
    if not args.save or not args.save.is_file():
        parser.error("--save must name your backup; it will not be modified")
    adb=[args.adb,"-s",args.serial]
    def call(*command,check=True):
        result=subprocess.run(adb+list(command),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        if check and result.returncode:
            raise RuntimeError(result.stderr.decode("utf-8",errors="replace"))
        return result.stdout.decode("utf-8",errors="replace")
    call("shell","am","force-stop",PACKAGE)
    call("push",str(replay),"/data/local/tmp/ffta-bench.input")
    call("push",str(args.save),"/data/local/tmp/ffta-bench.sav")
    call("shell","run-as",PACKAGE,"cp","/data/local/tmp/ffta-bench.input","files/benchmark.input")
    call("shell","run-as",PACKAGE,"cp","/data/local/tmp/ffta-bench.sav","files/benchmark.sav")
    call("shell","run-as",PACKAGE,"rm","cache/benchmark.csv","cache/benchmark-camera.csv",
         "cache/benchmark-hashes.txt",check=False)
    scale=2**(args.zoom/6)
    ex=math.ceil((256*scale-240)/2)
    ey=math.ceil((256*scale-160)/2)
    call("shell","am","start","-n",PACKAGE+"/.MainActivity","--ez","ffta_benchmark","true",
         "--ei","ffta_benchmark_frames",str(end),"--ei","ffta_benchmark_extra_x",str(ex),
         "--ei","ffta_benchmark_extra_y",str(ey),"--ez","ffta_benchmark_clock",
         str(not args.unpaced).lower(),"--ez","ffta_benchmark_parallel",str(not args.single_core).lower())
    deadline=time.monotonic()+900
    last=0
    while time.monotonic()<deadline:
        text=call("shell","run-as",PACKAGE,"cat","cache/benchmark.csv",check=False)
        rows=[r for r in csv.DictReader(io.StringIO(text)) if r.get("frame","").isdigit()]
        frame=int(rows[-1]["frame"]) if rows else 0
        if frame//1200!=last//1200:
            print(f"Replay frame {frame}/{end}",flush=True)
        last=frame
        hashes=call("shell","run-as",PACKAGE,"cat","cache/benchmark-hashes.txt",check=False)
        if frame==end and hashes.rstrip().endswith("COMPLETE"):
            break
        time.sleep(2)
    else:
        raise RuntimeError("Benchmark did not finish; check the app on the device")
    for name in ["benchmark.csv","benchmark-camera.csv","benchmark-hashes.txt","performance.txt"]:
        (args.out/name).write_text(call("shell","run-as",PACKAGE,"cat","cache/"+name),encoding="utf-8")
    call("shell","rm","/data/local/tmp/ffta-bench.input","/data/local/tmp/ffta-bench.sav")
    print(f"Complete: {args.out}. The app is paused at the end of the replay.")

if __name__=="__main__":
    main()
