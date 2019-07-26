import subprocess

files = []
t = 0.0
while (t<1000):
    # print(t)
    i = int(t)
    # print(i)
    f = "result_{:06}.png".format(i)
    #print(f)
    t = 1 + (t*1.1)
    files.append(f)

inputs = " ".join(files)
cmd = "convert -delay 50 {} result.gif".format(inputs)
print(cmd)
subprocess.call( cmd, shell=True  ) 
