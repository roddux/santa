default:
	nasm -felf64 example_asm.s -o example_asm.o
	ld -nmagic example_asm.o -o example_asm
	gcc -o example_c example_c.c
	strip example_asm example_c
clean:
	rm -f example_asm example_c example_asm.o 
