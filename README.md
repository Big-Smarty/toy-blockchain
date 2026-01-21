# Overview
This is a toy blockchain implemented on the GPU with slang compute shaders. It both hashes on the GPU and checks on the GPU if the hash fulfills the requirements (the first k nibbles should be all-zeroes, where k is the desired amount of extra points)

# WHY???
We had to do it in CrypTool. Unfortunately, CrypTool doesnt work on linux. I also couldnt get it working through wine and I'll be damned if I install windows or use a windows vm.
 
Thus, to gain aura, I am doing this in vulkan compute shaders. This will also (hopefully) allow me to gain far more extra points than using CrypTool.
 
Also, aura. Because everything about programming in your free time is about learning second and farming aura first.


# TODO
- create a crypto coin???
- farm more aura???
